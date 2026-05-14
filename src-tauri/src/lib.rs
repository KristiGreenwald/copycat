mod ai;
mod clipboard;
mod hotkeys;
mod settings;
mod tray;

use clipboard::manager::{ClipboardManager, SharedClipboardManager, SlotInfo};
use clipboard::persistence;
use settings::config::{AppConfig, PromptTemplate, SharedConfig};
use std::sync::Mutex;

// ── Tauri Commands ──

#[tauri::command]
fn get_all_slots(state: tauri::State<SharedClipboardManager>) -> Vec<SlotInfo> {
    let mgr = state.lock().unwrap();
    mgr.get_all_slots()
}

#[tauri::command]
fn get_occupied_slots(state: tauri::State<SharedClipboardManager>) -> Vec<SlotInfo> {
    let mgr = state.lock().unwrap();
    mgr.get_occupied_slots()
}

#[tauri::command]
fn copy_to_slot(
    slot_index: usize,
    state: tauri::State<SharedClipboardManager>,
) -> Result<SlotInfo, String> {
    let mut mgr = state.lock().unwrap();
    let result = mgr.copy_to_slot(slot_index)?;
    persistence::save_slots(mgr.slots_for_persistence()).ok();
    Ok(result)
}

#[tauri::command]
fn paste_from_slot(
    slot_index: usize,
    state: tauri::State<SharedClipboardManager>,
) -> Result<(), String> {
    let mgr = state.lock().unwrap();
    mgr.paste_from_slot(slot_index)
}

#[tauri::command]
fn clear_slot(
    slot_index: usize,
    state: tauri::State<SharedClipboardManager>,
) -> Result<Vec<SlotInfo>, String> {
    let mut mgr = state.lock().unwrap();
    mgr.clear_slot(slot_index)?;
    persistence::save_slots(mgr.slots_for_persistence()).ok();
    Ok(mgr.get_all_slots())
}

#[tauri::command]
fn clear_all_slots(state: tauri::State<SharedClipboardManager>) -> Vec<SlotInfo> {
    let mut mgr = state.lock().unwrap();
    mgr.clear_all();
    persistence::save_slots(mgr.slots_for_persistence()).ok();
    mgr.get_all_slots()
}

#[tauri::command]
fn get_config(state: tauri::State<SharedConfig>) -> AppConfig {
    state.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(
    config: AppConfig,
    state: tauri::State<SharedConfig>,
) -> Result<(), String> {
    let mut current = state.lock().unwrap();
    *current = config;
    current.save()
}

#[tauri::command]
fn get_prompts(state: tauri::State<SharedConfig>) -> Vec<PromptTemplate> {
    state.lock().unwrap().prompts.clone()
}

#[tauri::command]
fn save_prompt(
    prompt: PromptTemplate,
    state: tauri::State<SharedConfig>,
    clipboard_state: tauri::State<SharedClipboardManager>,
) -> Result<(), String> {
    let mut config = state.lock().unwrap();

    // If this prompt is assigned to a slot, clear any other prompt from that slot
    if let Some(slot) = prompt.assigned_slot {
        for p in config.prompts.iter_mut() {
            if p.id != prompt.id && p.assigned_slot == Some(slot) {
                p.assigned_slot = None;
            }
        }
    }

    if let Some(existing) = config.prompts.iter_mut().find(|p| p.id == prompt.id) {
        *existing = prompt.clone();
    } else {
        config.prompts.push(prompt.clone());
    }

    // Update clipboard manager's slot prompt mapping
    let mut mgr = clipboard_state.lock().unwrap();
    for i in 0..10 {
        let prompt_for_slot = config.get_prompt_for_slot(i).map(|p| p.id.clone());
        mgr.set_slot_prompt(i, prompt_for_slot).ok();
    }
    persistence::save_slots(mgr.slots_for_persistence()).ok();

    config.save()
}

#[tauri::command]
fn delete_prompt(
    prompt_id: String,
    state: tauri::State<SharedConfig>,
) -> Result<(), String> {
    let mut config = state.lock().unwrap();
    config.prompts.retain(|p| p.id != prompt_id);
    config.save()
}

#[tauri::command]
fn get_hud_duration(state: tauri::State<SharedConfig>) -> u64 {
    state.lock().unwrap().hud_duration_secs
}

// ── App Setup ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let clipboard_manager = persistence::load_slots().unwrap_or_else(|_| ClipboardManager::new());
    let config = AppConfig::load();

    // Sync prompt assignments to clipboard manager
    let mut mgr = clipboard_manager;
    for i in 0..10 {
        let prompt_id = config.get_prompt_for_slot(i).map(|p| p.id.clone());
        mgr.set_slot_prompt(i, prompt_id).ok();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage::<SharedClipboardManager>(Mutex::new(mgr))
        .manage::<SharedConfig>(Mutex::new(config))
        .invoke_handler(tauri::generate_handler![
            get_all_slots,
            get_occupied_slots,
            copy_to_slot,
            paste_from_slot,
            clear_slot,
            clear_all_slots,
            get_config,
            save_config,
            get_prompts,
            save_prompt,
            delete_prompt,
            get_hud_duration,
        ])
        .setup(|app| {
            // Set up system tray
            tray::menu::setup_tray(app.handle())?;

            // Register global shortcuts
            if let Err(e) = hotkeys::manager::register_shortcuts(app.handle()) {
                log::error!("Failed to register shortcuts: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
