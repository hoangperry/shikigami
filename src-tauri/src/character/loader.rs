//! Filesystem-based character loading. v1 reads an unpacked directory;
//! zip extraction for `.shikigami` packages is Phase 2 follow-up.

use super::manifest::CharacterManifest;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LoadedCharacter {
    pub manifest: CharacterManifest,
    /// Directory on disk holding the manifest.json and assets/ tree.
    pub root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("manifest.json missing at {0}")]
    MissingManifest(PathBuf),
    #[error("manifest JSON parse failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest validation failed: {0}")]
    Invalid(String),
    #[error("asset missing: {0}")]
    AssetMissing(PathBuf),
}

/// Load a character from an unpacked directory (containing manifest.json).
pub fn load_from_dir(dir: &Path) -> Result<LoadedCharacter, LoadError> {
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(LoadError::MissingManifest(manifest_path));
    }
    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: CharacterManifest = serde_json::from_str(&raw)?;

    let issues = manifest.validate();
    if !issues.is_empty() {
        return Err(LoadError::Invalid(issues.join("; ")));
    }

    // Verify every declared state directory exists.
    for (state_name, state) in &manifest.states {
        let state_dir = dir.join(&state.path);
        if !state_dir.is_dir() {
            return Err(LoadError::AssetMissing(state_dir));
        }
        if count_frame_files(&state_dir)? == 0 {
            return Err(LoadError::Invalid(format!(
                "state {:?} has no frame files under {}",
                state_name, state.path
            )));
        }
        // Texture directories are optional; verify only those declared.
        for (tex_name, tex_path) in &state.textures {
            let tex_dir = dir.join(tex_path);
            if !tex_dir.is_dir() {
                return Err(LoadError::Invalid(format!(
                    "state {:?} texture {:?} points to missing dir {}",
                    state_name,
                    tex_name,
                    tex_dir.display()
                )));
            }
        }
    }

    Ok(LoadedCharacter {
        manifest,
        root: dir.to_path_buf(),
    })
}

fn count_frame_files(dir: &Path) -> Result<usize, std::io::Error> {
    let mut count = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let sprite =
            name.starts_with("frame_") && (name.ends_with(".png") || name.ends_with(".webp"));
        let live2d = name.ends_with(".model3.json") || name.ends_with(".moc3");
        if sprite || live2d {
            count += 1;
        }
    }
    Ok(count)
}

impl LoadedCharacter {
    /// Absolute paths to frame files for a given state, sorted by filename.
    pub fn frame_paths(&self, state_name: &str) -> Vec<PathBuf> {
        let Some(state) = self.manifest.states.get(state_name) else {
            return Vec::new();
        };
        let state_dir = self.root.join(&state.path);
        collect_frame_files(&state_dir)
    }

    /// Resolve every texture variant declared on `state_name` to its
    /// absolute frame paths. Used by the registry to expand
    /// `manifest.states[*].textures` (path-only entries) into full
    /// per-variant frame lists for the renderer. Skips variants whose
    /// directory is missing on disk.
    pub fn texture_variant_frames(
        &self,
        state_name: &str,
    ) -> std::collections::BTreeMap<String, Vec<PathBuf>> {
        let mut out = std::collections::BTreeMap::new();
        let Some(state) = self.manifest.states.get(state_name) else {
            return out;
        };
        for (tex_name, tex_path) in &state.textures {
            let dir = self.root.join(tex_path);
            if !dir.is_dir() {
                continue;
            }
            let frames = collect_frame_files(&dir);
            if !frames.is_empty() {
                out.insert(tex_name.clone(), frames);
            }
        }
        out
    }
}

fn collect_frame_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| {
                    (n.starts_with("frame_") && (n.ends_with(".png") || n.ends_with(".webp")))
                        || n.ends_with(".model3.json")
                        || n.ends_with(".moc3")
                })
                .unwrap_or(false)
        })
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        // Resolve relative to the workspace root at test time.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("characters")
            .join("linh-pixel")
    }

    #[test]
    fn loads_linh_pixel_from_disk() {
        let c = load_from_dir(&fixture()).expect("should load");
        assert_eq!(c.manifest.id, "linh-pixel");
        assert!(c.manifest.states.len() >= 2);
    }

    #[test]
    fn frame_paths_are_sorted() {
        let c = load_from_dir(&fixture()).unwrap();
        let frames = c.frame_paths("idle");
        assert!(!frames.is_empty());
        let names: Vec<_> = frames
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn missing_dir_returns_error() {
        let res = load_from_dir(Path::new("/definitely/not/a/real/path/123"));
        assert!(matches!(res, Err(LoadError::MissingManifest(_))));
    }
}
