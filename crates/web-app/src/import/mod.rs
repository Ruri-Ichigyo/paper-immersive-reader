pub mod pdf_bridge;

use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};
use paper_immersive_reader_importing::ImportPipelineError;

use crate::storage::schema::StoredPaper;
use crate::storage::AppStorage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebImportError {
    EmptyInput,
    ExtractionFailed,
    StorageFailed,
}

impl From<ImportPipelineError> for WebImportError {
    fn from(err: ImportPipelineError) -> Self {
        match err {
            ImportPipelineError::EmptyInput | ImportPipelineError::NoBlocksProduced => {
                WebImportError::EmptyInput
            }
            ImportPipelineError::OcrRequired => WebImportError::ExtractionFailed,
            _ => WebImportError::ExtractionFailed,
        }
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub struct WebImportService<S: AppStorage> {
    store: S,
    next_paper_counter: u32,
}

impl<S: AppStorage> WebImportService<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            next_paper_counter: 1,
        }
    }

    fn next_paper_id(&mut self) -> PaperId {
        let id = PaperId(format!("paper-{}", self.next_paper_counter));
        self.next_paper_counter += 1;
        id
    }

    pub fn import_paste(
        &mut self,
        text: String,
        title: Option<String>,
    ) -> Result<Paper, WebImportError> {
        if text.trim().is_empty() {
            return Err(WebImportError::EmptyInput);
        }

        let paper_id = self.next_paper_id();
        let now = current_time_ms();

        let paper = Paper {
            id: paper_id.clone(),
            title,
            source_type: PaperSourceType::Paste,
            source_ref: None,
            status: PaperStatus::ReadyText,
            parse_version: 1,
        };

        let blocks = segment_text(&paper, &text)?;
        let progress = ReadingProgress {
            paper_id: paper_id.clone(),
            current_block_index: 0,
            current_block_id: Some(blocks[0].id.clone()),
            updated_at_ms: now,
        };

        let stored = StoredPaper {
            paper: paper.clone(),
            created_at_ms: now,
            updated_at_ms: now,
        };

        self.store
            .save_paper(&stored)
            .map_err(|_| WebImportError::StorageFailed)?;
        self.store
            .save_blocks(&blocks)
            .map_err(|_| WebImportError::StorageFailed)?;
        self.store
            .save_progress(&progress)
            .map_err(|_| WebImportError::StorageFailed)?;

        Ok(paper)
    }

    pub fn import_pdf(
        &mut self,
        bytes: Vec<u8>,
        title: Option<String>,
        extractor: &dyn pdf_bridge::PdfTextBridge,
    ) -> Result<Paper, WebImportError> {
        let text = extractor
            .extract_text(&bytes)
            .map_err(|_| WebImportError::ExtractionFailed)?;

        if text.trim().is_empty() {
            return Err(WebImportError::ExtractionFailed);
        }

        let paper_id = self.next_paper_id();
        let now = current_time_ms();

        let paper = Paper {
            id: paper_id.clone(),
            title,
            source_type: PaperSourceType::Pdf,
            source_ref: None,
            status: PaperStatus::ReadyText,
            parse_version: 1,
        };

        let blocks = segment_text(&paper, &text)?;
        let progress = ReadingProgress {
            paper_id: paper_id.clone(),
            current_block_index: 0,
            current_block_id: Some(blocks[0].id.clone()),
            updated_at_ms: now,
        };

        let stored = StoredPaper {
            paper: paper.clone(),
            created_at_ms: now,
            updated_at_ms: now,
        };

        self.store
            .save_paper(&stored)
            .map_err(|_| WebImportError::StorageFailed)?;
        self.store
            .save_blocks(&blocks)
            .map_err(|_| WebImportError::StorageFailed)?;
        self.store
            .save_progress(&progress)
            .map_err(|_| WebImportError::StorageFailed)?;

        Ok(paper)
    }

    pub fn list_papers(&self) -> Result<Vec<StoredPaper>, WebImportError> {
        self.store
            .list_papers()
            .map_err(|_| WebImportError::StorageFailed)
    }

    pub fn into_store(self) -> S {
        self.store
    }
}

fn segment_text(paper: &Paper, text: &str) -> Result<Vec<Block>, WebImportError> {
    let segments: Vec<String> = text
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if segments.is_empty() {
        return Err(WebImportError::EmptyInput);
    }

    segments
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            Block::try_new(
                BlockId(format!("block-{}", index)),
                paper.id.clone(),
                index as u32,
                segment,
                None,
            )
            .map_err(|_| WebImportError::ExtractionFailed)
        })
        .collect()
}
