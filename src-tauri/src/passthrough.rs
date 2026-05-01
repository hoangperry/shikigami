//! Smart click-through — only the character region accepts clicks; the
//! transparent surroundings pass clicks through to whatever app sits
//! below the overlay (terminal, browser, etc).
//!
//! Tauri's `set_ignore_cursor_events()` is all-or-nothing on every
//! supported platform — there is no per-pixel hit-testing primitive on
//! macOS (`NSWindow`), Windows (`WS_EX_TRANSPARENT`), or Linux (input
//! shape regions). We work around it with a polling loop:
//!
//!   1. Frontend computes the character's screen-space AABB after every
//!      fit() and pushes it via `set_character_bbox` (Tauri command).
//!   2. A background tokio task polls the cursor position at ~60Hz and
//!      flips `set_ignore_cursor_events` based on whether the cursor is
//!      inside the AABB.
//!
//! The flip itself is delegated to Tauri's cross-platform abstraction:
//! `NSWindow.ignoresMouseEvents` on macOS, `WS_EX_TRANSPARENT` toggling
//! on Windows, X11/Wayland input regions on Linux. Polling logic is
//! identical on every platform — no `cfg(target_os)` branches needed.
//!
//! Trade-off: a 16ms pre-click latency (one poll tick) — imperceptible
//! for drag interactions which dominate the use case.
//!
//! Runtime verification status:
//!   - macOS: verified end-to-end by the maintainer.
//!   - Windows: code compiles + tests run on windows-latest CI but the
//!     transparent-overlay end-to-end behaviour is unverified on real
//!     hardware. Tracked in GitHub issue #29.
//!   - Linux: not yet ported (v0.3 milestone).

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::RwLock;

const POLL_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CharacterBBox {
    /// Screen-space (physical pixels) top-left + size of the rendered
    /// character. Frontend computes this from anchor + scale + window pos.
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Default)]
pub struct PassthroughState {
    bbox: RwLock<Option<CharacterBBox>>,
    /// True when smart-passthrough is engaged. False = window fully
    /// clickable everywhere (legacy `click_through = false` behaviour).
    active: RwLock<bool>,
    /// Cached last decision so we don't issue a redundant
    /// `set_ignore_cursor_events` syscall every poll tick.
    last_ignore: RwLock<Option<bool>>,
}

impl PassthroughState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set_bbox(&self, b: CharacterBBox) {
        *self.bbox.write().await = Some(b);
    }

    pub async fn set_active(&self, on: bool) {
        *self.active.write().await = on;
        // Reset cached decision so the next poll definitely applies.
        *self.last_ignore.write().await = None;
    }

    async fn snapshot_bbox(&self) -> Option<CharacterBBox> {
        self.bbox.read().await.clone()
    }
}

/// Spawn the polling loop. Idempotent — safe to call once at startup.
pub fn spawn<R: Runtime>(app: AppHandle<R>, state: Arc<PassthroughState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;

            let active = *state.active.read().await;
            if !active {
                // Smart-passthrough disabled → ensure window is clickable
                // (idempotent via cached last_ignore).
                apply(&app, &state, false).await;
                continue;
            }

            let Some(bbox) = state.snapshot_bbox().await else {
                // Active but no bbox yet — leave clickable so user can
                // still interact with the window during character load.
                apply(&app, &state, false).await;
                continue;
            };

            // cursor_position() returns screen-relative physical pixels.
            // Same coordinate system the frontend uses to compute bbox.
            let Some(window) = app.get_webview_window("main") else {
                continue;
            };
            let Ok(cursor) = window.cursor_position() else {
                continue;
            };

            let inside = cursor.x >= bbox.x
                && cursor.x <= bbox.x + bbox.width
                && cursor.y >= bbox.y
                && cursor.y <= bbox.y + bbox.height;

            // Cursor inside character → window catches click (don't ignore).
            // Cursor outside → window passes click through (ignore).
            apply(&app, &state, !inside).await;
        }
    });
}

async fn apply<R: Runtime>(app: &AppHandle<R>, state: &PassthroughState, ignore: bool) {
    {
        let last = state.last_ignore.read().await;
        if *last == Some(ignore) {
            return; // no change → no syscall
        }
    }
    if let Some(w) = app.get_webview_window("main") {
        if let Err(e) = w.set_ignore_cursor_events(ignore) {
            tracing::warn!("set_ignore_cursor_events({ignore}) failed: {e}");
            return;
        }
    }
    *state.last_ignore.write().await = Some(ignore);
}
