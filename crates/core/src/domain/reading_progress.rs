use super::block::BlockId;
use super::paper::PaperId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub paper_id: PaperId,
    pub current_block_index: u32,
    pub current_block_id: Option<BlockId>,
    pub updated_at_ms: u64,
}
