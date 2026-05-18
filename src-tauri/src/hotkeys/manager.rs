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
        eprintln!("[CopyCat] Registering copy shortcut: {}", copy_shortcut_str);
        let copy_shortcut: Shortcut = copy_shortcut_str.parse().map_err(|e| {
            format!("Failed to parse copy shortcut '{}': {:?}", copy_shortcut_str, e)
        })?;
        let slot_index = i;
        let app_copy = app.clone();
        app.global_shortcut().on_shortcut(copy_shortcut, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
            if event.state == ShortcutState::Pressed {
                eprintln!("[CopyCat] Copy shortcut pressed for slot {}", slot_index);
                handle_copy(&app_copy, slot_index);
            }
        })?;

        // Register paste shortcut
        let paste_shortcut_str = format!("{}+{}", shortcuts_config.paste_modifier, key);
        eprintln!("[CopyCat] Registering paste shortcut: {}", paste_shortcut_str);
        let paste_shortcut: Shortcut = paste_shortcut_str.parse().map_err(|e| {
            format!("Failed to parse paste shortcut '{}': {:?}", paste_shortcut_str, e)
        })?;
        let app_paste = app.clone();
        app.global_shortcut().on_shortcut(paste_shortcut, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
            if event.state == ShortcutState::Pressed {
                eprintln!("[CopyCat] Paste shortcut pressed for slot {}", slot_index);
                handle_paste(&app_paste, slot_index);
            }
        })?;
    }

    // Register HUD toggle shortcut
    eprintln!("[CopyCat] Registering HUD toggle: {}", shortcuts_config.toggle_hud);
    let toggle_hud: Shortcut = shortcuts_config
        .toggle_hud
        .parse()
        .map_err(|e| format!("Failed to parse HUD toggle shortcut: {:?}", e))?;
    let app_hud = app.clone();
    app.global_shortcut().on_shortcut(toggle_hud, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
        if event.state == ShortcutState::Pressed {
            eprintln!("[CopyCat] HUD toggle pressed");
            let state = app_hud.state::<SharedClipboardManager>();
            let mgr = state.lock().unwrap();
            let occupied = mgr.get_occupied_slots().len();
            drop(mgr);
            show_hud_window(&app_hud, occupied);
            let _ = app_hud.get_webview_window("main").map(|w| w.emit("toggle-hud", ()));
        }
    })?;

    // Register clear-all shortcut
    eprintln!("[CopyCat] Registering clear-all: {}", shortcuts_config.clear_all);
    let clear_all: Shortcut = shortcuts_config
        .clear_all
        .parse()
        .map_err(|e| format!("Failed to parse clear-all shortcut: {:?}", e))?;
    let app_clear = app.clone();
    app.global_shortcut().on_shortcut(clear_all, move |_app, _shortcut: &Shortcut, event: ShortcutEvent| {
        if event.state == ShortcutState::Pressed {
            eprintln!("[CopyCat] Clear-all pressed");
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

    eprintln!("[CopyCat] All shortcuts registered successfully");
    Ok(())
}

pub fn show_hud_window(app: &AppHandle, slot_count: usize) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let screen_size = monitor.size();
            let screen_pos = monitor.position();
            let scale = monitor.scale_factor();
            let win_w = (300.0 * scale) as i32;
            let count = if slot_count == 0 { 1 } else { slot_count };
            let content_h = 50.0 + (count as f64 * 52.0) + 24.0;
            let win_h = (content_h * scale) as i32;

            // Read position from config
            let position = {
                let config = app.state::<SharedConfig>();
                let guard = config.lock().unwrap();
                guard.hud_position.clone()
            };

            let margin = (24.0 * scale) as i32;
            let dock_margin = (10.0 * scale) as i32;
            let sw = screen_size.width as i32;
            let sh = screen_size.height as i32;
            let sx = screen_pos.x;
            let sy = screen_pos.y;

            let (x, y) = match position.as_str() {
                "tl" => (sx + margin, sy + margin),
                "tc" => (sx + (sw - win_w) / 2, sy + margin),
                "tr" => (sx + sw - win_w - margin, sy + margin),
                "ml" => (sx + margin, sy + (sh - win_h) / 2),
                "mc" => (sx + (sw - win_w) / 2, sy + (sh - win_h) / 2),
                "mr" => (sx + sw - win_w - margin, sy + (sh - win_h) / 2),
                "bl" => (sx + margin, sy + sh - win_h - dock_margin),
                "bc" => (sx + (sw - win_w) / 2, sy + sh - win_h - dock_margin),
                _ => (sx + sw - win_w - margin, sy + sh - win_h - dock_margin), // br default
            };

            let _ = window.set_size(tauri::PhysicalSize::new(win_w as u32, win_h as u32));
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
        window.show().ok();
        window.set_focus().ok();
        eprintln!("[CopyCat] HUD window shown ({} slots)", slot_count);
    } else {
        eprintln!("[CopyCat] WARNING: main window not found");
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
            eprintln!("[CopyCat] Copied to slot {}: '{}'", slot_index, slot_info.preview);
            let has_prompt = slot_info.has_prompt;
            persistence::save_slots(mgr.slots_for_persistence()).ok();
            let occupied = mgr.get_occupied_slots().len();
            drop(mgr);

            show_hud_window(app, occupied);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-copied", &slot_info);
            }

            // Trigger AI processing if slot has an assigned prompt
            if has_prompt {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    eprintln!("[CopyCat AI] Auto-processing slot {} with assigned prompt", slot_index);
                    let engine = app_clone.state::<crate::ai::engine::SharedAiEngine>();
                    let clipboard = app_clone.state::<SharedClipboardManager>();
                    let config = app_clone.state::<crate::settings::config::SharedConfig>();

                    let prompt_template = {
                        let config_guard = config.lock().unwrap();
                        config_guard.get_prompt_for_slot(slot_index).cloned()
                    };

                    if let Some(prompt) = prompt_template {
                        let content = {
                            let mgr = clipboard.lock().unwrap();
                            mgr.get_slot(slot_index).and_then(|s| match &s.content {
                                crate::clipboard::manager::SlotContent::Text(t) => Some(t.clone()),
                                _ => None,
                            })
                        };

                        if let Some(text) = content {
                            {
                                let mut mgr = clipboard.lock().unwrap();
                                if let Some(slot) = mgr.get_slot_mut(slot_index) {
                                    slot.set_ai_processing();
                                }
                            }
                            if let Some(window) = app_clone.get_webview_window("main") {
                                let _ = window.emit("slots-updated", ());
                            }

                            let rendered = crate::ai::prompts::render_prompt(&prompt.template, &text);
                            let eng = engine.lock().await;
                            match eng.generate(&rendered).await {
                                Ok(output) => {
                                    let mut mgr = clipboard.lock().unwrap();
                                    if let Some(slot) = mgr.get_slot_mut(slot_index) {
                                        slot.set_ai_result(crate::clipboard::manager::SlotContent::Text(output));
                                    }
                                    persistence::save_slots(mgr.slots_for_persistence()).ok();
                                    drop(mgr);
                                    if let Some(window) = app_clone.get_webview_window("main") {
                                        let _ = window.emit("slots-updated", ());
                                    }
                                    eprintln!("[CopyCat AI] Slot {} auto-processed successfully", slot_index);
                                }
                                Err(e) => {
                                    let mut mgr = clipboard.lock().unwrap();
                                    if let Some(slot) = mgr.get_slot_mut(slot_index) {
                                        slot.set_ai_error(e.clone());
                                    }
                                    drop(mgr);
                                    if let Some(window) = app_clone.get_webview_window("main") {
                                        let _ = window.emit("slots-updated", ());
                                    }
                                    eprintln!("[CopyCat AI] Slot {} auto-processing failed: {}", slot_index, e);
                                }
                            }
                        }
                    }
                });
            }
        }
        Err(e) => {
            eprintln!("[CopyCat] Failed to copy to slot {}: {}", slot_index, e);
        }
    }
}

