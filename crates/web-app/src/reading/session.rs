use paper_immersive_reader_core::domain::{Block, BlockId, PaperId, ReadingProgress};

use crate::storage::AppStorage;

use super::navigation::NavigationAction;

#[derive(Debug, Clone)]
pub struct ReadingSession {
    paper_id: PaperId,
    blocks: Vec<Block>,
    current_index: u32,
    total_blocks: u32,
    updated_at_ms: u64,
}

impl ReadingSession {
    pub fn load<S: AppStorage>(
        store: &S,
        paper_id: PaperId,
    ) -> Result<Option<Self>, crate::storage::StorageError> {
        let blocks = store.load_blocks(&paper_id)?;
        if blocks.is_empty() {
            return Ok(None);
        }

        let restored_index = store
            .load_progress(&paper_id)?
            .map(|p| p.current_block_index)
            .unwrap_or(0);

        let current_index = if (restored_index as usize) < blocks.len() {
            restored_index
        } else {
            0
        };

        let total_blocks = blocks.len() as u32;
        let updated_at_ms = current_time_ms();

        let session = Self {
            paper_id,
            blocks,
            current_index,
            total_blocks,
            updated_at_ms,
        };

        Ok(Some(session))
    }

    pub fn current_block(&self) -> Option<&Block> {
        self.blocks.get(self.current_index as usize)
    }

    pub fn current_block_id(&self) -> Option<BlockId> {
        self.current_block().map(|b| b.id.clone())
    }

    pub fn current_index(&self) -> u32 {
        self.current_index
    }

    pub fn total_blocks(&self) -> u32 {
        self.total_blocks
    }

    pub fn progress_percent(&self) -> u32 {
        if self.total_blocks == 0 {
            return 0;
        }
        ((self.current_index + 1) as u64 * 100 / self.total_blocks as u64) as u32
    }

    pub fn navigate<S: AppStorage>(
        &mut self,
        action: NavigationAction,
        store: &mut S,
    ) -> Result<bool, crate::storage::StorageError> {
        match action.apply(self.current_index, self.total_blocks) {
            Some(new_index) => {
                self.current_index = new_index;
                self.updated_at_ms = current_time_ms();
                self.persist_progress(store)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn persist_progress<S: AppStorage>(
        &self,
        store: &mut S,
    ) -> Result<(), crate::storage::StorageError> {
        let progress = ReadingProgress {
            paper_id: self.paper_id.clone(),
            current_block_index: self.current_index,
            current_block_id: self.current_block_id(),
            updated_at_ms: self.updated_at_ms,
        };
        store.save_progress(&progress)
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
