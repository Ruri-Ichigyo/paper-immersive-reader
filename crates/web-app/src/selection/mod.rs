pub mod popover;

use paper_immersive_reader_core::domain::{BlockId, PaperId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    InvalidOffsets,
    MissingSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionState {
    pub paper_id: PaperId,
    pub block_id: BlockId,
    pub quote: String,
    pub start_offset: u32,
    pub end_offset: u32,
}
