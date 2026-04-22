//! OS-standard paths for Shikigami runtime data.

use std::path::PathBuf;

/// ~/.shikigami/
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".shikigami")
}

pub fn config_file() -> PathBuf {
    data_dir().join("config.json")
}

pub fn token_file() -> PathBuf {
    data_dir().join("token")
}

pub fn characters_dir() -> PathBuf {
    data_dir().join("characters")
}

pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}
