//! OS-standard paths for Shikigami runtime data.

use std::path::PathBuf;

/// `~/.shikigami/` (or `$SHIKIGAMI_HOME` when set).
///
/// Honoring the env override here keeps the Rust side consistent with
/// `hooks/shikigami-hook.py`, which already reads `SHIKIGAMI_HOME` to
/// support multi-instance / dev / test isolation.
pub fn data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("SHIKIGAMI_HOME") {
        return PathBuf::from(home);
    }
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

/// ~/.shikigami/tts/  — generated TTS audio files (auto-cleaned on rotation).
pub fn tts_dir() -> PathBuf {
    data_dir().join("tts")
}

/// ~/.shikigami/sessions.json — persisted session allowlist (only
/// explicitly muted ids). Restored on every SessionRegistry boot.
pub fn sessions_file() -> PathBuf {
    data_dir().join("sessions.json")
}
