use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaListResponse {
    pub models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaPullRequest {
    pub name: String,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaPullProgress {
    pub status: String,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub completed: u64,
}

pub struct AiEngine {
    client: reqwest::Client,
    model_name: String,
}

impl AiEngine {
    pub fn new(model_name: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            model_name: model_name.to_string(),
        }
    }

    pub fn set_model(&mut self, model_name: &str) {
        self.model_name = model_name.to_string();
    }

    pub async fn is_ollama_running(&self) -> bool {
        match self.client.get(OLLAMA_BASE_URL).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModelInfo>, String> {
        let resp = self
            .client
            .get(format!("{}/api/tags", OLLAMA_BASE_URL))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        let list: OllamaListResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(list.models)
    }

    pub async fn is_model_available(&self) -> bool {
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.name.starts_with(&self.model_name)),
            Err(_) => false,
        }
    }

    pub async fn pull_model(&self) -> Result<(), String> {
        eprintln!("[ClipX AI] Pulling model: {}", self.model_name);

        let req = OllamaPullRequest {
            name: self.model_name.clone(),
            stream: false,
        };

        let resp = self
            .client
            .post(format!("{}/api/pull", OLLAMA_BASE_URL))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Failed to pull model: {}", e))?;

        if resp.status().is_success() {
            eprintln!("[ClipX AI] Model pulled successfully");
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Failed to pull model: {}", text))
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        eprintln!(
            "[ClipX AI] Generating with model '{}', prompt length: {}",
            self.model_name,
            prompt.len()
        );

        let req = OllamaGenerateRequest {
            model: self.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let resp = self
            .client
            .post(format!("{}/api/generate", OLLAMA_BASE_URL))
            .json(&req)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama error: {}", text));
        }

        let gen_resp: OllamaGenerateResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        eprintln!(
            "[ClipX AI] Generated {} chars",
            gen_resp.response.len()
        );
        Ok(gen_resp.response.trim().to_string())
    }
}

pub type SharedAiEngine = Arc<TokioMutex<AiEngine>>;
