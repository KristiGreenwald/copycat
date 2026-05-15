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
            let state = app_hud.state::<SharedClipboardManager>();
            let mgr = state.lock().unwrap();
            let occupied = mgr.get_occupied_slots().len();
            drop(mgr);
            show_hud_window(&app_hud, occupied);
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

pub fn show_hud_window(app: &AppHandle, slot_count: usize) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let screen_size = monitor.size();
            let screen_pos = monitor.position();
            let scale = monitor.scale_factor();
            let win_w = (280.0 * scale) as i32;
            // Dynamic height: header(44) + slots(48 each) + padding(20)
            let count = if slot_count == 0 { 1 } else { slot_count };
            let content_h = 44.0 + (count as f64 * 48.0) + 20.0;
            let win_h = (content_h * scale) as i32;
            // Use 80px bottom margin to clear the macOS Dock
            let margin_x = (12.0 * scale) as i32;
            let margin_y = (80.0 * scale) as i32;
            let x = screen_pos.x + screen_size.width as i32 - win_w - margin_x;
            let y = screen_pos.y + screen_size.height as i32 - win_h - margin_y;
            let _ = window.set_size(tauri::PhysicalSize::new(win_w as u32, win_h as u32));
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
        window.show().ok();
        window.set_focus().ok();
        eprintln!("[ClipX] HUD window shown ({} slots)", slot_count);
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
    // Simulate Cmd+C to copy current selection to system clipboard
    #[cfg(target_os = "macos")]
    {
        simulate_copy_keystroke();
        // Brief delay to let the clipboard update
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let state = app.state::<SharedClipboardManager>();
    let mut mgr = state.lock().unwrap();

    match mgr.copy_to_slot(slot_index) {
        Ok(slot_info) => {
            eprintln!("[ClipX] Copied to slot {}: '{}'", slot_index, slot_info.preview);
            persistence::save_slots(mgr.slots_for_persistence()).ok();
            let occupied = mgr.get_occupied_slots().len();
            drop(mgr);

            show_hud_window(app, occupied);
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
            let occupied = mgr.get_occupied_slots().len();
            drop(mgr);
            show_hud_window(app, occupied);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-pasted", slot_index);
            }
        }
        Err(e) => {
            eprintln!("[ClipX] Failed to paste from slot {}: {}", slot_index, e);
        }
    }
}

#[cfg(target_os = "macos")]
fn simulate_copy_keystroke() {
    use std::ptr;

    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: *const std::ffi::c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> *mut std::ffi::c_void;
        fn CGEventSetFlags(event: *mut std::ffi::c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut std::ffi::c_void);
        fn CFRelease(cf: *const std::ffi::c_void);
    }

    // kVK_ANSI_C = 0x08, kCGEventFlagMaskCommand = 0x100000
    const VK_C: u16 = 0x08;
    const CMD_FLAG: u64 = 0x100000;

    unsafe {
        let key_down = CGEventCreateKeyboardEvent(ptr::null(), VK_C, true);
        CGEventSetFlags(key_down, CMD_FLAG);
        CGEventPost(0, key_down); // kCGHIDEventTap = 0

        let key_up = CGEventCreateKeyboardEvent(ptr::null(), VK_C, false);
        CGEventSetFlags(key_up, CMD_FLAG);
        CGEventPost(0, key_up);

        CFRelease(key_down as *const _);
        CFRelease(key_up as *const _);
    }
}
