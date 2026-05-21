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
    pull_client: reqwest::Client,
    model_name: String,
}

impl AiEngine {
    pub fn new(model_name: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            pull_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3600))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
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
        eprintln!("[CopyCat AI] Pulling model: {}", self.model_name);

        let req = OllamaPullRequest {
            name: self.model_name.clone(),
            stream: true,
        };

        let resp = self
            .pull_client
            .post(format!("{}/api/pull", OLLAMA_BASE_URL))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Failed to pull model: {}", e))?;

        if resp.status().is_success() {
            // Stream the response to avoid timeout — each chunk resets the read timer
            use futures_util::StreamExt;
            let mut stream = resp.bytes_stream();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        // Check for error in streamed JSON lines
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            if text.contains("\"error\"") {
                                return Err(format!("Pull error: {}", text));
                            }
                        }
                    }
                    Err(e) => return Err(format!("Stream error during pull: {}", e)),
                }
            }
            eprintln!("[CopyCat AI] Model pulled successfully");
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Failed to pull model: {}", text))
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        eprintln!(
            "[CopyCat AI] Generating with model '{}', prompt length: {}",
            self.model_name,
            prompt.len()
        );

        // Use the chat API with system + user messages for better instruction following
        let req = serde_json::json!({
            "model": self.model_name,
            "stream": false,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant. Follow the user's instruction precisely. Only output the requested result — no explanations, preambles, or commentary."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let resp = self
            .client
            .post(format!("{}/api/chat", OLLAMA_BASE_URL))
            .json(&req)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama error: {}", text));
        }

        let chat_resp: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let output = chat_resp["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        eprintln!(
            "[CopyCat AI] Generated {} chars",
            output.len()
        );
        Ok(output)
    }
}

pub type SharedAiEngine = Arc<TokioMutex<AiEngine>>;
