//! Canonical state + texture + severity enums.
//!
//! See `docs/adr/002-signal-source.md` for the Hierarchical Fusion design.

use serde::{Deserialize, Serialize};

/// Primary emotion state. Mapped from structured event type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DominantState {
    Idle,
    Happy,
    Focused,
    Warning,
    Confused,
    Sleepy,
    Shy,
    Flirty,
    Overloaded,
}

/// Optional texture modifier extracted from `event.text`. Composes with the
/// dominant state to produce the final animation key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Texture {
    Relieved,
    Playful,
    Exhausted,
    Alarmed,
    Cute,
    Smug,
}

/// Severity tag carried on every event. Scales state duration; `Critical`
/// suppresses the texture layer entirely.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

/// Output of the state machine. Emitted to the frontend as a Tauri event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedState {
    pub dominant: DominantState,
    pub texture: Option<Texture>,
    pub severity: Severity,
    pub duration_ms: u32,
    /// Monotonic counter for debugging; not cryptographic.
    pub event_id: u64,
    /// Truncated copy of `event.text` (<=160 chars) for UI display.
    pub text: Option<String>,
}

impl ResolvedState {
    /// Animation key used by the renderer. Examples: `idle`, `happy_relieved`.
    pub fn animation_key(&self) -> String {
        let dom = serde_json::to_value(self.dominant)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "idle".into());
        match self.texture {
            None => dom,
            Some(t) => {
                let tex = serde_json::to_value(t)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "".into());
                format!("{dom}_{tex}")
            }
        }
    }
}
