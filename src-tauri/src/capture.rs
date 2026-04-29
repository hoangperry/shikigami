//! Screen-capture detection — best-effort process-based heuristic.
//!
//! macOS only ships fully-reliable capture detection through private
//! APIs (e.g. `CGSGetScreenIsCaptured`) that App Store review rejects.
//! For an OSS desktop overlay we use a public, no-private-API approach
//! instead: every 3 seconds, scan the running process list for known
//! capture / screen-share applications. When `auto_hide_during_capture`
//! is enabled in settings and a match appears, hide the main window;
//! when nothing matches anymore, restore it.
//!
//! Caveats:
//!   - Lists must stay short and well-known. We don't try to detect
//!     every screen-recorder; users with niche tools can hide manually
//!     via the tray menu.
//!   - QuickTime Player is open whenever someone watches a video. We
//!     check it via "ScreenCaptureService" (the helper QuickTime spawns
//!     ONLY during recording) rather than the main app to avoid false
//!     positives.

use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};

const POLL_INTERVAL: Duration = Duration::from_secs(3);

/// Process names commonly associated with screen capture / sharing.
/// Match is case-insensitive substring against `pgrep -lf` output, so
/// e.g. "Loom Helper" matches "loom".
const CAPTURE_PROCESS_HINTS: &[&str] = &[
    "obs",                  // OBS Studio + obs-cmd
    "ScreenCaptureService", // QuickTime / Screenshot.app while recording
    "screencapture",        // CLI tool / SystemUIServer hand-off
    "Loom",
    "ScreenFlow",
    "CleanShot",
    "Kap",
    "zoom.us CptHost", // Zoom screen-share helper
    "Webex",
    "Microsoft Teams",
];

pub fn spawn<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut last_hidden = false;
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;

            // Read freshly each tick — toggling auto-hide in Preferences
            // takes effect on the next poll without restart.
            let cfg = crate::config::Settings::load();
            if !cfg.auto_hide_during_capture {
                if last_hidden {
                    // User just disabled the feature; restore visibility
                    // so we don't leave them with a missing window.
                    show(&app);
                    last_hidden = false;
                }
                continue;
            }

            let capturing = is_capture_active();
            match (capturing, last_hidden) {
                (true, false) => {
                    tracing::info!("capture detected → hiding window");
                    hide(&app);
                    last_hidden = true;
                }
                (false, true) => {
                    tracing::info!("capture ended → showing window");
                    show(&app);
                    last_hidden = false;
                }
                _ => {} // no transition
            }
        }
    });
}

fn is_capture_active() -> bool {
    // `pgrep -lf` lists pid + full command for each match. We pass a
    // single regex alternation built from CAPTURE_PROCESS_HINTS — fewer
    // forks than calling pgrep per hint.
    let pattern = CAPTURE_PROCESS_HINTS.join("|");
    let Ok(out) = Command::new("pgrep").args(["-i", "-lf", &pattern]).output() else {
        return false;
    };
    !out.stdout.is_empty()
}

fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        if let Err(e) = w.hide() {
            tracing::warn!("hide failed: {e}");
        }
    }
}

fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        if let Err(e) = w.show() {
            tracing::warn!("show failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_hints_are_non_empty() {
        // Sanity guard against accidentally emptying the list during a
        // refactor — would silently disable the whole feature.
        assert!(!CAPTURE_PROCESS_HINTS.is_empty());
        for h in CAPTURE_PROCESS_HINTS {
            assert!(!h.is_empty());
        }
    }

    #[test]
    fn pgrep_invocation_works() {
        // Verify the pgrep flag combination exits cleanly. We can't
        // assert a specific match because CI runners vary. `launchd`
        // runs everywhere as PID 1; if pgrep returns *any* exit code
        // (0=found, 1=not found) and produced no error to stderr the
        // call mechanics are sound. Skipped on non-macOS where the
        // flag spelling can differ.
        if !cfg!(target_os = "macos") {
            return;
        }
        let out = Command::new("pgrep")
            .args(["-i", "-lf", "launchd"])
            .output()
            .expect("pgrep available");
        // pgrep itself shouldn't write to stderr on success or empty match.
        assert!(out.stderr.is_empty(), "pgrep stderr: {:?}", out.stderr);
    }
}
