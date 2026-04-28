//! ElevenLabs TTS — premium voices, custom voice clones. MP3 output.
//!
//! Auth: `ELEVENLABS_API_KEY` env var (preferred) or `tts.api_key` in config.
//! Voice id (e.g. "21m00Tcm4TlvDq8ikWAM") via `tts.voice`. Discover IDs at
//! https://api.elevenlabs.io/v1/voices.

use super::provider::{TtsError, TtsOutput, TtsProvider};
use crate::config::settings::TtsConfig;
use async_trait::async_trait;
use serde_json::json;
use tokio::io::AsyncWriteExt;

pub struct ElevenLabs {
    cfg: TtsConfig,
}

impl ElevenLabs {
    pub fn new(cfg: TtsConfig) -> Self {
        Self { cfg }
    }

    fn api_key(&self) -> Option<String> {
        std::env::var("ELEVENLABS_API_KEY")
            .ok()
            .or_else(|| self.cfg.api_key.clone())
    }
}

#[async_trait]
impl TtsProvider for ElevenLabs {
    fn name(&self) -> &'static str {
        "elevenlabs"
    }

    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<TtsOutput, TtsError> {
        let key = self.api_key().ok_or(TtsError::MissingKey("elevenlabs"))?;
        // Default to "Rachel" — public sample voice on the free tier.
        let voice_id = voice
            .or(self.cfg.voice.as_deref())
            .unwrap_or("21m00Tcm4TlvDq8ikWAM");

        let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{voice_id}");
        let body = json!({
            "text": text,
            "model_id": "eleven_turbo_v2_5",
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75,
            }
        });

        let resp = reqwest::Client::new()
            .post(&url)
            .header("xi-api-key", key)
            .header("accept", "audio/mpeg")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(TtsError::Other(format!(
                "elevenlabs tts {status}: {body_text}"
            )));
        }

        let bytes = resp.bytes().await?;
        let out = super::fresh_output_path("mp3")?;
        let mut f = tokio::fs::File::create(&out).await?;
        f.write_all(&bytes).await?;

        Ok(TtsOutput {
            path: out,
            mime: "audio/mpeg",
            provider: "elevenlabs",
        })
    }
}
