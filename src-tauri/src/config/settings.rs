//! Persistent user settings and runtime config (port, character id, etc.).

use serde::{Deserialize, Serialize};
use std::fmt;

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
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TtsConfig {
    /// One of: "none", "say-macos", "piper", "openai", "elevenlabs".
    pub provider: String,
    /// Provider-specific voice id. e.g. "Linh" for `say`, "alloy" for OpenAI,
    /// `<voice_id>` for ElevenLabs, model path for Piper.
    pub voice: Option<String>,
    /// Optional inline API key. Env var takes precedence.
    ///
    /// `skip_serializing_if` keeps the key out of any serialized payload when
    /// absent — combined with [`Settings::redacted`] this guarantees the
    /// plaintext key is never sent across the IPC bridge to the WebView
    /// (which would let a malicious character pack exfiltrate it). On-disk
    /// persistence still works: `save()` runs on a non-redacted `Settings`
    /// so a configured key is written, and a `None` key is simply omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
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

// Manual `Debug` so the API key never lands in a log line, panic backtrace,
// or crash report. The derived impl would print the raw secret via `{:?}`.
impl fmt::Debug for TtsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TtsConfig")
            .field("provider", &self.provider)
            .field("voice", &self.voice)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("piper_binary", &self.piper_binary)
            .field("piper_model", &self.piper_model)
            .field("rate", &self.rate)
            .field("volume", &self.volume)
            .field("announce_events", &self.announce_events)
            .finish()
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
        std::fs::write(&p, serialized)?;
        // Owner-only perms: `config.json` may hold a plaintext TTS API key,
        // so it must not be world-readable on a shared machine. Mirrors the
        // 0600 treatment of the bearer token file (see event::auth).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&p)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&p, perms)?;
        }
        Ok(())
    }

    /// Return a clone with secrets stripped, safe to hand to the WebView.
    ///
    /// The bridge between the Rust core and the renderer is a trust boundary:
    /// the inline TTS `api_key` must never cross it. `get_settings` and the
    /// `settings_changed` event both serialize a redacted copy; persistence
    /// (`save`) always runs on the non-redacted original.
    pub fn redacted(&self) -> Settings {
        let mut copy = self.clone();
        copy.tts.api_key = None;
        copy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_strips_api_key_but_keeps_other_fields() {
        let mut s = Settings::default();
        s.tts.provider = "openai".into();
        s.tts.api_key = Some("sk-secret".into());
        s.tts.voice = Some("alloy".into());

        let r = s.redacted();
        assert_eq!(r.tts.api_key, None);
        assert_eq!(r.tts.provider, "openai");
        assert_eq!(r.tts.voice.as_deref(), Some("alloy"));
        // Original is untouched — persistence must still see the real key.
        assert_eq!(s.tts.api_key.as_deref(), Some("sk-secret"));
    }

    #[test]
    fn serialized_redacted_settings_omit_api_key_entirely() {
        let mut s = Settings::default();
        s.tts.api_key = Some("sk-leak".into());
        let json = serde_json::to_string(&s.redacted()).unwrap();
        assert!(
            !json.contains("api_key"),
            "redacted JSON must not contain the api_key field at all: {json}"
        );
        assert!(!json.contains("sk-leak"));
    }

    #[test]
    fn non_redacted_settings_serialize_api_key_for_persistence() {
        let mut s = Settings::default();
        s.tts.api_key = Some("sk-persist".into());
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("sk-persist"), "save() must persist the key");
    }

    #[test]
    fn debug_impl_redacts_api_key() {
        let cfg = TtsConfig {
            api_key: Some("sk-supersecret".into()),
            ..Default::default()
        };
        let dbg = format!("{cfg:?}");
        assert!(
            !dbg.contains("sk-supersecret"),
            "Debug leaked the key: {dbg}"
        );
        assert!(dbg.contains("<redacted>"));
    }
}
