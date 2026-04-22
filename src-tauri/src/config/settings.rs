//! Persistent user settings and runtime config (port, character id, etc.).

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7796;
pub const PORT_SCAN_SPAN: u16 = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub port: u16,
    pub active_character: Option<String>,
    pub click_through: bool,
    pub opacity: f32,
    pub scale: f32,
    pub auto_hide_during_capture: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            active_character: None,
            click_through: false,
            opacity: 1.0,
            scale: 1.0,
            auto_hide_during_capture: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let p = super::paths::config_file();
        if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str::<Settings>(&s) {
                return cfg;
            }
            tracing::warn!("invalid settings at {}, using defaults", p.display());
        }
        Settings::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let p = super::paths::config_file();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(self)?;
        std::fs::write(p, serialized)
    }
}
