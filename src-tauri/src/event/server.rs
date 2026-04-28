//! HTTP event ingress — `POST /v1/events` on 127.0.0.1 with bearer auth.
//!
//! See ADR-001 and TDD §4.

use super::schema::{EventPayload, EventType, SayRequest, SpeakEvent};
use crate::config::Settings;
use crate::session::SessionRegistry;
use crate::state::{announcements, Dampener, DominantState, IdleTracker, ResolvedState, Severity};
use crate::tts::TtsProvider;
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub type StateEmitter = Arc<dyn Fn(ResolvedState) + Send + Sync>;
pub type SpeakEmitter = Arc<dyn Fn(SpeakEvent) + Send + Sync>;

pub struct AppState {
    pub token: String,
    pub dampener: Mutex<Dampener>,
    pub emitter: StateEmitter,
    pub speak_emitter: SpeakEmitter,
    /// Active TTS provider. None when `tts.provider = "none"`.
    pub tts: Option<Arc<dyn TtsProvider>>,
    /// Tracks last-event timestamp for the synthetic sleepy timer.
    pub idle_tracker: Arc<IdleTracker>,
    /// Per-session allowlist + activity log (for the multi-tab picker).
    pub sessions: Arc<SessionRegistry>,
    /// Last announced dominant state — used to debounce voice
    /// announcements so Hiyori doesn't repeat the same line on every
    /// event of the same type.
    pub last_announced: Mutex<Option<DominantState>>,
}

/// Find the first bindable port in the range `preferred..preferred+span` and
/// start serving on it. Returns the bound port.
pub async fn serve(state: Arc<AppState>, preferred_port: u16, span: u16) -> anyhow::Result<u16> {
    let (listener, port) = bind_with_recovery(preferred_port, span).await?;
    // 64 KiB body cap — comfortably above any legitimate event or TTS
    // text payload (we cap text at 2000 chars elsewhere) but blocks
    // accidental / malicious upload attempts. Defense-in-depth; the
    // listener is loopback-only, but never trust the wire.
    let app = Router::new()
        .route("/v1/events", post(ingest_event))
        .route("/v1/say", post(ingest_say))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(state);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("event server error: {e}");
        }
    });
    Ok(port)
}

async fn bind_with_recovery(preferred: u16, span: u16) -> anyhow::Result<(TcpListener, u16)> {
    let span = span.max(1);
    for offset in 0..span {
        let port = preferred.saturating_add(offset);
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                tracing::info!("event server bound to 127.0.0.1:{port}");
                return Ok((listener, port));
            }
            Err(e) => {
                tracing::warn!("bind on {port} failed: {e}");
            }
        }
    }
    Err(anyhow::anyhow!(
        "no free port found in {preferred}..{}",
        preferred.saturating_add(span)
    ))
}

async fn ingest_event(
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Result<Json<EventPayload>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    if !authenticated(&app.token, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"invalid bearer token"})),
        )
            .into_response();
    }

    let Json(event) = match body {
        Ok(j) => j,
        Err(rej) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error":"invalid JSON payload","details":rej.to_string()})),
            )
                .into_response();
        }
    };

    if let Err(msg) = event.validate_version() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response();
    }

    // Session filter — record the session into the registry and drop the
    // event entirely if the user has muted that tab in Preferences.
    // Synthetic / id-less events bypass the filter (always allowed).
    let allowed = app
        .sessions
        .observe(event.session_id.as_deref(), event.cwd.as_deref());
    if !allowed {
        tracing::trace!(
            session = ?event.session_id,
            "event dropped — session muted in preferences"
        );
        return StatusCode::ACCEPTED.into_response();
    }

    let now = std::time::Instant::now();
    let mut dampener = app.dampener.lock().await;
    let should_pass = dampener.observe(event.event_type, event.severity_or_default(), now);
    drop(dampener);

    if !should_pass {
        tracing::trace!(
            event_type = ?event.event_type,
            severity = ?event.severity_or_default(),
            "event dampened (duplicate within window)"
        );
        return StatusCode::ACCEPTED.into_response();
    }

    let resolved = crate::state::resolve(&event);
    tracing::info!(
        dominant = ?resolved.dominant,
        texture = ?resolved.texture,
        severity = ?resolved.severity,
        animation_key = %resolved.animation_key(),
        "state resolved"
    );
    let dominant = resolved.dominant;
    let severity = resolved.severity;
    let event_type = event.event_type;
    (app.emitter)(resolved);

    // Reset the idle timer — any real event indicates the user / agent is
    // active again, so cancel any pending nap.
    app.idle_tracker.touch().await;

    // Optional: voice announcements on state transitions. Reads settings
    // fresh each time (cheap file read) so the toggle takes effect
    // without a restart.
    maybe_announce(app.clone(), dominant, severity, event_type).await;

    StatusCode::ACCEPTED.into_response()
}

