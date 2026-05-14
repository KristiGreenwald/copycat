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
        let copy_shortcut: Shortcut = copy_shortcut_str.parse().map_err(|e| {
            format!("Failed to parse copy shortcut '{}': {:?}", copy_shortcut_str, e)
        })?;
        let slot_index = i;
        let app_copy = app.clone();
        app.global_shortcut().on_shortcut(copy_shortcut, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
            if event.state == ShortcutState::Pressed {
                handle_copy(&app_copy, slot_index);
            }
        })?;

        // Register paste shortcut
        let paste_shortcut_str = format!("{}+{}", shortcuts_config.paste_modifier, key);
        let paste_shortcut: Shortcut = paste_shortcut_str.parse().map_err(|e| {
            format!("Failed to parse paste shortcut '{}': {:?}", paste_shortcut_str, e)
        })?;
        let app_paste = app.clone();
        app.global_shortcut().on_shortcut(paste_shortcut, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
            if event.state == ShortcutState::Pressed {
                handle_paste(&app_paste, slot_index);
            }
        })?;
    }

    // Register HUD toggle shortcut
    let toggle_hud: Shortcut = shortcuts_config
        .toggle_hud
        .parse()
        .map_err(|e| format!("Failed to parse HUD toggle shortcut: {:?}", e))?;
    let app_hud = app.clone();
    app.global_shortcut().on_shortcut(toggle_hud, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
        if event.state == ShortcutState::Pressed {
            if let Some(window) = app_hud.get_webview_window("main") {
                let _ = window.emit("toggle-hud", ());
            }
        }
    })?;

    // Register clear-all shortcut
    let clear_all: Shortcut = shortcuts_config
        .clear_all
        .parse()
        .map_err(|e| format!("Failed to parse clear-all shortcut: {:?}", e))?;
    let app_clear = app.clone();
    app.global_shortcut().on_shortcut(clear_all, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
        if event.state == ShortcutState::Pressed {
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

    Ok(())
}

fn handle_copy(app: &AppHandle, slot_index: usize) {
    let state = app.state::<SharedClipboardManager>();
    let mut mgr = state.lock().unwrap();

    match mgr.copy_to_slot(slot_index) {
        Ok(slot_info) => {
            persistence::save_slots(mgr.slots_for_persistence()).ok();
            drop(mgr);

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-copied", &slot_info);
            }
        }
        Err(e) => {
            log::error!("Failed to copy to slot {}: {}", slot_index, e);
        }
    }
}

fn handle_paste(app: &AppHandle, slot_index: usize) {
    let state = app.state::<SharedClipboardManager>();
    let mgr = state.lock().unwrap();

    match mgr.paste_from_slot(slot_index) {
        Ok(()) => {
            drop(mgr);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-pasted", slot_index);
            }
        }
        Err(e) => {
            log::error!("Failed to paste from slot {}: {}", slot_index, e);
        }
    }
}
