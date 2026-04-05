use super::error::DomainError;
use super::paper::PaperId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub paper_id: PaperId,
    pub order_index: u32,
    pub text: String,
    pub block_hash: Option<String>,
}

impl Block {
    pub fn try_new(
        id: BlockId,
        paper_id: PaperId,
        order_index: u32,
        text: String,
        block_hash: Option<String>,
    ) -> Result<Self, DomainError> {
        if text.trim().is_empty() {
            return Err(DomainError::EmptyBlockText);
        }

        Ok(Self {
            id,
            paper_id,
            order_index,
            text,
            block_hash,
        })
    }
}
