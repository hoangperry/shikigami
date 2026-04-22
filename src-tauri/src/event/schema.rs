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
    #[serde(
        default,
        rename = "exitCode",
        skip_serializing_if = "Option::is_none"
    )]
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
