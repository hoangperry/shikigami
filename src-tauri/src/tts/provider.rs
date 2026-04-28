//! Provider abstraction. All concrete TTS backends implement this trait.

use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("tts disabled")]
    Disabled,
    #[error("missing api key for provider {0}")]
    MissingKey(&'static str),
    #[error("missing dependency: {0}")]
    MissingDep(String),
    #[error("subprocess failed: {0}")]
    Subprocess(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("other: {0}")]
    Other(String),
}

/// Result of a TTS synthesis operation.
#[derive(Clone, Debug)]
pub struct TtsOutput {
    /// Absolute path to the audio file on disk.
    pub path: PathBuf,
    /// Audio MIME type, e.g. "audio/mpeg", "audio/wav".
    pub mime: &'static str,
    /// Provider name that produced this audio (for diagnostics).
    pub provider: &'static str,
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// Provider identifier (lowercase, kebab-case).
    fn name(&self) -> &'static str;

    /// Synthesise `text` (optionally overriding the configured voice) into
    /// an audio file under `~/.shikigami/tts/`. Returns the resulting path.
    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<TtsOutput, TtsError>;
}
