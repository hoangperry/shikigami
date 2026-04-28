//! OpenAI TTS provider — `audio.speech` endpoint, MP3 output.
//!
//! Auth: `OPENAI_API_KEY` env var (preferred) or `tts.api_key` in config.

use super::provider::{TtsError, TtsOutput, TtsProvider};
use crate::config::settings::TtsConfig;
use async_trait::async_trait;
use serde_json::json;
use tokio::io::AsyncWriteExt;

pub struct OpenAi {
    cfg: TtsConfig,
}

impl OpenAi {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }

    fn api_key(&self) -> Option<String> {
        std::env::var("OPENAI_API_KEY")
            .ok()
            .or_else(|| self.cfg.api_key.clone())
    }
}

#[async_trait]
impl TtsProvider for OpenAi {
    fn name(&self) -> &'static str {
        "openai"
    }

    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<TtsOutput, TtsError> {
        let key = self.api_key().ok_or(TtsError::MissingKey("openai"))?;
        let voice = voice.or(self.cfg.voice.as_deref()).unwrap_or("alloy");

        // Default model: `gpt-4o-mini-tts` (fastest tier as of 2026-04). User
        // can override later via TtsConfig if needed.
        let body = json!({
            "model": "gpt-4o-mini-tts",
            "voice": voice,
            "input": text,
            "response_format": "mp3",
            "speed": self.cfg.rate.clamp(0.25, 4.0),
        });

        let resp = reqwest::Client::new()
            .post("https://api.openai.com/v1/audio/speech")
            .bearer_auth(key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(TtsError::Other(format!("openai tts {status}: {body_text}")));
        }

        let bytes = resp.bytes().await?;
        let out = super::fresh_output_path("mp3")?;
        let mut f = tokio::fs::File::create(&out).await?;
        f.write_all(&bytes).await?;

        Ok(TtsOutput {
            path: out,
            mime: "audio/mpeg",
            provider: "openai",
        })
    }
}
