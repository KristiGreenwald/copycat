use crate::clipboard::manager::{ClipboardManager, Slot};
use std::fs;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("com.krisgreenwald.clipx");
    fs::create_dir_all(&dir).ok();
    dir
}

fn slots_file() -> PathBuf {
    data_dir().join("slots.json")
}

pub fn save_slots(slots: &[Slot]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(slots)
        .map_err(|e| format!("Failed to serialize slots: {}", e))?;
    fs::write(slots_file(), json).map_err(|e| format!("Failed to write slots file: {}", e))?;
    Ok(())
}

pub fn load_slots() -> Result<ClipboardManager, String> {
    let path = slots_file();
    if !path.exists() {
        return Ok(ClipboardManager::new());
    }

    let json = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read slots file: {}", e))?;
    let slots: Vec<Slot> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to deserialize slots: {}", e))?;

    Ok(ClipboardManager::from_slots(slots))
}
