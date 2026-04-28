//! Background sweep that purges stale TTS audio files. Without this the
//! `~/.shikigami/tts/` directory grows unboundedly across long sessions.
//!
//! Strategy: sweep every `INTERVAL` seconds, delete any file older than
//! `MAX_AGE`. Idempotent and resilient to read failures (we just skip files
//! we can't stat or delete).

use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(5 * 60); // sweep every 5 min
const MAX_AGE: Duration = Duration::from_secs(60 * 60); // keep 1 hour

pub fn spawn_background_sweep() {
    // `tauri::async_runtime::spawn` enters Tauri's managed Tokio runtime so
    // this works whether we're called from setup() (no ambient runtime) or
    // from inside a `#[tokio::main]` test. Plain `tokio::spawn` panics in
    // setup() because did_finish_launching has no reactor in scope.
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(e) = sweep_once() {
                tracing::warn!("tts sweep failed: {e}");
            }
            tokio::time::sleep(INTERVAL).await;
        }
    });
}

fn sweep_once() -> std::io::Result<()> {
    let dir = crate::config::paths::tts_dir();
    if !dir.is_dir() {
        return Ok(());
    }
    let now = std::time::SystemTime::now();
    let mut deleted = 0usize;
    for entry in std::fs::read_dir(&dir)? {
        let Ok(entry) = entry else { continue };
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age > MAX_AGE && std::fs::remove_file(entry.path()).is_ok() {
            deleted += 1;
        }
    }
    if deleted > 0 {
        tracing::debug!("tts sweep: removed {deleted} stale audio file(s)");
    }
    Ok(())
}
