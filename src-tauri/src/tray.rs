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

use crate::config::Settings;

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
    // Escape hatch: when click-through is on the user can't click the
    // window to open Preferences, so the toggle MUST be reachable from
    // the tray. Otherwise enabling click-through is a one-way trip into
    // editing config.json by hand.
    let click_through_item = MenuItem::with_id(
        app,
        "toggle_click_through",
        "Toggle Click-Through",
        true,
        None::<&str>,
    )?;
    let run_demo = MenuItem::with_id(app, "run_demo", "Run Demo", true, None::<&str>)?;
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
            &click_through_item,
            &run_demo,
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
            "toggle_click_through" => toggle_click_through(app),
            "run_demo" => crate::demo::spawn(app.clone()),
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

/// Flip `click_through` in the persisted settings + apply to the live
/// window in one go. This is the only way to recover from an accidental
/// click-through enable without editing config.json manually.
///
/// Must update `PassthroughState.active` too — the 60Hz polling loop
/// reads `active` to decide whether to engage smart hit-testing. Without
/// this, the tray flip would be clobbered by the next poll tick.
fn toggle_click_through<R: Runtime>(app: &AppHandle<R>) {
    use std::sync::Arc;
    let mut s = Settings::load();
    s.click_through = !s.click_through;
    if let Err(e) = s.save() {
        tracing::warn!("toggle_click_through save failed: {e}");
        return;
    }
    let on = s.click_through;
    if let Some(ps) = app.try_state::<Arc<crate::passthrough::PassthroughState>>() {
        let ps_arc: Arc<crate::passthrough::PassthroughState> = ps.inner().clone();
        tauri::async_runtime::spawn(async move {
            ps_arc.set_active(on).await;
        });
    }
    if !on {
        if let Some(w) = app.get_webview_window("main") {
            if let Err(e) = w.set_ignore_cursor_events(false) {
                tracing::warn!("set_ignore_cursor_events failed: {e}");
            }
        }
    }
    tracing::info!("click_through toggled → {on}");
}

fn reset_main_window_position<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    // Manually center on primary monitor; w.center() is unreliable on macOS
    // with transparent + multi-display configurations.
    if let Ok(Some(mon)) = w.primary_monitor() {
        let monsize = mon.size();
        let scale = mon.scale_factor();
        let mon_w = monsize.width as f64 / scale;
        let mon_h = monsize.height as f64 / scale;
        if let Ok(outer) = w.outer_size() {
            let win_w = outer.width as f64 / scale;
            let win_h = outer.height as f64 / scale;
            let x = ((mon_w - win_w) / 2.0).max(0.0);
            let y = ((mon_h - win_h) / 2.0).max(0.0);
            let _ = w.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }
    } else {
        let _ = w.center();
    }
    let _ = w.show();
    let _ = w.set_focus();
    let _ = app.emit("tray:position_reset", ());
}
