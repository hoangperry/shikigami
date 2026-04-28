//! Text-to-speech module — synthesises spoken audio from text and writes
//! the result to `~/.shikigami/tts/<uuid>.<ext>`. The audio path is then
//! emitted to the frontend so the Live2D renderer can drive lip-sync via
//! `pixi-live2d-display-mulmotion::speak()`.
//!
//! Multi-provider design (KISS):
//!   - `say-macos`     → built-in `say(1)` (no install, no key)
//!   - `piper`         → local Piper TTS subprocess (offline neural)
//!   - `openai`        → OpenAI `audio.speech` API (cloud)
//!   - `elevenlabs`    → ElevenLabs `text-to-speech` API (cloud, premium)
//!   - `none`          → disabled (default)
//!
//! All providers share the `TtsProvider` trait; `synthesize()` returns the
//! absolute path of the produced audio file.

pub mod cleanup;
pub mod elevenlabs;
pub mod openai;
pub mod piper;
pub mod provider;
pub mod say_macos;

use crate::config::settings::TtsConfig;
pub use provider::{TtsError, TtsOutput, TtsProvider};

/// Build a provider implementation from settings. Returns `None` if disabled.
pub fn build(cfg: &TtsConfig) -> Option<Box<dyn TtsProvider>> {
    match cfg.provider.as_str() {
        "say-macos" => Some(Box::new(say_macos::SayMacos::new(cfg.clone()))),
        "piper" => Some(Box::new(piper::Piper::new(cfg.clone()))),
        "openai" => Some(Box::new(openai::OpenAi::new(cfg.clone()))),
        "elevenlabs" => Some(Box::new(elevenlabs::ElevenLabs::new(cfg.clone()))),
        "none" | "" => None,
        other => {
            tracing::warn!("unknown tts.provider {other:?}; TTS disabled");
            None
        }
    }
}

/// Ensure the TTS output directory exists. Idempotent.
pub fn ensure_output_dir() -> std::io::Result<std::path::PathBuf> {
    let dir = crate::config::paths::tts_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Generate a unique output path with the provider-chosen extension.
pub fn fresh_output_path(extension: &str) -> std::io::Result<std::path::PathBuf> {
    let dir = ensure_output_dir()?;
    let id = uuid::Uuid::new_v4();
    Ok(dir.join(format!("{id}.{extension}")))
}
