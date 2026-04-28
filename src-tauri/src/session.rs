//! Session registry — tracks every distinct Claude Code session that has
//! posted an event to the local server, plus a per-session "allowed" flag.
//! Lets the user (via the Preferences modal) choose which sessions
//! Hiyori reacts to when multiple Claude Code tabs are running.
//!
//! Default policy: every newly seen session is **allowed**. The user
//! flips a session off explicitly. Sessions with no events for
//! `STALE_AFTER` are pruned so the picker doesn't accumulate forever.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const STALE_AFTER: Duration = Duration::from_secs(60 * 60); // 1 hour

#[derive(Clone, Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    /// Best-effort short label (basename of cwd, or session id prefix).
    pub label: String,
    pub cwd: Option<String>,
    pub event_count: u64,
    /// Unix epoch millis — frontend can format relative ("2m ago").
    pub last_seen_ms: u64,
    pub allowed: bool,
}

#[derive(Default)]
pub struct SessionRegistry {
    inner: RwLock<HashMap<String, Entry>>,
    /// Persisted set of explicitly-muted session ids. Survives restart.
    /// Loaded once at boot, written through on every `set_allowed(false)`.
    /// Only mutes are persisted — unknown ids default to allowed.
    muted: RwLock<HashSet<String>>,
}

/// Disk shape for `~/.shikigami/sessions.json`. Compact, easy to hand-edit
/// if the user wants to bulk-clear mutes.
#[derive(Default, Serialize, Deserialize)]
struct PersistedAllowlist {
    /// Session ids the user has explicitly muted in Preferences.
    #[serde(default)]
    muted: Vec<String>,
}

#[derive(Clone, Debug)]
struct Entry {
    cwd: Option<String>,
    event_count: u64,
    last_seen: Instant,
    last_seen_ms: u64,
    allowed: bool,
}

impl SessionRegistry {
    /// Boot the registry, restoring any previously-muted session ids from
    /// disk. Missing / unreadable file = empty mute set (graceful default).
    pub fn new() -> Self {
        let r = Self::default();
        let path = crate::config::paths::sessions_file();
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(p) = serde_json::from_str::<PersistedAllowlist>(&raw) {
                let mut muted = r.muted.write().unwrap_or_else(PoisonError::into_inner);
                muted.extend(p.muted);
            }
        }
        r
    }

    /// Persist the current mute set. Called after every `set_allowed`
    /// change so the file stays in sync with in-memory state. Never
    /// panics — disk failures are logged and swallowed (a stale on-disk
    /// view is fine, just degrades to "default-allow" on next boot).
    fn persist(&self) {
        let muted = self
            .muted
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let payload = PersistedAllowlist { muted };
        let path = crate::config::paths::sessions_file();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&payload) {
            Ok(s) => {
                if let Err(e) = std::fs::write(&path, s) {
                    tracing::warn!("session allowlist persist failed: {e}");
                }
            }
            Err(e) => tracing::warn!("session allowlist serialize failed: {e}"),
        }
    }

    // Lock helpers that recover from poisoned state instead of panicking.
    // The HashMap inside `inner` is internally consistent — a poisoned
    // lock from a panicking writer leaves data we can still read. Falling
    // back to `into_inner()` keeps the registry usable rather than taking
    // the entire app down.
    fn read_map(&self) -> RwLockReadGuard<'_, HashMap<String, Entry>> {
        self.inner.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write_map(&self) -> RwLockWriteGuard<'_, HashMap<String, Entry>> {
        self.inner.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// Record an event from `id`. Returns whether the session is allowed
    /// (i.e. caller should forward the event downstream). Sessions
    /// without an id (synthetic / generic events) are always allowed.
    /// First sighting of a session that's in the persisted mute set
    /// inherits `allowed = false` so user mutes survive restart.
    pub fn observe(&self, id: Option<&str>, cwd: Option<&str>) -> bool {
        let Some(id) = id.filter(|s| !s.is_empty()) else {
            return true;
        };
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let muted_default = self
            .muted
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(id);
        let mut map = self.write_map();
        let e = map.entry(id.to_string()).or_insert(Entry {
            cwd: cwd.map(|s| s.to_string()),
            event_count: 0,
            last_seen: Instant::now(),
            last_seen_ms: now_ms,
            allowed: !muted_default,
        });
        e.event_count += 1;
        e.last_seen = Instant::now();
        e.last_seen_ms = now_ms;
        // Refresh cwd in case the user changed dirs in the same session.
        if let Some(c) = cwd {
            e.cwd = Some(c.to_string());
        }
        e.allowed
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        self.prune();
        let map = self.read_map();
        let mut out: Vec<SessionInfo> = map
            .iter()
            .map(|(id, e)| SessionInfo {
                id: id.clone(),
                label: friendly_label(id, e.cwd.as_deref()),
                cwd: e.cwd.clone(),
                event_count: e.event_count,
                last_seen_ms: e.last_seen_ms,
                allowed: e.allowed,
            })
            .collect();
        // Most-recently-active first so the picker shows live sessions
        // at the top.
        out.sort_by(|a, b| b.last_seen_ms.cmp(&a.last_seen_ms));
        out
    }

    pub fn set_allowed(&self, id: &str, allowed: bool) {
        if let Some(e) = self.write_map().get_mut(id) {
            e.allowed = allowed;
        }
        // Update the persisted mute set + flush to disk. Only explicit
        // mutes are stored (allowlisted = absence from set), keeping the
        // file small even after thousands of sessions.
        {
            let mut muted = self.muted.write().unwrap_or_else(PoisonError::into_inner);
            if allowed {
                muted.remove(id);
            } else {
                muted.insert(id.to_string());
            }
        }
        self.persist();
    }

    /// Drop sessions silent for longer than STALE_AFTER. Cheap O(n) sweep.
    fn prune(&self) {
        self.write_map()
            .retain(|_, e| e.last_seen.elapsed() < STALE_AFTER);
    }
}

/// Pick a short human label: basename of cwd, falling back to the first
/// 8 chars of the session id. Avoids dumping a 36-char UUID into the UI.
fn friendly_label(id: &str, cwd: Option<&str>) -> String {
    if let Some(c) = cwd {
        if let Some(name) = std::path::Path::new(c).file_name() {
            let s = name.to_string_lossy().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    id.chars().take(8).collect()
}
