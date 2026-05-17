use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const NUM_SLOTS: usize = 10;
const PREVIEW_LENGTH: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlotContent {
    Text(String),
    Image(Vec<u8>),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingState {
    Idle,
    Processing,
    Complete,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub index: usize,
    pub content: SlotContent,
    pub preview: String,
    pub prompt_id: Option<String>,
    pub processing_state: ProcessingState,
    pub original_preview: Option<String>,
    pub original_content: Option<String>,
}

impl Slot {
    pub fn empty(index: usize) -> Self {
        Self {
            index,
            content: SlotContent::Empty,
            preview: String::new(),
            prompt_id: None,
            processing_state: ProcessingState::Idle,
            original_preview: None,
            original_content: None,
        }
    }

    pub fn is_occupied(&self) -> bool {
        !matches!(self.content, SlotContent::Empty)
    }

    fn generate_preview(content: &SlotContent) -> String {
        match content {
            SlotContent::Text(text) => {
                let clean: String = text.chars().take(PREVIEW_LENGTH).collect();
                clean.replace('\n', " ")
            }
            SlotContent::Image(_) => "[Image]".to_string(),
            SlotContent::Empty => String::new(),
        }
    }

    pub fn set_content(&mut self, content: SlotContent) {
        self.preview = Self::generate_preview(&content);
        self.content = content;
        self.processing_state = ProcessingState::Idle;
        self.original_preview = None;
        self.original_content = None;
    }

    pub fn set_ai_processing(&mut self) {
        self.original_preview = Some(self.preview.clone());
        self.original_content = match &self.content {
            SlotContent::Text(t) => Some(t.clone()),
            _ => None,
        };
        self.processing_state = ProcessingState::Processing;
    }

    pub fn set_ai_result(&mut self, content: SlotContent) {
        self.preview = Self::generate_preview(&content);
        self.content = content;
        self.processing_state = ProcessingState::Complete;
    }

    pub fn set_ai_error(&mut self, error: String) {
        self.processing_state = ProcessingState::Error(error);
        self.original_preview = None;
        self.original_content = None;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlotInfo {
    pub index: usize,
    pub preview: String,
    pub occupied: bool,
    pub processing_state: ProcessingState,
    pub original_preview: Option<String>,
    pub has_prompt: bool,
    pub prompt_id: Option<String>,
}

impl From<&Slot> for SlotInfo {
    fn from(slot: &Slot) -> Self {
        Self {
            index: slot.index,
            preview: slot.preview.clone(),
            occupied: slot.is_occupied(),
            processing_state: slot.processing_state.clone(),
            original_preview: slot.original_preview.clone(),
            has_prompt: slot.prompt_id.is_some(),
            prompt_id: slot.prompt_id.clone(),
        }
    }
}

pub struct ClipboardManager {
    slots: [Slot; NUM_SLOTS],
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|i| Slot::empty(i)),
        }
    }

    pub fn from_slots(slots: Vec<Slot>) -> Self {
        let mut manager = Self::new();
        for slot in slots {
            let idx = slot.index;
            if idx < NUM_SLOTS {
                manager.slots[idx] = slot;
            }
        }
        manager
    }

    pub fn copy_to_slot(&mut self, slot_index: usize) -> Result<SlotInfo, String> {
        if slot_index >= NUM_SLOTS {
            return Err(format!("Invalid slot index: {}", slot_index));
        }

        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?;

        let content = if let Ok(text) = clipboard.get_text() {
            if text.is_empty() {
                SlotContent::Empty
            } else {
                SlotContent::Text(text)
            }
        } else if let Ok(img) = clipboard.get_image() {
            let rgba = img.bytes.into_owned();
            SlotContent::Image(rgba)
        } else {
            return Err("Clipboard is empty or contains unsupported content".to_string());
        };

        self.slots[slot_index].set_content(content);
        Ok(SlotInfo::from(&self.slots[slot_index]))
    }

    pub fn paste_from_slot(&self, slot_index: usize) -> Result<(), String> {
        if slot_index >= NUM_SLOTS {
            return Err(format!("Invalid slot index: {}", slot_index));
        }

        let slot = &self.slots[slot_index];
        if !slot.is_occupied() {
            return Err(format!("Slot {} is empty", slot_index));
        }

        if slot.processing_state == ProcessingState::Processing {
            return Err(format!("Slot {} is still being processed by AI", slot_index));
        }

        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?;

        match &slot.content {
            SlotContent::Text(text) => {
                clipboard
                    .set_text(text.clone())
                    .map_err(|e| format!("Failed to set clipboard: {}", e))?;
            }
            SlotContent::Image(data) => {
                let img = arboard::ImageData {
                    width: 0,
                    height: 0,
                    bytes: std::borrow::Cow::Borrowed(data),
                };
                clipboard
                    .set_image(img)
                    .map_err(|e| format!("Failed to set clipboard image: {}", e))?;
            }
            SlotContent::Empty => {
                return Err("Slot is empty".to_string());
            }
        }

        Ok(())
    }

    pub fn clear_slot(&mut self, slot_index: usize) -> Result<(), String> {
        if slot_index >= NUM_SLOTS {
            return Err(format!("Invalid slot index: {}", slot_index));
        }
        self.slots[slot_index] = Slot::empty(slot_index);
        Ok(())
    }

    pub fn clear_all(&mut self) {
        self.slots = std::array::from_fn(|i| Slot::empty(i));
    }

    pub fn get_all_slots(&self) -> Vec<SlotInfo> {
        self.slots.iter().map(SlotInfo::from).collect()
    }

    pub fn get_occupied_slots(&self) -> Vec<SlotInfo> {
        self.slots
            .iter()
            .filter(|s| s.is_occupied())
            .map(SlotInfo::from)
            .collect()
    }

    pub fn get_slot(&self, index: usize) -> Option<&Slot> {
        self.slots.get(index)
    }

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Slot> {
        self.slots.get_mut(index)
    }

    pub fn slots_for_persistence(&self) -> &[Slot; NUM_SLOTS] {
        &self.slots
    }

    pub fn set_slot_prompt(&mut self, slot_index: usize, prompt_id: Option<String>) -> Result<(), String> {
        if slot_index >= NUM_SLOTS {
            return Err(format!("Invalid slot index: {}", slot_index));
        }
        self.slots[slot_index].prompt_id = prompt_id;
        Ok(())
    }
}

pub type SharedClipboardManager = Mutex<ClipboardManager>;
