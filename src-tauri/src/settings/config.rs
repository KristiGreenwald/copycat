use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template: String,
    pub assigned_slot: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub copy_modifier: String,
    pub paste_modifier: String,
    pub toggle_hud: String,
    pub clear_all: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            copy_modifier: "CmdOrCtrl+Alt".to_string(),
            paste_modifier: "CmdOrCtrl+Alt+Shift".to_string(),
            toggle_hud: "Control+Alt+Super+Space".to_string(),
            clear_all: "CmdOrCtrl+Alt+Backspace".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelConfig {
    pub model_name: String,
    pub model_path: Option<String>,
    pub download_url: Option<String>,
    pub downloaded: bool,
}

impl Default for AiModelConfig {
    fn default() -> Self {
        Self {
            model_name: "llama3.2:1b".to_string(),
            model_path: None,
            download_url: None,
            downloaded: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcuts: ShortcutConfig,
    pub ai_model: AiModelConfig,
    pub prompts: Vec<PromptTemplate>,
    pub hud_duration_secs: u64,
    pub launch_at_startup: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcuts: ShortcutConfig::default(),
            ai_model: AiModelConfig::default(),
            prompts: vec![
                PromptTemplate {
                    id: "summarize".to_string(),
                    name: "Summarize in 5 bullet points".to_string(),
                    template: "Summarize the following text in exactly 5 concise bullet points:\n\n{{content}}".to_string(),
                    assigned_slot: None,
                },
                PromptTemplate {
                    id: "fix-grammar".to_string(),
                    name: "Fix grammar".to_string(),
                    template: "Fix the grammar and spelling in the following text. Only return the corrected text, nothing else:\n\n{{content}}".to_string(),
                    assigned_slot: None,
                },
                PromptTemplate {
                    id: "translate-spanish".to_string(),
                    name: "Translate to Spanish".to_string(),
                    template: "Translate the following text to Spanish. Only return the translation, nothing else:\n\n{{content}}".to_string(),
                    assigned_slot: None,
                },
            ],
            hud_duration_secs: 10,
            launch_at_startup: false,
        }
    }
}

fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("com.krisgreenwald.copycat");
    fs::create_dir_all(&dir).ok();
    dir
}

fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_file();
        if !path.exists() {
            let config = Self::default();
            config.save().ok();
            return config;
        }

        fs::read_to_string(&path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(config_file(), json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    pub fn get_prompt_for_slot(&self, slot_index: usize) -> Option<&PromptTemplate> {
        self.prompts
            .iter()
            .find(|p| p.assigned_slot == Some(slot_index))
    }

    pub fn slot_prompt_map(&self) -> HashMap<usize, &PromptTemplate> {
        self.prompts
            .iter()
            .filter_map(|p| p.assigned_slot.map(|slot| (slot, p)))
            .collect()
    }
}

pub type SharedConfig = std::sync::Mutex<AppConfig>;
