#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockNavigator {
    pub can_go_previous: bool,
    pub can_go_next: bool,
}

impl BlockNavigator {
    pub fn from_position(current_block_index: u32, total_blocks: usize) -> Self {
        Self {
            can_go_previous: current_block_index > 0,
            can_go_next: total_blocks > 0 && (current_block_index as usize + 1) < total_blocks,
        }
    }
}
