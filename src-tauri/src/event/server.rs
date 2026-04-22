//! HTTP event ingress — `POST /v1/events` on 127.0.0.1 with bearer auth.
//!
//! See ADR-001 and TDD §4.

use super::schema::EventPayload;
use crate::state::{Dampener, ResolvedState};
use axum::{
    extract::State,
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

pub struct AppState {
    pub token: String,
    pub dampener: Mutex<Dampener>,
    pub emitter: StateEmitter,
}

/// Find the first bindable port in the range `preferred..preferred+span` and
/// start serving on it. Returns the bound port.
pub async fn serve(state: Arc<AppState>, preferred_port: u16, span: u16) -> anyhow::Result<u16> {
    let (listener, port) = bind_with_recovery(preferred_port, span).await?;
    let app = Router::new()
        .route("/v1/events", post(ingest_event))
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
    (app.emitter)(resolved);

    StatusCode::ACCEPTED.into_response()
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
