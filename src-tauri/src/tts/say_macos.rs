//! macOS `say(1)` provider — zero install, zero key. Best for MVP / fallback.
//!
//! Strategy:
//!   1. Spawn `say -v <voice> -r <rate-wpm> -o <out>.aiff <text>`
//!   2. AIFF is uncompressed PCM and decodable by Web Audio in WKWebView,
//!      so we keep it as-is (no transcode dep on `afconvert`).

use super::provider::{TtsError, TtsOutput, TtsProvider};
use crate::config::settings::TtsConfig;
use async_trait::async_trait;
use tokio::process::Command;

pub struct SayMacos {
    cfg: TtsConfig,
}

impl SayMacos {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl TtsProvider for SayMacos {
    fn name(&self) -> &'static str {
        "say-macos"
    }

    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<TtsOutput, TtsError> {
        let out = super::fresh_output_path("aiff")?;
        let voice = voice.or(self.cfg.voice.as_deref()).unwrap_or("Samantha");
        // `say -r` expects words-per-minute (180 ≈ natural speech). Map our
        // 1.0 = normal multiplier into a reasonable WPM band.
        let wpm = (180.0 * self.cfg.rate.clamp(0.5, 2.0)).round() as u32;

        let status = Command::new("say")
            .arg("-v")
            .arg(voice)
            .arg("-r")
            .arg(wpm.to_string())
            .arg("-o")
            .arg(&out)
            .arg(text)
            .status()
            .await
            .map_err(|e| TtsError::Subprocess(format!("spawn say: {e}")))?;

        if !status.success() {
            return Err(TtsError::Subprocess(format!(
                "say exited with {status} (voice={voice})"
            )));
        }

        Ok(TtsOutput {
            path: out,
            mime: "audio/aiff",
            provider: "say-macos",
        })
    }
}
