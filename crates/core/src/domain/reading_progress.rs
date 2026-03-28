use super::block::BlockId;
use super::paper::PaperId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadingProgress {
    pub paper_id: PaperId,
    pub current_block_index: u32,
    pub current_block_id: Option<BlockId>,
    pub updated_at_ms: u64,
}
