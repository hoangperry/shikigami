//! In-memory registry of installed characters + active selection.
//!
//! Scan order at startup:
//!   1. ~/.shikigami/characters/* (user installs land here)
//!   2. SHIKIGAMI_DEV_CHARACTERS env var (colon-separated paths, dev convenience)
//!   3. Workspace-relative `./characters/*` (when launching `pnpm tauri:dev` from repo root)

use super::loader::{self, LoadError, LoadedCharacter};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct CharacterRegistry {
    /// Keyed by character id.
    characters: RwLock<BTreeMap<String, LoadedCharacter>>,
    /// Currently active character id.
    active: RwLock<Option<String>>,
}

impl CharacterRegistry {
    pub fn new() -> Self {
        Self {
            characters: RwLock::new(BTreeMap::new()),
            active: RwLock::new(None),
        }
    }

    pub fn load_from_default_paths(&self) -> LoadReport {
        let mut report = LoadReport::default();
        for path in discover_character_paths() {
            match loader::load_from_dir(&path) {
                Ok(character) => {
                    report.loaded.push(character.manifest.id.clone());
                    self.characters
                        .write()
                        .unwrap()
                        .insert(character.manifest.id.clone(), character);
                }
                Err(e) => {
                    report.failed.push((path.clone(), e.to_string()));
                }
            }
        }
        // Auto-select a default if none was chosen yet.
        if self.active.read().unwrap().is_none() {
            if let Some((first_id, _)) = self.characters.read().unwrap().iter().next() {
                *self.active.write().unwrap() = Some(first_id.clone());
            }
        }
        report
    }

    pub fn list_summaries(&self) -> Vec<CharacterSummary> {
        let guard = self.characters.read().unwrap();
        let active = self.active.read().unwrap().clone();
        guard
            .values()
            .map(|c| CharacterSummary {
                id: c.manifest.id.clone(),
                name: c.manifest.name.clone(),
                author: c.manifest.author_name(),
                version: c.manifest.version.clone(),
                is_active: Some(c.manifest.id.clone()) == active,
                default_state: c.manifest.default_state.clone(),
                state_count: c.manifest.states.len(),
            })
            .collect()
    }

    pub fn active_character(&self) -> Option<ActiveCharacter> {
        let active = self.active.read().unwrap().clone()?;
        let guard = self.characters.read().unwrap();
        let ch = guard.get(&active)?;
        Some(ActiveCharacter {
            id: ch.manifest.id.clone(),
            name: ch.manifest.name.clone(),
            default_state: ch.manifest.default_state.clone(),
            states: ch
                .manifest
                .states
                .iter()
                .map(|(k, s)| {
                    (
                        k.clone(),
                        StatePayload {
                            fps: s.fps,
                            r#loop: s.r#loop,
                            then: s.then.clone(),
                            duration_ms: s.duration_ms,
                            frames: ch
                                .frame_paths(k)
                                .into_iter()
                                .map(|p| p.display().to_string())
                                .collect(),
                            textures: s.textures.keys().cloned().collect(),
                        },
                    )
                })
                .collect(),
        })
    }

    pub fn set_active(&self, id: &str) -> bool {
        if self.characters.read().unwrap().contains_key(id) {
            *self.active.write().unwrap() = Some(id.to_string());
            true
        } else {
            false
        }
    }
}

impl Default for CharacterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug)]
pub struct LoadReport {
    pub loaded: Vec<String>,
    pub failed: Vec<(PathBuf, String)>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CharacterSummary {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub is_active: bool,
    pub default_state: String,
    pub state_count: usize,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ActiveCharacter {
    pub id: String,
    pub name: String,
    pub default_state: String,
    pub states: BTreeMap<String, StatePayload>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct StatePayload {
    pub fps: u32,
    pub r#loop: bool,
    pub then: Option<String>,
    pub duration_ms: Option<u32>,
    /// Absolute paths to each frame file. Frontend wraps them in Tauri asset
    /// protocol URLs (`convertFileSrc`) before feeding PIXI.
    pub frames: Vec<String>,
    /// Names of optional texture variants available for this state.
    pub textures: Vec<String>,
}

/// Enumerate candidate directories that may each hold manifest.json.
fn discover_character_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();

    // 1. ~/.shikigami/characters/<id>
    let home = crate::config::paths::characters_dir();
    if let Ok(rd) = std::fs::read_dir(&home) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && p.join("manifest.json").is_file() {
                out.push(p);
            }
        }
    }

    // 2. SHIKIGAMI_DEV_CHARACTERS=/path/a:/path/b
    if let Ok(v) = std::env::var("SHIKIGAMI_DEV_CHARACTERS") {
        for token in v.split(':').filter(|s| !s.is_empty()) {
            let p = PathBuf::from(token);
            if p.is_dir() && p.join("manifest.json").is_file() {
                out.push(p);
            }
        }
    }

    // 3. Workspace-relative `./characters/<id>`  (useful for pnpm tauri:dev)
    if let Ok(cwd) = std::env::current_dir() {
        // Try both cwd and parent (in case cwd is src-tauri/).
        for base in [
            cwd.clone(),
            cwd.parent().map(|p| p.to_path_buf()).unwrap_or(cwd),
        ] {
            let chars_dir = base.join("characters");
            if let Ok(rd) = std::fs::read_dir(&chars_dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() && p.join("manifest.json").is_file() && !out.contains(&p) {
                        out.push(p);
                    }
                }
            }
        }
    }

    out
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("load error: {0}")]
    Load(#[from] LoadError),
}
