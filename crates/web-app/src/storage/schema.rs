use paper_immersive_reader_core::domain::{Block, Paper, PaperId, ReadingProgress};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperBundle {
    pub paper: Paper,
    pub blocks: Vec<Block>,
    pub reading_progress: ReadingProgress,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiRecordId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiRecordType {
    Translation,
    Explanation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiRecord {
    pub id: AiRecordId,
    pub paper_id: PaperId,
    pub block_id: Option<String>,
    pub record_type: AiRecordType,
    pub input: String,
    pub output: String,
    pub language: String,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderSettings {
    pub explanation_language: String,
    pub custom_tags: Vec<String>,
    pub ai_gateway_url: Option<String>,
    pub ai_gateway_token_ref: Option<String>,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            explanation_language: "en".to_string(),
            custom_tags: Vec::new(),
            ai_gateway_url: None,
            ai_gateway_token_ref: None,
        }
    }
}
