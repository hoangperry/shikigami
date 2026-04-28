//! Event payload structure. Mirrors `schemas/event.v1.0.json`.

use crate::state::Severity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventPayload {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub source: EventSource,
    #[serde(rename = "type")]
    pub event_type: EventType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, rename = "exitCode", skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(
        default,
        rename = "durationMs",
        skip_serializing_if = "Option::is_none"
    )]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Originating session identifier — lets the backend group / filter
    /// events when multiple Claude Code tabs send to the same instance.
    #[serde(default, rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Working directory of the session — used as a friendly label in
    /// the session picker UI ("shikigami", "tts-server", …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventSource {
    ClaudeCode,
    Cursor,
    Windsurf,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStart,
    SessionEnd,
    SessionIdleShort,
    SessionIdleLong,
    UserPrompt,
    AssistantMessage,
    ToolStart,
    ToolComplete,
    Error,
    DestructiveOpDetected,
    GitCommit,
    GitPush,
}

impl EventPayload {
    /// Verify schemaVersion matches v1.0.
    pub fn validate_version(&self) -> Result<(), String> {
        if self.schema_version == "1.0" {
            Ok(())
        } else {
            Err(format!(
                "unsupported schemaVersion {:?}; expected \"1.0\"",
                self.schema_version
            ))
        }
    }

    pub fn severity_or_default(&self) -> Severity {
        self.severity.unwrap_or_default()
    }
}

/// Body of `POST /v1/say`. Synthesises `text` to audio via the configured
/// TTS provider, then emits a `tts:speak` Tauri event with the audio path.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SayRequest {
    pub text: String,
    /// Optional override of the configured voice id.
    #[serde(default)]
    pub voice: Option<String>,
}

/// Payload emitted on the Tauri `tts:speak` event. Frontend uses `audio_url`
/// (Tauri asset protocol URL) to drive Live2D `model.speak()`.
#[derive(Clone, Debug, Serialize)]
pub struct SpeakEvent {
    pub audio_url: String,
    pub mime: &'static str,
    pub provider: &'static str,
    pub text: String,
}
