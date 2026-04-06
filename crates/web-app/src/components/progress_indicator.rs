#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressIndicator {
    pub current_block_index: u32,
    pub total_blocks: usize,
    pub progress_percent: u8,
}

impl ProgressIndicator {
    pub fn from_position(current_block_index: u32, total_blocks: usize) -> Self {
        let progress_percent = if total_blocks == 0 {
            0
        } else {
            (((current_block_index as usize + 1) * 100) / total_blocks) as u8
        };

        Self {
            current_block_index,
            total_blocks,
            progress_percent,
        }
    }
}
