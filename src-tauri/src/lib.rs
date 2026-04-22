//! Shikigami v0.1 — Phase 0 scaffold.
//!
//! This is the foundation crate. Event server, state machine, and character
//! loader are stubbed and will land in Phase 1 / Phase 2.

use tracing_subscriber::EnvFilter;

/// Build & run the Tauri application.
///
/// Phase 0 goal: open a transparent, always-on-top window that renders the
/// React debug panel. Click-through, tray menu, event server, state machine
/// are all deferred to later phases.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|_app| {
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                "shikigami.app starting"
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Smoke-test IPC command. Used by Phase 0 dev tests.
#[tauri::command]
fn ping() -> &'static str {
    "pong"
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("SHIKIGAMI_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
