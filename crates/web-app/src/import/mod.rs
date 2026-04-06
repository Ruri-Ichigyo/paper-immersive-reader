mod paste;
mod pdf;
mod pdf_bridge;

use std::sync::{Arc, Mutex};

use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{segment_text_for_existing_paper, ImportPipelineError};

use crate::storage::{StorageError, WebStorage};

pub use paste::PasteImportRequest;
pub use pdf::PdfImportRequest;
pub use pdf_bridge::{PdfTextBridge, UnavailablePdfTextBridge};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebImportResult {
    pub paper_id: PaperId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebImportError {
    ImportPipeline(ImportPipelineError),
    Storage(StorageError),
    PdfBridgeUnavailable,
}

impl From<ImportPipelineError> for WebImportError {
    fn from(value: ImportPipelineError) -> Self {
        Self::ImportPipeline(value)
    }
}

impl From<StorageError> for WebImportError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

pub struct WebImportService<B = UnavailablePdfTextBridge> {
    storage: WebStorage,
    pdf_bridge: B,
    next_paper_sequence: Arc<Mutex<u64>>,
}

impl WebImportService<UnavailablePdfTextBridge> {
    pub fn new(storage: WebStorage) -> Self {
        Self::new_with_pdf_bridge(storage, UnavailablePdfTextBridge)
    }
}

impl<B> WebImportService<B>
where
    B: PdfTextBridge,
{
    pub fn new_with_pdf_bridge(storage: WebStorage, pdf_bridge: B) -> Self {
        Self {
            storage,
            pdf_bridge,
            next_paper_sequence: Arc::new(Mutex::new(1)),
        }
    }

    pub fn import_paste_text(
        &self,
        request: PasteImportRequest,
    ) -> Result<WebImportResult, WebImportError> {
        let paper = self.build_paper(request.title, request.source_ref, PaperSourceType::Paste)?;
        let artifacts =
            segment_text_for_existing_paper(&paper, request.text, request.imported_at_ms)?;

        self.storage.save_paper_bundle(
            paper.clone(),
            artifacts.blocks,
            artifacts.reading_progress,
        )?;

        Ok(WebImportResult { paper_id: paper.id })
    }

    pub fn import_pdf_document(
        &self,
        request: PdfImportRequest,
    ) -> Result<WebImportResult, WebImportError> {
        let text = self.pdf_bridge.extract_text(&request.bytes)?;
        let paper = self.build_paper(request.title, request.source_ref, PaperSourceType::Pdf)?;
        let artifacts = segment_text_for_existing_paper(&paper, text, request.imported_at_ms)?;

        self.storage.save_pdf_asset(&paper.id, request.bytes)?;
        self.storage.save_paper_bundle(
            paper.clone(),
            artifacts.blocks,
            artifacts.reading_progress,
        )?;

        Ok(WebImportResult { paper_id: paper.id })
    }

    fn build_paper(
        &self,
        title: Option<String>,
        source_ref: Option<String>,
        source_type: PaperSourceType,
    ) -> Result<Paper, WebImportError> {
        let paper_id = self.next_paper_id()?;

        Ok(Paper {
            id: PaperId(paper_id),
            title,
            source_type,
            source_ref,
            status: PaperStatus::ReadyText,
            parse_version: 1,
        })
    }

    fn next_paper_id(&self) -> Result<String, WebImportError> {
        let mut sequence = self
            .next_paper_sequence
            .lock()
            .map_err(|_| WebImportError::Storage(StorageError::StatePoisoned))?;
        let paper_id = format!("web-paper-{}", *sequence);
        *sequence += 1;
        Ok(paper_id)
    }
}
