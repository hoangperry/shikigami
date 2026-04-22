//! Shikigami v0.1 — Phase 1 complete: event transport + state machine wired.
//!
//! The Tauri app:
//!   1. Loads / creates the bearer token
//!   2. Starts the local HTTP event server on 127.0.0.1 (with port recovery)
//!   3. On each valid event, resolves it into a `ResolvedState` and emits
//!      a `state_changed` Tauri event to the frontend
//!
//! Phase 2 will load `.shikigami` character packages and replace the debug
//! panel with the PixiJS sprite renderer.

pub mod character;
pub mod config;
pub mod event;
pub mod state;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use crate::character::{ActiveCharacter, CharacterRegistry, CharacterSummary};
use crate::config::{Settings, DEFAULT_PORT, PORT_SCAN_SPAN};
use crate::event::AppState;
use crate::state::Dampener;

/// Build & run the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let registry = Arc::new(CharacterRegistry::new());
    let report = registry.load_from_default_paths();
    tracing::info!(
        loaded = ?report.loaded,
        failed_count = report.failed.len(),
        "character registry initialized"
    );
    for (path, err) in &report.failed {
        tracing::warn!(path = %path.display(), error = %err, "character failed to load");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(registry)
        .setup(|app| {
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                "shikigami.app starting"
            );
            start_event_pipeline(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            get_settings,
            list_characters,
            get_active_character,
            set_active_character
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawns the HTTP event server on the Tokio runtime that Tauri is already
/// managing.
fn start_event_pipeline(app: AppHandle) {
    // Load persistent settings; fall back to defaults on missing / invalid.
    let mut settings = Settings::load();

    // Ensure bearer token exists.
    let token_path = config::paths::token_file();
    let token = match event::auth::load_or_create_token(&token_path) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("cannot read/create token at {}: {e}", token_path.display());
            return;
        }
    };

    let app_for_emitter = app.clone();
    let emitter: event::StateEmitter = Arc::new(move |resolved| {
        if let Err(e) = app_for_emitter.emit("state_changed", &resolved) {
            tracing::warn!("failed to emit state_changed: {e}");
        }
    });

    let app_state = Arc::new(AppState {
        token,
        dampener: Mutex::new(Dampener::new(2000)),
        emitter,
    });

    // Choose a preferred port: existing config > default.
    let preferred = if settings.port == 0 {
        DEFAULT_PORT
    } else {
        settings.port
    };

    tauri::async_runtime::spawn(async move {
        match event::serve(app_state, preferred, PORT_SCAN_SPAN).await {
            Ok(bound) => {
                tracing::info!("event server listening on 127.0.0.1:{bound}");
                if settings.port != bound {
                    settings.port = bound;
                    if let Err(e) = settings.save() {
                        tracing::warn!("failed to persist chosen port: {e}");
                    }
                }
            }
            Err(e) => {
                tracing::error!("failed to start event server: {e}");
            }
        }
    });
}

#[tauri::command]
fn ping() -> &'static str {
    "pong"
}

#[tauri::command]
fn get_settings(_app: AppHandle) -> Settings {
    Settings::load()
}

#[tauri::command]
fn list_characters(registry: tauri::State<'_, Arc<CharacterRegistry>>) -> Vec<CharacterSummary> {
    registry.list_summaries()
}

#[tauri::command]
fn get_active_character(
    registry: tauri::State<'_, Arc<CharacterRegistry>>,
) -> Option<ActiveCharacter> {
    registry.active_character()
}

#[tauri::command]
fn set_active_character(
    id: String,
    registry: tauri::State<'_, Arc<CharacterRegistry>>,
) -> Result<(), String> {
    if registry.set_active(&id) {
        Ok(())
    } else {
        Err(format!("character {id:?} not found"))
    }
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_env("SHIKIGAMI_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
