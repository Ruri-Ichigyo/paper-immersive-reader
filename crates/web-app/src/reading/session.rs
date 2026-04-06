use paper_immersive_reader_core::domain::{Block, Paper, PaperId, ReadingProgress};

use crate::reading::navigation::NavigationDirection;
use crate::storage::{StorageError, WebStorage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadingSessionError {
    Storage(StorageError),
    MissingPaper(PaperId),
}

impl From<StorageError> for ReadingSessionError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

#[derive(Debug, Clone)]
pub struct ReadingSession {
    storage: WebStorage,
    paper: Paper,
    blocks: Vec<Block>,
    reading_progress: ReadingProgress,
}

impl ReadingSession {
    pub fn load(storage: WebStorage, paper_id: PaperId) -> Result<Self, ReadingSessionError> {
        let bundle = storage
            .load_paper_bundle(&paper_id)?
            .ok_or_else(|| ReadingSessionError::MissingPaper(paper_id.clone()))?;

        Ok(Self {
            storage,
            paper: bundle.paper,
            blocks: bundle.blocks,
            reading_progress: bundle.reading_progress,
        })
    }

    pub fn paper(&self) -> &Paper {
        &self.paper
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn reading_progress(&self) -> &ReadingProgress {
        &self.reading_progress
    }

    pub fn current_block(&self) -> &Block {
        &self.blocks[self.reading_progress.current_block_index as usize]
    }

    pub fn navigate(
        &mut self,
        direction: NavigationDirection,
        updated_at_ms: u64,
    ) -> Result<(), ReadingSessionError> {
        let current_index = self.reading_progress.current_block_index as usize;
        let next_index = match direction {
            NavigationDirection::Previous => current_index.saturating_sub(1),
            NavigationDirection::Next => {
                if current_index + 1 < self.blocks.len() {
                    current_index + 1
                } else {
                    current_index
                }
            }
        };

        self.reading_progress.current_block_index = next_index as u32;
        self.reading_progress.current_block_id = Some(self.blocks[next_index].id.clone());
        self.reading_progress.updated_at_ms = updated_at_ms;

        self.storage.save_paper_bundle(
            self.paper.clone(),
            self.blocks.clone(),
            self.reading_progress.clone(),
        )?;

        Ok(())
    }
}
