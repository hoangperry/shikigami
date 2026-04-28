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
pub mod demo;
pub mod event;
pub mod passthrough;
pub mod session;
pub mod state;
pub mod tray;
pub mod tts;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use crate::character::{ActiveCharacter, CharacterRegistry, CharacterSummary};
use crate::config::{Settings, DEFAULT_PORT, PORT_SCAN_SPAN};
use crate::event::{AppState, SpeakEvent};
use crate::session::{SessionInfo, SessionRegistry};
use crate::state::{Dampener, DominantState, IdleTracker};

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
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(registry)
        .setup(|app| {
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                "shikigami.app starting"
            );
            if let Err(e) = tray::install(app.handle()) {
                tracing::warn!("failed to install tray: {e}");
            }
            // Ensure the main window is on-screen and visible. On macOS with
            // transparent + decorations:false the window occasionally opens
            // at 0,0 off-screen or with an opaque default size that the
            // compositor rejects. Center + show + focus is idempotent.
            use tauri::Manager;
            // Smart click-through state (must be `manage`d before any
            // command that takes it as `tauri::State` is invoked).
            let passthrough_state = Arc::new(passthrough::PassthroughState::new());
            app.manage(passthrough_state.clone());
            passthrough::spawn(app.handle().clone(), passthrough_state);

            // Hot-reload watcher for ~/.shikigami/characters/. New /
            // changed character dirs trigger a registry rescan + a
            // `characters:changed` event the frontend listens for.
            if let Some(reg) = app.try_state::<Arc<CharacterRegistry>>() {
                character::watcher::spawn(app.handle().clone(), reg.inner().clone());
            }

            if let Some(w) = app.get_webview_window("main") {
                // Position + size are restored by tauri-plugin-window-state
                // on launch (it remembers wherever the user dragged the
                // window last time). We only enforce a minimum size when
                // the saved state is missing or corrupt — handled by
                // tauri.conf.json `minWidth/minHeight`.
                //
                // Apply persisted runtime knobs (click-through, opacity)
                // from settings.json so they survive across launches.
                apply_runtime_settings(&w, &Settings::load());
                // Log every attached monitor so we can verify the user is
                // looking at the right display.
                if let Ok(mons) = w.available_monitors() {
                    for (i, mon) in mons.iter().enumerate() {
                        tracing::info!(
                            idx = i,
                            name = ?mon.name(),
                            size = ?mon.size(),
                            position = ?mon.position(),
                            scale = mon.scale_factor(),
                            "detected monitor"
                        );
                    }
                }
                let _ = w.show();
                let _ = w.set_focus();

                tracing::info!(
                    visible = ?w.is_visible(),
                    outer_size = ?w.outer_size(),
                    outer_pos = ?w.outer_position(),
                    "main window state after setup"
                );
            } else {
                tracing::warn!("main window missing at setup() — did tauri.conf windows[0] fail?");
            }
            start_event_pipeline(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            get_settings,
            update_settings,
            apply_runtime_settings_cmd,
            list_characters,
            get_active_character,
            set_active_character,
            list_sessions,
            set_session_allowed,
            set_character_bbox
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawns the HTTP event server on the Tokio runtime that Tauri is already
/// managing.
fn start_event_pipeline(app: AppHandle) {
    use tauri::Manager;
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

    let app_for_speak = app.clone();
    let speak_emitter: event::SpeakEmitter = Arc::new(move |speak: SpeakEvent| {
        if let Err(e) = app_for_speak.emit("tts:speak", &speak) {
            tracing::warn!("failed to emit tts:speak: {e}");
        }
    });

    // Build the TTS provider once at boot. None for "none" — endpoint will
    // 503 cleanly. We pre-create the output dir so first call is a no-op.
    let tts: Option<Arc<dyn tts::TtsProvider>> = match tts::build(&settings.tts) {
        Some(p) => {
            if let Err(e) = tts::ensure_output_dir() {
                tracing::warn!("tts output dir setup failed: {e}");
            }
            tracing::info!("tts provider: {}", p.name());
            // Background sweep purges audio files older than 1h every 5min.
            tts::cleanup::spawn_background_sweep();
            Some(Arc::from(p))
        }
        None => {
            tracing::info!("tts disabled (provider=\"{}\")", settings.tts.provider);
            None
        }
    };

    let idle_tracker = Arc::new(IdleTracker::new());
    // Background watcher: when the user goes silent for IDLE_THRESHOLD,
    // emit a synthetic SessionIdleLong → state machine resolves to
    // Sleepy → Hiyori naps. Fired exactly once per quiet stretch; resets
    // on the next real event. Also fires a TTS nudge if a provider is
    // configured — random message from the idle nudge pool, gives the
    // assistant a touch of personality during long silences.
    let emitter_for_idle = emitter.clone();
    let speak_emitter_for_idle = speak_emitter.clone();
    let tts_for_idle = tts.clone();
    state::idle_timer::spawn(
        idle_tracker.clone(),
        Arc::new(move || {
            let resolved = state::idle_timer::synth_idle_state();
            (emitter_for_idle)(resolved);

            // Optional TTS nudge — only when a provider is configured.
            // Fire-and-forget; never block the idle loop on synthesis.
            if let Some(tts) = tts_for_idle.clone() {
                let speak_emit = speak_emitter_for_idle.clone();
                tauri::async_runtime::spawn(async move {
                    let msg = state::idle_timer::pick_idle_message();
                    match tts.synthesize(msg, None).await {
                        Ok(out) => {
                            speak_emit(event::SpeakEvent {
                                audio_url: out.path.to_string_lossy().to_string(),
                                mime: out.mime,
                                provider: out.provider,
                                text: msg.to_string(),
                            });
                        }
                        Err(e) => tracing::warn!("idle nudge TTS failed: {e}"),
                    }
                });
            }
        }),
    );

    // Session registry is shared across the HTTP server (writes on each
    // event) and the Tauri commands (reads + toggles from the modal).
    let sessions = Arc::new(SessionRegistry::new());
    app.manage(sessions.clone());

    let app_state = Arc::new(AppState {
        token,
        dampener: Mutex::new(Dampener::new(2000)),
        emitter,
        speak_emitter,
        tts,
        idle_tracker,
        sessions,
        last_announced: Mutex::new(None::<DominantState>),
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

/// Apply settings that affect the live OS window. With `click_through`,
/// activate the smart-passthrough polling loop (the polling task does
/// the actual `set_ignore_cursor_events` toggling per cursor position).
/// With it off, force the window clickable.
fn apply_runtime_settings<R: tauri::Runtime>(w: &tauri::WebviewWindow<R>, s: &Settings) {
    use tauri::Manager;
    let app = w.app_handle();
    if let Some(ps) = app.try_state::<Arc<passthrough::PassthroughState>>() {
        let ps_arc: Arc<passthrough::PassthroughState> = ps.inner().clone();
        let on = s.click_through;
        tauri::async_runtime::spawn(async move {
            ps_arc.set_active(on).await;
        });
        if !on {
            if let Err(e) = w.set_ignore_cursor_events(false) {
                tracing::warn!("set_ignore_cursor_events(false) failed: {e}");
            }
        }
    } else if let Err(e) = w.set_ignore_cursor_events(s.click_through) {
        // Fallback if passthrough state hasn't been managed yet (cold setup).
        tracing::warn!("set_ignore_cursor_events failed: {e}");
    }
}

#[tauri::command]
fn set_character_bbox(
    bbox: passthrough::CharacterBBox,
    state: tauri::State<'_, Arc<passthrough::PassthroughState>>,
) {
    let s = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        s.set_bbox(bbox).await;
    });
}

#[tauri::command]
fn ping() -> &'static str {
    "pong"
}

#[tauri::command]
fn apply_runtime_settings_cmd(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let Some(w) = app.get_webview_window("main") else {
        return Err("main window missing".into());
    };
    apply_runtime_settings(&w, &Settings::load());
    Ok(())
}

#[tauri::command]
fn get_settings(_app: AppHandle) -> Settings {
    Settings::load()
}

#[tauri::command]
fn update_settings(app: AppHandle, patch: serde_json::Value) -> Result<Settings, String> {
    let mut current = Settings::load();
    let mut merged =
        serde_json::to_value(&current).map_err(|e| format!("serialize failed: {e}"))?;
    if let (Some(m), Some(p)) = (merged.as_object_mut(), patch.as_object()) {
        for (k, v) in p {
            m.insert(k.clone(), v.clone());
        }
    }
    current = serde_json::from_value(merged).map_err(|e| format!("merge failed: {e}"))?;
    current.save().map_err(|e| format!("persist failed: {e}"))?;
    // Notify the frontend so renderer-applied knobs (scale, opacity)
    // take effect without a restart.
    let _ = app.emit("settings_changed", &current);
    Ok(current)
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
    if !registry.set_active(&id) {
        return Err(format!("character {id:?} not found"));
    }
    // Persist so the choice survives a restart. The in-memory flag
    // alone gets reset by `Settings::load()` on the next launch.
    let mut s = Settings::load();
    s.active_character = Some(id);
    s.save().map_err(|e| format!("persist failed: {e}"))?;
    Ok(())
}

#[tauri::command]
fn list_sessions(sessions: tauri::State<'_, Arc<SessionRegistry>>) -> Vec<SessionInfo> {
    sessions.list()
}

#[tauri::command]
fn set_session_allowed(
    id: String,
    allowed: bool,
    sessions: tauri::State<'_, Arc<SessionRegistry>>,
) {
    sessions.set_allowed(&id, allowed);
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_env("SHIKIGAMI_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
