//! Hierarchical Fusion state machine.
//!
//! Pipeline:
//!   1. Event + exit_code → DominantState
//!   2. Severity scales duration; `Critical` suppresses texture
//!   3. event.text → Texture via regex extraction
//!   4. Emit `ResolvedState`

use super::canonical::{DominantState, ResolvedState, Severity};
use super::texture;
use crate::event::schema::{EventPayload, EventType};
use std::sync::atomic::{AtomicU64, Ordering};

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn resolve(event: &EventPayload) -> ResolvedState {
    let dominant = map_event(event);
    let severity = event.severity_or_default();
    let texture = if severity == Severity::Critical {
        None
    } else {
        event.text.as_deref().and_then(texture::extract)
    };
    let base = base_duration(dominant);
    let duration_ms = scale_duration(base, severity);
    ResolvedState {
        dominant,
        texture,
        severity,
        duration_ms,
        event_id: EVENT_COUNTER.fetch_add(1, Ordering::Relaxed),
    }
}

fn map_event(event: &EventPayload) -> DominantState {
    use DominantState::*;
    match (event.event_type, event.exit_code) {
        (EventType::SessionStart, _) => Idle,
        (EventType::SessionEnd, _) => Idle,
        (EventType::SessionIdleLong, _) => Sleepy,
        (EventType::SessionIdleShort, _) => Idle,
        (EventType::UserPrompt, _) => Focused,
        (EventType::ToolStart, _) => Focused,
        (EventType::ToolComplete, Some(0)) => Happy,
        (EventType::ToolComplete, Some(_)) => Warning,
        (EventType::ToolComplete, None) => Happy, // assume success if not reported
        (EventType::Error, _) => Warning,
        (EventType::DestructiveOpDetected, _) => Warning,
        (EventType::GitCommit, _) => Happy,
        (EventType::GitPush, _) => Happy,
        (EventType::AssistantMessage, _) => Idle,
    }
}

/// Base duration in ms for a dominant state. `0` means looping (renderer picks).
fn base_duration(state: DominantState) -> u32 {
    use DominantState::*;
    match state {
        Idle | Focused | Sleepy | Warning => 0, // loop states
        Happy => 1500,
        Confused => 2000,
        Shy => 1000,
        Flirty => 1500,
        Overloaded => 3000,
    }
}

fn scale_duration(base: u32, sev: Severity) -> u32 {
    if base == 0 {
        return 0;
    }
    use Severity::*;
    match sev {
        Info => base,
        Warning => (base as f32 * 1.5) as u32,
        Error => std::cmp::max((base as f32 * 2.0) as u32, 2500),
        Critical => std::cmp::max((base as f32 * 3.0) as u32, 4000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::schema::{EventSource, EventType};

    fn evt(ty: EventType, exit_code: Option<i32>, sev: Option<Severity>) -> EventPayload {
        EventPayload {
            schema_version: "1.0".into(),
            source: EventSource::ClaudeCode,
            event_type: ty,
            tool: None,
            exit_code,
            duration_ms: None,
            severity: sev,
            text: None,
            metadata: None,
        }
    }

    #[test]
    fn tool_complete_success_maps_happy() {
        let s = resolve(&evt(EventType::ToolComplete, Some(0), None));
        assert_eq!(s.dominant, DominantState::Happy);
    }

    #[test]
    fn tool_complete_failure_maps_warning() {
        let s = resolve(&evt(EventType::ToolComplete, Some(1), None));
        assert_eq!(s.dominant, DominantState::Warning);
    }

    #[test]
    fn critical_severity_suppresses_texture() {
        let mut e = evt(
            EventType::DestructiveOpDetected,
            None,
            Some(Severity::Critical),
        );
        e.text = Some("finally done".into()); // would normally trigger Relieved
        let s = resolve(&e);
        assert_eq!(s.texture, None);
    }

    #[test]
    fn text_with_finally_adds_relieved_texture() {
        let mut e = evt(EventType::GitCommit, None, None);
        e.text = Some("fix critical bug, finally".into());
        let s = resolve(&e);
        assert_eq!(s.dominant, DominantState::Happy);
        assert_eq!(s.texture, Some(super::super::canonical::Texture::Relieved));
        assert_eq!(s.animation_key(), "happy_relieved");
    }

    #[test]
    fn error_severity_min_duration_2500ms() {
        let s = resolve(&evt(EventType::GitCommit, None, Some(Severity::Error)));
        assert!(s.duration_ms >= 2500);
    }

    #[test]
    fn looping_state_has_zero_duration() {
        let s = resolve(&evt(EventType::ToolStart, None, None));
        assert_eq!(s.dominant, DominantState::Focused);
        assert_eq!(s.duration_ms, 0);
    }
}
