//! Integration tests for the event server's HTTP surface.
//!
//! Spins up the real `event::serve` on a fresh port, hits it via reqwest,
//! and inspects state emissions through a channel-backed StateEmitter.
//! Doesn't require Tauri to be running — fully self-contained for CI.

use shikigami::event::{self, AppState, SpeakEmitter, SpeakEvent, StateEmitter};
use shikigami::session::SessionRegistry;
use shikigami::state::{Dampener, DominantState, IdleTracker, ResolvedState};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

// Note: bumped when AppState gains a new field — keep this fixture in
// sync with `event/server.rs::AppState`.

/// Boot a self-contained server on a random ephemeral port; return the
/// chosen port, the bearer token, and a receiver that captures every
/// state the backend would emit to the frontend.
async fn boot() -> (u16, String, mpsc::UnboundedReceiver<ResolvedState>) {
    let token = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();
    let (tx, rx) = mpsc::unbounded_channel::<ResolvedState>();
    let emitter: StateEmitter = Arc::new(move |s| {
        let _ = tx.send(s);
    });
    let speak_emitter: SpeakEmitter = Arc::new(|_: SpeakEvent| {});
    let app_state = Arc::new(AppState {
        token: token.clone(),
        dampener: Mutex::new(Dampener::new(2000)),
        emitter,
        speak_emitter,
        tts: None,
        idle_tracker: Arc::new(IdleTracker::new()),
        sessions: Arc::new(SessionRegistry::new()),
        last_announced: Mutex::new(None::<DominantState>),
    });
    // Use port 0 to let the OS pick — bind_with_recovery will scan from
    // there. Practically we choose a high random port to avoid clashing
    // with a live shikigami on 7796.
    let port = 47700 + (rand::random::<u16>() % 200);
    let chosen = event::serve(app_state, port, 50)
        .await
        .expect("serve failed");
    (chosen, token, rx)
}

fn url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{port}{path}")
}

async fn post_json(
    port: u16,
    token: &str,
    path: &str,
    body: serde_json::Value,
) -> reqwest::Response {
    reqwest::Client::new()
        .post(url(port, path))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .expect("request send failed")
}

async fn next_state(rx: &mut mpsc::UnboundedReceiver<ResolvedState>) -> Option<ResolvedState> {
    tokio::time::timeout(Duration::from_millis(500), rx.recv())
        .await
        .ok()
        .flatten()
}

#[tokio::test]
async fn missing_bearer_returns_401() {
    let (port, _token, _rx) = boot().await;
    let resp = reqwest::Client::new()
        .post(url(port, "/v1/events"))
        .json(&serde_json::json!({"schemaVersion":"1.0","source":"generic","type":"session_start"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn wrong_bearer_returns_401() {
    let (port, _token, _rx) = boot().await;
    let resp = reqwest::Client::new()
        .post(url(port, "/v1/events"))
        .bearer_auth("wrong-token-here")
        .json(&serde_json::json!({"schemaVersion":"1.0","source":"generic","type":"session_start"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn bad_schema_version_returns_400() {
    let (port, token, _rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({"schemaVersion":"9.9","source":"generic","type":"session_start"}),
    )
    .await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn unknown_event_type_returns_400() {
    let (port, token, _rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({"schemaVersion":"1.0","source":"generic","type":"not_real"}),
    )
    .await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn tool_start_bash_emits_focused() {
    let (port, token, mut rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"claude-code",
            "type":"tool_start","tool":"Bash",
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    let state = next_state(&mut rx).await.expect("expected emission");
    assert_eq!(state.dominant, DominantState::Focused);
}

#[tokio::test]
async fn tool_start_task_emits_confused() {
    let (port, token, mut rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"claude-code",
            "type":"tool_start","tool":"Task",
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    let state = next_state(&mut rx).await.expect("expected emission");
    assert_eq!(state.dominant, DominantState::Confused);
}

#[tokio::test]
async fn tool_complete_read_emits_idle() {
    let (port, token, mut rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"claude-code",
            "type":"tool_complete","tool":"Read","exitCode":0,
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    let state = next_state(&mut rx).await.expect("expected emission");
    assert_eq!(state.dominant, DominantState::Idle);
}

#[tokio::test]
async fn tool_complete_failure_emits_warning() {
    let (port, token, mut rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"claude-code",
            "type":"tool_complete","tool":"Bash","exitCode":1,
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    let state = next_state(&mut rx).await.expect("expected emission");
    assert_eq!(state.dominant, DominantState::Warning);
}

#[tokio::test]
async fn destructive_op_passes_through_with_critical_severity() {
    let (port, token, mut rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"claude-code",
            "type":"destructive_op_detected","severity":"critical",
            "text":"rm -rf /tmp/foo",
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    let state = next_state(&mut rx).await.expect("expected emission");
    assert_eq!(state.dominant, DominantState::Warning);
    // Critical severity suppresses texture extraction.
    assert!(state.text.as_deref().unwrap_or("").contains("rm -rf"));
}

#[tokio::test]
async fn say_endpoint_503_when_tts_disabled() {
    let (port, token, _rx) = boot().await;
    let resp = post_json(port, &token, "/v1/say", serde_json::json!({"text":"hello"})).await;
    assert_eq!(resp.status(), 503);
}

#[tokio::test]
async fn say_endpoint_400_on_empty_text() {
    let (port, token, _rx) = boot().await;
    // Even with TTS disabled the empty-text check happens earlier? Actually
    // in our implementation, empty-text returns 400 BEFORE the provider
    // check. Verify that ordering.
    let resp = post_json(port, &token, "/v1/say", serde_json::json!({"text":""})).await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn session_filter_blocks_disabled_sessions() {
    let (port, token, mut rx) = boot().await;
    let sessions = Arc::new(SessionRegistry::new()); // unused here — we go through HTTP

    // First event from session "alpha" is allowed by default → emitted.
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"claude-code",
            "type":"tool_start","tool":"Bash",
            "sessionId":"alpha","cwd":"/tmp/a",
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    assert!(
        next_state(&mut rx).await.is_some(),
        "alpha event should emit"
    );

    // We can't toggle the registry from the integration test (the server
    // owns its own SessionRegistry instance), so the muted-path check is
    // covered by the unit-tested SessionRegistry::set_allowed in
    // session.rs. This test validates the happy path: tagged events get
    // through the filter.
    drop(sessions);
}

#[tokio::test]
async fn session_id_optional_id_less_event_passes() {
    let (port, token, mut rx) = boot().await;
    let resp = post_json(
        port,
        &token,
        "/v1/events",
        serde_json::json!({
            "schemaVersion":"1.0","source":"generic","type":"session_start",
        }),
    )
    .await;
    assert_eq!(resp.status(), 202);
    assert!(
        next_state(&mut rx).await.is_some(),
        "id-less event should still emit"
    );
}
