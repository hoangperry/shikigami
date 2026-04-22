//! System tray menu. Owns Show/Hide, Reset Position, Toggle Diag, Quit.
//!
//! The tray lives for the lifetime of the app. Menu events are routed to
//! small helpers that either manipulate the main window directly or emit
//! a Tauri event to the frontend (for user-facing UI toggles).

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, Runtime,
};

pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Show / Hide", true, None::<&str>)?;
    let reset_pos = MenuItem::with_id(app, "reset_position", "Reset Position", true, None::<&str>)?;
    let toggle_diag = MenuItem::with_id(
        app,
        "toggle_diag",
        "Toggle Diagnostics (⌘I)",
        true,
        None::<&str>,
    )?;
    let open_settings =
        MenuItem::with_id(app, "open_settings", "Preferences…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let about = PredefinedMenuItem::about(
        app,
        Some("About Shikigami"),
        Some(tauri::menu::AboutMetadata {
            name: Some("Shikigami".into()),
            version: Some(env!("CARGO_PKG_VERSION").into()),
            short_version: Some("0.1.0-alpha".into()),
            ..Default::default()
        }),
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit Shikigami", true, Some("CmdOrCtrl+Q"))?;

    let menu = Menu::with_items(
        app,
        &[
            &show_hide,
            &reset_pos,
            &toggle_diag,
            &separator,
            &open_settings,
            &about,
            &separator,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("shikigami-tray")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            // Fall back to a 1x1 transparent pixel if no icon is configured
            // at build time. Prevents a panic on first boot in dev.
            tauri::image::Image::new_owned(vec![0, 0, 0, 0], 1, 1)
        }))
        .menu(&menu)
        .tooltip("Shikigami")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_hide" => toggle_main_window_visibility(app),
            "reset_position" => reset_main_window_position(app),
            "toggle_diag" => {
                // Frontend owns the diag overlay; ask it to toggle.
                let _ = app.emit("tray:toggle_diag", ());
            }
            "open_settings" => {
                let _ = app.emit("tray:open_settings", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn toggle_main_window_visibility<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        // "Show" is cheap and idempotent; we always ensure the window is
        // on-screen + focused so a lost or off-screen window can be
        // recovered by clicking the tray menu. Toggle hide only when the
        // user explicitly wants to dismiss while it is in front.
        let was_focused = w.is_focused().unwrap_or(false);
        if was_focused && w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.unminimize();
            let _ = w.center();
            let _ = w.set_focus();
        }
    }
}

fn reset_main_window_position<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    // Center on whichever monitor currently holds the cursor (best-effort).
    if let Err(e) = w.center() {
        tracing::warn!("tray: center failed: {e}");
    }
    let _ = w.show();
    let _ = w.set_focus();
    let _ = app.emit("tray:position_reset", ());
}
