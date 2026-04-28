//! Session allowlist persistence across registry restarts.
//!
//! Simulates the user-mute → quit → relaunch cycle: a SessionRegistry
//! that mutes a session id should write `sessions.json`, and a fresh
//! registry constructed from the same `~/.shikigami/` should observe
//! that id as `allowed = false` on first sighting.

use shikigami::session::SessionRegistry;
use std::sync::OnceLock;

/// Each integration test gets its own SHIKIGAMI_HOME so the persisted
/// `sessions.json` from one test doesn't bleed into another. tempfile
/// crate isn't pulled in just for this — `std::env::temp_dir()` + a
/// unique suffix is enough.
fn isolated_home(suffix: &str) -> std::path::PathBuf {
    static SEQ: OnceLock<std::sync::Mutex<u64>> = OnceLock::new();
    let mut s = SEQ.get_or_init(|| std::sync::Mutex::new(0)).lock().unwrap();
    *s += 1;
    let dir = std::env::temp_dir().join(format!(
        "shikigami-test-{}-{}-{}",
        std::process::id(),
        *s,
        suffix
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Tests share process-wide env state, so we serialize them through a
/// global mutex. `unwrap_or_else(PoisonError::into_inner)` keeps a
/// poisoned lock from cascading a single panic into a test-suite-wide
/// failure cluster.
fn with_home<F: FnOnce()>(home: &std::path::Path, body: F) {
    use std::sync::PoisonError;
    static GUARD: OnceLock<std::sync::Mutex<()>> = OnceLock::new();
    let _g = GUARD
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let prev = std::env::var_os("SHIKIGAMI_HOME");
    std::env::set_var("SHIKIGAMI_HOME", home);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    // Always restore env even if the body panicked, so subsequent tests
    // don't observe a stale override.
    match prev {
        Some(v) => std::env::set_var("SHIKIGAMI_HOME", v),
        None => std::env::remove_var("SHIKIGAMI_HOME"),
    }
    if let Err(p) = result {
        std::panic::resume_unwind(p);
    }
}

#[test]
fn unmuted_session_is_allowed_by_default() {
    let home = isolated_home("default");
    with_home(&home, || {
        let r = SessionRegistry::new();
        assert!(
            r.observe(Some("alpha"), Some("/repo")),
            "first sighting allowed"
        );
    });
}

#[test]
fn mute_persists_across_registry_restart() {
    let home = isolated_home("persist");
    with_home(&home, || {
        // Run 1: observe + mute.
        {
            let r = SessionRegistry::new();
            assert!(r.observe(Some("beta"), Some("/repo")), "initial allowed");
            r.set_allowed("beta", false);
        }
        // sessions.json should exist on disk now.
        let session_file = home.join("sessions.json");
        assert!(
            session_file.is_file(),
            "expected sessions.json after mute, got nothing at {}",
            session_file.display()
        );
        // Run 2: fresh registry — first observation must inherit the mute.
        {
            let r2 = SessionRegistry::new();
            let allowed = r2.observe(Some("beta"), Some("/repo"));
            assert!(!allowed, "muted session should stay muted after restart");
        }
    });
}

#[test]
fn unmute_removes_session_from_persisted_file() {
    let home = isolated_home("unmute");
    with_home(&home, || {
        let r = SessionRegistry::new();
        r.observe(Some("gamma"), None);
        r.set_allowed("gamma", false); // muted on disk
        r.set_allowed("gamma", true); // unmuted — should remove from file

        let json = std::fs::read_to_string(home.join("sessions.json")).unwrap();
        assert!(
            !json.contains("gamma"),
            "gamma should be absent from persisted file after unmute, got: {json}"
        );
    });
}

#[test]
fn id_less_events_always_allowed_regardless_of_mutes() {
    let home = isolated_home("idless");
    with_home(&home, || {
        let r = SessionRegistry::new();
        // No id → can't be muted, always allowed (synthetic events path).
        assert!(r.observe(None, None));
        assert!(r.observe(Some(""), None), "empty id treated as no id");
    });
}

#[test]
fn malformed_sessions_file_falls_back_to_empty_set() {
    let home = isolated_home("malformed");
    with_home(&home, || {
        // Write garbage where the registry expects JSON.
        std::fs::write(home.join("sessions.json"), b"not valid json {{").unwrap();
        let r = SessionRegistry::new();
        // Default-allow when the file fails to parse.
        assert!(r.observe(Some("delta"), None));
    });
}
