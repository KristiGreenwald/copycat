use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template: String,
    pub assigned_slot: Option<usize>,
}

pub fn render_prompt(template: &str, content: &str) -> String {
    if template.contains("{{content}}") {
        template.replace("{{content}}", content)
    } else {
        // If the user didn't include {{content}}, append the text after the instruction
        format!("{}\n\nText:\n{}\n\nOutput:", template.trim(), content)
    }
}