fn handle_paste(app: &AppHandle, slot_index: usize) {
    let state = app.state::<SharedClipboardManager>();
    let mgr = state.lock().unwrap();

    match mgr.paste_from_slot(slot_index) {
        Ok(()) => {
            eprintln!("[CopyCat] Pasted from slot {}", slot_index);
            let occupied = mgr.get_occupied_slots().len();
            drop(mgr);

            // Simulate Cmd+V to paste into the focused app
            #[cfg(target_os = "macos")]
            {
                std::thread::sleep(std::time::Duration::from_millis(50));
                simulate_paste_keystroke();
            }

            show_hud_window(app, occupied);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("slot-pasted", slot_index);
            }
        }
        Err(e) => {
            eprintln!("[CopyCat] Failed to paste from slot {}: {}", slot_index, e);
        }
    }
}

#[cfg(target_os = "macos")]
fn simulate_copy_keystroke() {
    simulate_key_with_cmd(0x08); // kVK_ANSI_C
}

#[cfg(target_os = "macos")]
fn simulate_paste_keystroke() {
    simulate_key_with_cmd(0x09); // kVK_ANSI_V
}

#[cfg(target_os = "macos")]
fn simulate_key_with_cmd(virtual_key: u16) {
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

    const CMD_FLAG: u64 = 0x100000; // kCGEventFlagMaskCommand

    unsafe {
        let key_down = CGEventCreateKeyboardEvent(ptr::null(), virtual_key, true);
        CGEventSetFlags(key_down, CMD_FLAG);
        CGEventPost(0, key_down);

        let key_up = CGEventCreateKeyboardEvent(ptr::null(), virtual_key, false);
        CGEventSetFlags(key_up, CMD_FLAG);
        CGEventPost(0, key_up);

        CFRelease(key_down as *const _);
        CFRelease(key_up as *const _);
    }
}
