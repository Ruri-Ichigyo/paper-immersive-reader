use paper_immersive_reader_core::domain::{BlockId, Paper, PaperId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPaper {
    pub paper: Paper,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiRecord {
    pub id: String,
    pub paper_id: PaperId,
    pub block_id: BlockId,
    pub record_type: String,
    pub input: String,
    pub output: String,
    pub language: String,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub explanation_language: String,
    pub custom_tags: Vec<String>,
    pub ai_gateway_url: Option<String>,
    pub ai_gateway_token_ref: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            explanation_language: "en".to_string(),
            custom_tags: Vec::new(),
            ai_gateway_url: None,
            ai_gateway_token_ref: None,
        }
    }
}
