//! Idle-timer background task — fires a synthetic `SessionIdleLong` event
//! after a stretch of inactivity so the character drifts into the
//! `sleepy` state. Without this Hiyori would loop the last "active"
//! state forever when the user steps away.
//!
//! Design choices (KISS):
//!   - Single shared `Instant` updated on every real event ingress
//!   - One Tokio task polls every 30s; if `now - last > IDLE_THRESHOLD`
//!     and we haven't already fired idle, emit + remember
//!   - No HTTP roundtrip — call the in-process emitter directly
//!   - Resets the moment any new real event arrives

use crate::state::resolve;
use rand::seq::SliceRandom;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const POLL_INTERVAL: Duration = Duration::from_secs(30);
/// Long-idle threshold — after this much silence Hiyori naps. 10 minutes
/// matches a typical "user went to grab coffee" stretch.
const IDLE_THRESHOLD: Duration = Duration::from_secs(10 * 60);

/// Pool of idle nudge messages — Hiyori speaks one at random when the
/// idle threshold is crossed (only when TTS is enabled). Vietnamese
/// because the default voice (`Linh`) is vi_VN and the user's primary
/// language. Add localisations later if we ship multi-language voices.
pub const IDLE_NUDGE_MESSAGES: &[&str] = &[
    "Anh ơi đang làm gì đó? Em buồn ngủ rồi meow.",
    "Hơi yên tĩnh quá nha, em đợi anh nãy giờ.",
    "Em đi ngủ một chút nhé, anh rảnh thì gọi em dậy meow.",
    "Anh nghỉ tay rồi à? Đừng quên uống nước nhé.",
    "Em đứng đây mãi thấy buồn, anh sai em việc gì đi.",
];

pub fn pick_idle_message() -> &'static str {
    let mut rng = rand::thread_rng();
    IDLE_NUDGE_MESSAGES
        .choose(&mut rng)
        .copied()
        .unwrap_or("meow~")
}

/// Shared idle-tracker state. Wrap in `Arc<IdleTracker>` and clone freely.
pub struct IdleTracker {
    last_event: Mutex<Instant>,
    /// Whether we've already fired the synthetic idle event for the
    /// current quiet stretch. Reset on the next real event.
    fired: Mutex<bool>,
}

impl IdleTracker {
    pub fn new() -> Self {
        Self {
            last_event: Mutex::new(Instant::now()),
            fired: Mutex::new(false),
        }
    }

    /// Record that a real event just arrived — resets the idle window.
    pub async fn touch(&self) {
        *self.last_event.lock().await = Instant::now();
        *self.fired.lock().await = false;
    }
}

impl Default for IdleTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn the background watcher. Calls `emit` with a synthesised
/// `SessionIdleLong` event when the threshold is crossed.
pub fn spawn(tracker: Arc<IdleTracker>, emit: Arc<dyn Fn() + Send + Sync>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            let last = *tracker.last_event.lock().await;
            // Lock-update-release pattern: the `fired` mutex is held only
            // while we mutate the flag. emit() must run OUTSIDE the lock
            // — it's a user-supplied callback that today only `spawn`s,
            // but a future blocking body would otherwise stall every
            // `touch()` call from the HTTP server.
            let should_emit = {
                let mut fired = tracker.fired.lock().await;
                let trigger = !*fired && last.elapsed() >= IDLE_THRESHOLD;
                if trigger {
                    *fired = true;
                }
                trigger
            };
            if should_emit {
                tracing::info!("idle threshold reached → emitting sleepy state");
                emit();
            }
        }
    });
}

/// Build the `ResolvedState` we want to emit when the timer fires.
/// Uses the same state machine path so all downstream consumers see a
/// consistent payload shape.
pub fn synth_idle_state() -> crate::state::ResolvedState {
    use crate::event::schema::{EventPayload, EventSource, EventType};
    let evt = EventPayload {
        schema_version: "1.0".into(),
        source: EventSource::Generic,
        event_type: EventType::SessionIdleLong,
        tool: None,
        exit_code: None,
        duration_ms: None,
        severity: None,
        text: None,
        metadata: None,
        session_id: None,
        cwd: None,
    };
    resolve(&evt)
}
