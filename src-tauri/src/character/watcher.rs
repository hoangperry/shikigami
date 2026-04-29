//! Hot-reload watcher for `~/.shikigami/characters/`.
//!
//! Uses `notify-debouncer-mini` to coalesce filesystem bursts into a
//! single signal (the OS emits a flurry of events when a directory is
//! copied in or a manifest is saved by an editor). On debounced change,
//! we rescan the registry and emit a `characters:changed` Tauri event so
//! the frontend can re-fetch the list and (if the active character was
//! affected) full-reload the window with the swapped renderer.

use crate::character::CharacterRegistry;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

const DEBOUNCE: Duration = Duration::from_millis(500);

/// Spawn the watcher on the Tokio blocking pool. The notify library uses
/// std `mpsc::Receiver` so we can't easily integrate into Tokio's async
/// channel without extra plumbing — `spawn_blocking` is the cleaner fit.
pub fn spawn<R: Runtime>(app: AppHandle<R>, registry: Arc<CharacterRegistry>) {
    let dir = crate::config::paths::characters_dir();
    if !dir.is_dir() {
        // No user characters dir yet — skip, the registry already loaded
        // the bundled / dev-time characters at boot. Watcher restarts
        // when the dir appears (next launch).
        tracing::debug!(
            "character watcher skipped — {} does not exist",
            dir.display()
        );
        return;
    }

    tauri::async_runtime::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = match new_debouncer(DEBOUNCE, tx) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("failed to start character watcher: {e}");
                return;
            }
        };
        if let Err(e) = debouncer.watcher().watch(&dir, RecursiveMode::Recursive) {
            tracing::warn!("failed to watch {}: {e}", dir.display());
            return;
        }
        tracing::info!("character watcher: {}", dir.display());

        // Loop forever processing debounced batches.
        for batch in rx {
            match batch {
                Ok(events) if !events.is_empty() => {
                    tracing::info!(
                        "character dir changed ({} events) — rescanning registry",
                        events.len()
                    );
                    registry.reload_from_default_paths();
                    if let Err(e) = app.emit("characters:changed", ()) {
                        tracing::warn!("emit characters:changed failed: {e}");
                    }
                }
                Ok(_) => {} // empty batch (rare)
                Err(errs) => tracing::warn!("watcher errors: {errs:?}"),
            }
        }
    });
}
