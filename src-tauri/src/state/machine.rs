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
        text: event
            .text
            .as_deref()
            .map(|t| t.chars().take(160).collect::<String>()),
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
        (EventType::ToolStart, _) => map_tool_start(event.tool.as_deref()),
        (EventType::ToolComplete, Some(0)) => map_tool_complete(event.tool.as_deref()),
        (EventType::ToolComplete, Some(_)) => Warning,
        (EventType::ToolComplete, None) => map_tool_complete(event.tool.as_deref()),
        (EventType::Error, _) => Warning,
        (EventType::DestructiveOpDetected, _) => Warning,
        (EventType::GitCommit, _) => Happy,
        (EventType::GitPush, _) => Happy,
        (EventType::AssistantMessage, _) => Idle,
    }
}

/// Tool-aware dominant for `ToolStart`. Reading / scanning tools and deep
/// reasoning tools (Task subagents, web fetches) get a distinct mood so
/// the user can read intent at a glance — Hiyori "focuses" for code work
/// and "thinks" for research / delegation work.
///
/// Characters that don't define every dominant fall back via the renderer's
/// resolveAnimKey to their default state.
fn map_tool_start(tool: Option<&str>) -> DominantState {
    use DominantState::*;
    match tool {
        // Deep-reasoning / external lookups → "thinking" mood.
        Some("Task") | Some("WebFetch") | Some("WebSearch") | Some("ToolSearch") => Confused,
        // Read-only inspection → focused (default).
        Some("Read") | Some("Grep") | Some("Glob") | Some("ListDir") => Focused,
        // Writes / mutations → focused.
        Some("Write") | Some("Edit") | Some("MultiEdit") | Some("NotebookEdit") => Focused,
        // Shell — many things, but usually focused work.
        Some("Bash") => Focused,
        // Planning → happy little nod (TodoWrite means progress is being tracked).
        Some("TodoWrite") | Some("TodoRead") => Happy,
        // Subagent dispatch (Agent tool) → thinking deeply.
        Some("Agent") => Confused,
        // Default: focused.
        _ => Focused,
    }
}

/// Tool-aware dominant for `ToolComplete` (success path). Most tools just
/// resolve to Happy; planning tools also Happy; long-running research
/// completions get a "shy/relieved" reaction via texture extraction
/// downstream.
fn map_tool_complete(tool: Option<&str>) -> DominantState {
    use DominantState::*;
    match tool {
        // Read-only inspection finishing → quiet idle nod (less hype).
        Some("Read") | Some("Grep") | Some("Glob") | Some("ListDir") => Idle,
        // Everything else: little happy beat.
        _ => Happy,
    }
}

/// Base duration in ms for a dominant state. `0` means looping (renderer picks).
fn base_duration(state: DominantState) -> u32 {
    use DominantState::*;
    match state {
        Idle | Focused | Sleepy | Warning => 0, // loop states
        Happy => 1500,
        Confused => 2000,
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
            session_id: None,
            cwd: None,
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

    fn evt_with_tool(ty: EventType, tool: &str, exit: Option<i32>) -> EventPayload {
        let mut e = evt(ty, exit, None);
        e.tool = Some(tool.into());
        e
    }

    #[test]
    fn task_tool_start_is_confused() {
        let s = resolve(&evt_with_tool(EventType::ToolStart, "Task", None));
        assert_eq!(s.dominant, DominantState::Confused);
    }

    #[test]
    fn web_fetch_start_is_confused() {
        let s = resolve(&evt_with_tool(EventType::ToolStart, "WebFetch", None));
        assert_eq!(s.dominant, DominantState::Confused);
    }

    #[test]
    fn read_complete_falls_to_idle() {
        let s = resolve(&evt_with_tool(EventType::ToolComplete, "Read", Some(0)));
        assert_eq!(s.dominant, DominantState::Idle);
    }

    #[test]
    fn todowrite_start_is_happy() {
        let s = resolve(&evt_with_tool(EventType::ToolStart, "TodoWrite", None));
        assert_eq!(s.dominant, DominantState::Happy);
    }

    #[test]
    fn unknown_tool_defaults_focused() {
        let s = resolve(&evt_with_tool(EventType::ToolStart, "NewMysteryTool", None));
        assert_eq!(s.dominant, DominantState::Focused);
    }
}
