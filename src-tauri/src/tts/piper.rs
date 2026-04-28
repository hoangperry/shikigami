//! Piper TTS provider — local neural TTS. Offline, fast (~200ms for short
//! lines), good quality. Requires `piper` binary + a `.onnx` voice model.
//!
//! Install (macOS): `brew install piper-tts`.
//! Download voice models from https://huggingface.co/rhasspy/piper-voices

use super::provider::{TtsError, TtsOutput, TtsProvider};
use crate::config::settings::TtsConfig;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct Piper {
    cfg: TtsConfig,
}

impl Piper {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl TtsProvider for Piper {
    fn name(&self) -> &'static str {
        "piper"
    }

    async fn synthesize(&self, text: &str, _voice: Option<&str>) -> Result<TtsOutput, TtsError> {
        let bin = self.cfg.piper_binary.as_deref().unwrap_or("piper");
        let model = self
            .cfg
            .piper_model
            .as_deref()
            .ok_or_else(|| TtsError::MissingDep("piper_model not configured".into()))?;

        let out = super::fresh_output_path("wav")?;

        // Piper reads text from stdin, writes WAV to --output_file.
        let mut child = Command::new(bin)
            .arg("--model")
            .arg(model)
            .arg("--output_file")
            .arg(&out)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TtsError::MissingDep(format!("piper binary not found ({bin}): {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .await
                .map_err(|e| TtsError::Subprocess(format!("piper stdin: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| TtsError::Subprocess(format!("piper wait: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TtsError::Subprocess(format!(
                "piper exited {}: {stderr}",
                output.status
            )));
        }

        Ok(TtsOutput {
            path: out,
            mime: "audio/wav",
            provider: "piper",
        })
    }
}
