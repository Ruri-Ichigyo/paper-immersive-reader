#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    Next,
    Previous,
}

impl NavigationAction {
    pub fn apply(&self, current_index: u32, total_blocks: u32) -> Option<u32> {
        if total_blocks == 0 {
            return None;
        }
        match self {
            NavigationAction::Next => {
                if current_index + 1 < total_blocks {
                    Some(current_index + 1)
                } else {
                    None
                }
            }
            NavigationAction::Previous => {
                if current_index > 0 {
                    Some(current_index - 1)
                } else {
                    None
                }
            }
        }
    }
}
