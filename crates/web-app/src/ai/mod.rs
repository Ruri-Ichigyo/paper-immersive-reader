pub mod client;

use paper_immersive_reader_core::domain::{BlockId, PaperId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiRequestType {
    Translate {
        source_language: String,
        target_language: String,
    },
    ExplainTerm {
        language: String,
    },
    ExplainSentence {
        language: String,
    },
}

#[derive(Debug, Clone)]
pub struct AiResult {
    pub record_type: String,
    pub input: String,
    pub output: String,
    pub language: String,
    pub normalized_term: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    Offline,
    GatewayError(String),
}

pub struct AiContext {
    pub paper_id: PaperId,
    pub block_id: BlockId,
    pub selected_text: String,
    pub block_text: String,
}
