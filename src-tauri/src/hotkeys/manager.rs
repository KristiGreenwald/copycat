use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::clipboard::manager::SharedClipboardManager;
use crate::clipboard::persistence;
use crate::settings::config::SharedConfig;

pub fn register_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let config = app.state::<SharedConfig>();
    let config_guard = config.lock().unwrap();
    let shortcuts_config = config_guard.shortcuts.clone();
    drop(config_guard);

    let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

    for (i, key) in keys.iter().enumerate() {
        // Register copy shortcut
        let copy_shortcut_str = format!("{}+{}", shortcuts_config.copy_modifier, key);
        eprintln!("[ClipX] Registering copy shortcut: {}", copy_shortcut_str);
        let copy_shortcut: Shortcut = copy_shortcut_str.parse().map_err(|e| {
            format!("Failed to parse copy shortcut '{}': {:?}", copy_shortcut_str, e)
        })?;
        let slot_index = i;
        let app_copy = app.clone();
        app.global_shortcut().on_shortcut(copy_shortcut, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
            if event.state == ShortcutState::Pressed {
                eprintln!("[ClipX] Copy shortcut pressed for slot {}", slot_index);
                handle_copy(&app_copy, slot_index);
            }
        })?;

        // Register paste shortcut
        let paste_shortcut_str = format!("{}+{}", shortcuts_config.paste_modifier, key);
        eprintln!("[ClipX] Registering paste shortcut: {}", paste_shortcut_str);
        let paste_shortcut: Shortcut = paste_shortcut_str.parse().map_err(|e| {
            format!("Failed to parse paste shortcut '{}': {:?}", paste_shortcut_str, e)
        })?;
        let app_paste = app.clone();
        app.global_shortcut().on_shortcut(paste_shortcut, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
            if event.state == ShortcutState::Pressed {
                eprintln!("[ClipX] Paste shortcut pressed for slot {}", slot_index);
                handle_paste(&app_paste, slot_index);
            }
        })?;
    }

    // Register HUD toggle shortcut
    eprintln!("[ClipX] Registering HUD toggle: {}", shortcuts_config.toggle_hud);
    let toggle_hud: Shortcut = shortcuts_config
        .toggle_hud
        .parse()
        .map_err(|e| format!("Failed to parse HUD toggle shortcut: {:?}", e))?;
    let app_hud = app.clone();
    app.global_shortcut().on_shortcut(toggle_hud, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
        if event.state == ShortcutState::Pressed {
            eprintln!("[ClipX] HUD toggle pressed");
            show_hud_window(&app_hud);
            let _ = app_hud.get_webview_window("main").map(|w| w.emit("toggle-hud", ()));
        }
    })?;

    // Register clear-all shortcut
    eprintln!("[ClipX] Registering clear-all: {}", shortcuts_config.clear_all);
    let clear_all: Shortcut = shortcuts_config
        .clear_all
        .parse()
        .map_err(|e| format!("Failed to parse clear-all shortcut: {:?}", e))?;
    let app_clear = app.clone();
    app.global_shortcut().on_shortcut(clear_all, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
        if event.state == ShortcutState::Pressed {
            eprintln!("[ClipX] Clear-all pressed");
            let state = app_clear.state::<SharedClipboardManager>();
            let mut mgr = state.lock().unwrap();
            mgr.clear_all();
            persistence::save_slots(mgr.slots_for_persistence()).ok();
            drop(mgr);
            if let Some(window) = app_clear.get_webview_window("main") {
                let _ = window.emit("slots-updated", ());
            }
        }
    })?;

    eprintln!("[ClipX] All shortcuts registered successfully");
    Ok(())
}

pub fn show_hud_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // Position window at bottom-right of primary monitor
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let screen_size = monitor.size();
            let screen_pos = monitor.position();
            let scale = monitor.scale_factor();
            let win_w = (320.0 * scale) as i32;
            let win_h = (400.0 * scale) as i32;
            let margin = (16.0 * scale) as i32;
            let x = screen_pos.x + screen_size.width as i32 - win_w - margin;
            let y = screen_pos.y + screen_size.height as i32 - win_h - margin;
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
        window.show().ok();
        window.set_focus().ok();
        eprintln!("[ClipX] HUD window shown");
    } else {
        eprintln!("[ClipX] WARNING: main window not found");
    }
}

pub fn hide_hud_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().ok();
    }
}

fn handle_copy(app: &AppHandle, slot_index: usize) {
    let state = app.state::<SharedClipboardManager>();
    let mut mgr = state.lock().unwrap();

    match mgr.copy_to_slot(slot_index) {
        Ok(slot_info) => {
            eprintln!("[ClipX] Copied to slot {}: '{}'", slot_index, slot_info.preview);
            persistence::save_slots(mgr.slots_for_persistence()).ok();
            drop(mgr);

            show_hud_window(app);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-copied", &slot_info);
            }
        }
        Err(e) => {
            eprintln!("[ClipX] Failed to copy to slot {}: {}", slot_index, e);
        }
    }
}

fn handle_paste(app: &AppHandle, slot_index: usize) {
    let state = app.state::<SharedClipboardManager>();
    let mgr = state.lock().unwrap();

    match mgr.paste_from_slot(slot_index) {
        Ok(()) => {
            eprintln!("[ClipX] Pasted from slot {}", slot_index);
            drop(mgr);
            show_hud_window(app);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-pasted", slot_index);
            }
        }
        Err(e) => {
            eprintln!("[ClipX] Failed to paste from slot {}: {}", slot_index, e);
        }
    }
}
