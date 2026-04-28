//! Demo mode — fires a scripted sequence of synthetic events so the user
//! (or a QA observer) can see every emotion state cycle through without
//! needing to wait for Claude Code to organically trigger them.
//!
//! Triggered from the tray menu. Bypasses the HTTP server and dampener
//! and emits `state_changed` straight to the frontend, mirroring the
//! shape the state machine would produce. Also fires one `/v1/say`-
//! equivalent through the TTS pipeline if a provider is configured.

use crate::state::{DominantState, ResolvedState, Severity};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

static DEMO_EVENT_ID: AtomicU64 = AtomicU64::new(100_000);

/// One step in the scripted demo: dominant + bubble text + how long to
/// hold before advancing.
struct Beat {
    dominant: DominantState,
    severity: Severity,
    text: &'static str,
    hold: Duration,
}

const SCRIPT: &[Beat] = &[
    Beat {
        dominant: DominantState::Focused,
        severity: Severity::Info,
        text: "[demo] PreToolUse → focused",
        hold: Duration::from_millis(2200),
    },
    Beat {
        dominant: DominantState::Happy,
        severity: Severity::Info,
        text: "[demo] PostToolUse exit=0 → happy",
        hold: Duration::from_millis(2200),
    },
    Beat {
        dominant: DominantState::Warning,
        severity: Severity::Warning,
        text: "[demo] PostToolUse exit≠0 → warning",
        hold: Duration::from_millis(2400),
    },
    Beat {
        dominant: DominantState::Warning,
        severity: Severity::Critical,
        text: "[demo] destructive op detected — critical",
        hold: Duration::from_millis(2600),
    },
    Beat {
        dominant: DominantState::Sleepy,
        severity: Severity::Info,
        text: "[demo] idle 10min → sleepy",
        hold: Duration::from_millis(2400),
    },
    Beat {
        dominant: DominantState::Idle,
        severity: Severity::Info,
        text: "[demo] back to idle",
        hold: Duration::from_millis(1200),
    },
];

/// Run the demo on Tauri's async runtime. Non-blocking — the tray menu
/// returns immediately while playback continues. Generic over the
/// runtime so the tray (which uses `AppHandle<R>`) can call us directly.
pub fn spawn<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        tracing::info!("demo: starting playback");
        for beat in SCRIPT {
            let resolved = ResolvedState {
                dominant: beat.dominant,
                texture: None,
                severity: beat.severity,
                duration_ms: 0,
                event_id: DEMO_EVENT_ID.fetch_add(1, Ordering::Relaxed),
                text: Some(beat.text.into()),
            };
            if let Err(e) = app.emit("state_changed", &resolved) {
                tracing::warn!("demo emit failed: {e}");
            }
            tokio::time::sleep(beat.hold).await;
        }
        tracing::info!("demo: playback complete");
    });
}