/// Fire a short TTS announcement when the dominant state changes (or when
/// a critical destructive op fires). Skipped when:
///   - `tts.provider = "none"` or `tts.announce_events = false`
///   - The new dominant matches the previous announcement (debounce)
///   - The state is one we deliberately keep silent (idle/sleepy/etc)
async fn maybe_announce(
    app: Arc<AppState>,
    dominant: DominantState,
    severity: Severity,
    event_type: EventType,
) {
    let Some(tts) = app.tts.clone() else { return };
    let cfg = Settings::load().tts;
    if !cfg.announce_events {
        return;
    }

    // Critical destructive ops bypass the dominant-change debounce —
    // every one is announce-worthy.
    let phrase: Option<String> =
        if event_type == EventType::DestructiveOpDetected && severity == Severity::Critical {
            Some(announcements::critical_destructive_phrase().to_string())
        } else {
            // Debounce: skip when the dominant hasn't changed since the
            // last announcement.
            let mut last = app.last_announced.lock().await;
            if *last == Some(dominant) {
                return;
            }
            *last = Some(dominant);
            announcements::phrase_for(dominant).map(|s| s.to_string())
        };

    let Some(text) = phrase else { return };

    let speak_emitter = app.speak_emitter.clone();
    tauri::async_runtime::spawn(async move {
        match tts.synthesize(&text, None).await {
            Ok(out) => {
                speak_emitter(SpeakEvent {
                    audio_url: out.path.to_string_lossy().to_string(),
                    mime: out.mime,
                    provider: out.provider,
                    text,
                });
            }
            Err(e) => tracing::warn!("announcement TTS failed: {e}"),
        }
    });
}

async fn ingest_say(
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Result<Json<SayRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    if !authenticated(&app.token, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"invalid bearer token"})),
        )
            .into_response();
    }

    let Json(req) = match body {
        Ok(j) => j,
        Err(rej) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error":"invalid JSON","details":rej.to_string()})),
            )
                .into_response();
        }
    };

    if req.text.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":"text is empty"})),
        )
            .into_response();
    }
    // Cap spoken text length. Cloud providers bill per-character (so a
    // leaked token + giant payload = unbounded cost) and `say-macos`
    // passes the text as a CLI arg subject to ARG_MAX. 2000 chars is
    // ~30s of speech at typical rate — well past any reasonable nudge.
    if req.text.chars().count() > 2000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error":"text exceeds 2000-character cap"})),
        )
            .into_response();
    }

    let Some(tts) = app.tts.clone() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error":"tts disabled (set tts.provider in config)"})),
        )
            .into_response();
    };

    match tts.synthesize(&req.text, req.voice.as_deref()).await {
        Ok(out) => {
            // Frontend receives the absolute path; it converts to a Tauri
            // asset URL via `convertFileSrc`. Keeping path serialisation
            // platform-native avoids URL-escape mistakes in the bridge.
            let audio_url = out.path.to_string_lossy().to_string();
            (app.speak_emitter)(SpeakEvent {
                audio_url: audio_url.clone(),
                mime: out.mime,
                provider: out.provider,
                text: req.text.chars().take(160).collect(),
            });
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "audio_url": audio_url,
                    "mime": out.mime,
                    "provider": out.provider,
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::warn!("tts synthesize failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error":format!("tts failed: {e}")})),
            )
                .into_response()
        }
    }
}

fn authenticated(expected_token: &str, headers: &HeaderMap) -> bool {
    let Some(h) = headers.get(AUTHORIZATION) else {
        return false;
    };
    let Ok(s) = h.to_str() else {
        return false;
    };
    let provided = match s.strip_prefix("Bearer ") {
        Some(t) => t.trim(),
        None => return false,
    };
    super::auth::verify(expected_token, provided)
}
