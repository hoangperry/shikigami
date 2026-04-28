//! `.shikigami` zip-package installer.
//!
//! ADR-003 defines the package format: a standard ZIP archive containing
//! `manifest.json` at the root plus `assets/states/<state>/...` (sprite)
//! or a Live2D `<Model>.model3.json` tree inside `assets/states/idle/`.
//!
//! Install pipeline:
//!   1. Open the archive and read `manifest.json` straight from memory
//!      so we can reject malformed bundles before touching the disk.
//!   2. Validate the manifest (SPDX license, required states, paths).
//!   3. Resolve the destination as `~/.shikigami/characters/<id>/`.
//!      Reject installs that would escape that root via `..` or
//!      symlinks (zip-slip defence).
//!   4. Extract atomically: write to a sibling temp dir, then `rename`.
//!      Half-installed characters never appear on disk.
//!   5. Caller triggers a registry reload (the filesystem watcher
//!      already does this for `~/.shikigami/characters/` mutations,
//!      so installs flow through the normal hot-reload path).

use crate::character::manifest::CharacterManifest;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("manifest.json missing from archive")]
    MissingManifest,
    #[error("manifest.json is not valid JSON: {0}")]
    BadManifestJson(#[from] serde_json::Error),
    #[error("manifest validation failed: {0}")]
    ManifestInvalid(String),
    #[error(
        "archive entry {entry:?} resolves outside the install dir — refusing to extract \
         (zip-slip)"
    )]
    PathEscape { entry: String },
}

/// Result of a successful install. Contains the new character id + the
/// directory it was extracted to, for callers that want to log or trigger
/// a follow-up registry refresh.
#[derive(Clone, Debug)]
pub struct Installed {
    pub id: String,
    pub install_dir: PathBuf,
}

/// Install a `.shikigami` zip from `archive_path`. Overwrites any
/// existing install for the same id (after validation passes).
pub fn install_zip(archive_path: &Path) -> Result<Installed, InstallError> {
    let file = std::fs::File::open(archive_path)?;
    let mut zip = ZipArchive::new(file)?;

    // -- Step 1+2: read + validate manifest before touching disk --
    let manifest = read_manifest(&mut zip)?;
    let issues = manifest.validate();
    if !issues.is_empty() {
        return Err(InstallError::ManifestInvalid(issues.join("; ")));
    }

    // -- Step 3: resolve destination + zip-slip defence --
    let chars_dir = crate::config::paths::characters_dir();
    std::fs::create_dir_all(&chars_dir)?;
    let final_dir = chars_dir.join(&manifest.id);
    let staging_dir = chars_dir.join(format!(".{}.installing", manifest.id));
    let _ = std::fs::remove_dir_all(&staging_dir);
    std::fs::create_dir_all(&staging_dir)?;

    // -- Step 4: extract every entry with path-escape checks --
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let entry_name = entry.name().to_string();
        let Some(rel) = entry.enclosed_name() else {
            // Returns None for absolute paths or paths containing `..`
            // segments — both of which we refuse.
            return Err(InstallError::PathEscape { entry: entry_name });
        };

        let dest = staging_dir.join(&rel);
        if !dest.starts_with(&staging_dir) {
            return Err(InstallError::PathEscape { entry: entry_name });
        }

        if entry.is_dir() {
            std::fs::create_dir_all(&dest)?;
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = std::fs::File::create(&dest)?;
        std::io::copy(&mut entry, &mut out)?;
    }

    // -- Step 5: atomic rename into final location --
    let _ = std::fs::remove_dir_all(&final_dir);
    std::fs::rename(&staging_dir, &final_dir)?;

    Ok(Installed {
        id: manifest.id,
        install_dir: final_dir,
    })
}

fn read_manifest<R: std::io::Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<CharacterManifest, InstallError> {
    // Try common locations: top-level `manifest.json` (ADR-003 spec) or
    // `<id>/manifest.json` (some authors zip the parent directory).
    let mut buf = String::new();
    let candidates = ["manifest.json"];
    for name in candidates {
        if let Ok(mut f) = zip.by_name(name) {
            buf.clear();
            f.read_to_string(&mut buf)?;
            return Ok(serde_json::from_str(&buf)?);
        }
    }
    // Fallback: any `*/manifest.json` at depth 1.
    let nested = (0..zip.len())
        .find_map(|i| {
            let f = zip.by_index(i).ok()?;
            let n = f.name();
            if n.ends_with("/manifest.json") && n.matches('/').count() == 1 {
                Some(n.to_string())
            } else {
                None
            }
        })
        .ok_or(InstallError::MissingManifest)?;
    let mut f = zip.by_name(&nested)?;
    f.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let opts = zip::write::SimpleFileOptions::default();
            for (name, data) in entries {
                w.start_file(*name, opts).unwrap();
                w.write_all(data).unwrap();
            }
            w.finish().unwrap();
        }
        buf
    }

    fn minimal_manifest() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schemaVersion": "1.0",
            "id": "test-pkg",
            "name": "T",
            "author": "x",
            "version": "1.0.0",
            "license": "MIT",
            "renderer": "sprite",
            "defaultState": "idle",
            "states": {
                "idle":  { "path": "assets/states/idle",  "fps": 12 },
                "happy": { "path": "assets/states/happy", "fps": 12 }
            }
        }))
        .unwrap()
    }

    #[test]
    fn rejects_missing_manifest() {
        let zip_bytes = build_zip(&[("README.md", b"hello")]);
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("shikigami-test-{}.zip", std::process::id()));
        std::fs::write(&tmp, &zip_bytes).unwrap();
        let err = install_zip(&tmp).unwrap_err();
        assert!(matches!(err, InstallError::MissingManifest), "got {err:?}");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn rejects_zip_slip_attempt() {
        let zip_bytes = build_zip(&[
            ("manifest.json", &minimal_manifest()),
            ("../escape.txt", b"evil"),
        ]);
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("shikigami-slip-{}.zip", std::process::id()));
        std::fs::write(&tmp, &zip_bytes).unwrap();
        let err = install_zip(&tmp).unwrap_err();
        assert!(
            matches!(err, InstallError::PathEscape { .. }),
            "expected path escape rejection, got {err:?}"
        );
        let _ = std::fs::remove_file(&tmp);
    }
}
