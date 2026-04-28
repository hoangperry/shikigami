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
    pub tts: TtsConfig,
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
            tts: TtsConfig::default(),
        }
    }
}

/// TTS provider configuration. `provider = "none"` (default) disables TTS so
/// existing users see no behavior change. API keys may be supplied via env
/// (`OPENAI_API_KEY`, `ELEVENLABS_API_KEY`) which take precedence over the
/// inline `api_key` field — keeps secrets out of `config.json` if desired.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TtsConfig {
    /// One of: "none", "say-macos", "piper", "openai", "elevenlabs".
    pub provider: String,
    /// Provider-specific voice id. e.g. "Linh" for `say`, "alloy" for OpenAI,
    /// `<voice_id>` for ElevenLabs, model path for Piper.
    pub voice: Option<String>,
    /// Optional inline API key. Env var takes precedence.
    pub api_key: Option<String>,
    /// Optional Piper binary path (default: looks up `piper` on $PATH).
    pub piper_binary: Option<String>,
    /// Optional Piper voice model path.
    pub piper_model: Option<String>,
    /// Speech rate hint (provider-specific). 1.0 = normal.
    pub rate: f32,
    /// Playback volume applied client-side via `model.speak({volume})` /
    /// HTMLAudioElement.volume. 0.0 = mute, 1.0 = full. Server-side TTS
    /// providers ignore this — volume is enforced at the renderer.
    pub volume: f32,
    /// When true, fire a short TTS announcement on every meaningful
    /// state transition (focused/happy/warning/critical/etc). Off by
    /// default — most users want quiet ambient reactions plus the
    /// end-of-turn read-aloud, not constant chatter.
    pub announce_events: bool,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: "none".into(),
            voice: None,
            api_key: None,
            piper_binary: None,
            piper_model: None,
            rate: 1.0,
            volume: 1.0,
            announce_events: false,
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
