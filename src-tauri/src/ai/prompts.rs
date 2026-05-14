use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template: String,
    pub assigned_slot: Option<usize>,
}

pub fn render_prompt(template: &str, content: &str) -> String {
    template.replace("{{content}}", content)
}
