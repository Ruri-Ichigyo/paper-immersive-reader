pub mod pdf_bridge;

use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

use crate::storage::schema::StoredPaper;
use crate::storage::AppStorage;
use crate::util::{current_time_ms, short_id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebImportError {
    EmptyInput,
    ExtractionFailed,
    StorageFailed,
}

#[cfg(not(target_arch = "wasm32"))]
mod native_import_bridge {
    use super::WebImportError;
    use paper_immersive_reader_importing::ImportPipelineError;

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
}

pub struct WebImportService<S: AppStorage> {
    store: S,
}

impl<S: AppStorage> WebImportService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    fn next_paper_id(&self) -> PaperId {
        PaperId(short_id("paper"))
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

    /// Native (non-wasm32) PDF import using a sync `PdfTextBridge`.
    #[cfg(not(target_arch = "wasm32"))]
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

    /// WASM (browser) PDF import using the async pdf.js bridge.
    #[cfg(target_arch = "wasm32")]
    pub async fn import_pdf_async(
        &mut self,
        bytes: Vec<u8>,
        title: Option<String>,
        bridge: &pdf_bridge::WasmPdfBridge,
    ) -> Result<Paper, WebImportError> {
        let text = bridge
            .extract_text_async(&bytes)
            .await
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

/// Filter repeated header/footer lines from multi-page extracted text.
///
/// pdf.js separates pages with `\x0c` (form-feed).  When there are 3+
/// pages, lines that appear as the first or last line of > 50% of pages
/// are treated as headers/footers and stripped.
fn filter_repeated_lines(text: &str) -> String {
    use std::collections::{HashMap, HashSet};

    let pages: Vec<&str> = text.split('\x0c').collect();
    if pages.len() < 3 {
        return text.to_string();
    }

    let mut line_counts: HashMap<String, usize> = HashMap::new();
    let threshold = pages.len() / 2;

    for page in &pages {
        let lines: Vec<&str> = page.lines().filter(|l| !l.trim().is_empty()).collect();
        if let Some(first) = lines.first() {
            *line_counts.entry(first.trim().to_string()).or_insert(0) += 1;
        }
        if let Some(last) = lines.last() {
            *line_counts.entry(last.trim().to_string()).or_insert(0) += 1;
        }
    }

    let noise_lines: HashSet<&str> = line_counts
        .iter()
        .filter(|(_, &count)| count > threshold)
        .map(|(line, _)| line.as_str())
        .collect();

    pages
        .iter()
        .map(|page| {
            page.lines()
                .filter(|line| !noise_lines.contains(line.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\x0c\n")
}

fn segment_text(paper: &Paper, text: &str) -> Result<Vec<Block>, WebImportError> {
    let cleaned = filter_repeated_lines(text);

    let mut segments: Vec<String> = cleaned
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    // Fallback: if only 0-1 segments from double-newline split,
    // try splitting on single newlines for lines > 20 chars.
    // This catches PDFs where paragraphs are separated by a single newline.
    if segments.len() <= 1 {
        segments = cleaned
            .split('\n')
            .map(str::trim)
            .filter(|s| s.len() > 20)
            .map(ToOwned::to_owned)
            .collect();
    }

    if segments.is_empty() {
        return Err(WebImportError::EmptyInput);
    }

    segments
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            Block::try_new(
                BlockId(short_id("block")),
                paper.id.clone(),
                index as u32,
                segment,
                None,
            )
            .map_err(|_| WebImportError::ExtractionFailed)
        })
        .collect()
}
