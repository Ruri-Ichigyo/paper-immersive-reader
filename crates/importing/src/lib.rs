use std::sync::Arc;

use paper_immersive_reader_core::domain::{
    Block, BlockId, DomainError, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

pub enum ImportRequest {
    PasteText {
        text: String,
        title: Option<String>,
        source_ref: Option<String>,
    },
    PdfDocument {
        bytes: Vec<u8>,
        title: Option<String>,
        source_ref: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResult {
    pub paper: Paper,
    pub blocks: Vec<Block>,
    pub reading_progress: ReadingProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportPipelineError {
    EmptyInput,
    NoBlocksProduced,
    PdfExtractionFailed,
    OcrRequired,
    Domain(DomainError),
}

pub trait ImportPipeline {
    fn import(&self, request: ImportRequest) -> Result<ImportResult, ImportPipelineError>;
}

pub trait PdfTextExtractor: Send + Sync {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, ImportPipelineError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PdfExtractorMode {
    Unavailable,
    Deterministic(String),
    Fail,
}

impl PdfTextExtractor for PdfExtractorMode {
    fn extract_text(&self, _bytes: &[u8]) -> Result<String, ImportPipelineError> {
        match self {
            Self::Unavailable => Err(ImportPipelineError::PdfExtractionFailed),
            Self::Deterministic(text) => Ok(text.clone()),
            Self::Fail => Err(ImportPipelineError::EmptyInput),
        }
    }
}

pub struct DefaultImportPipeline {
    next_paper_id: String,
    pdf_text_extractor: Arc<dyn PdfTextExtractor>,
}

impl DefaultImportPipeline {
    pub fn new_for_tests() -> Self {
        Self::new_with_pdf_extractor(PdfExtractorMode::Unavailable)
    }

    pub fn new_for_tests_with_pdf_text(text: &str) -> Self {
        Self::new_with_pdf_extractor(PdfExtractorMode::Deterministic(text.to_string()))
    }

    pub fn new_for_tests_with_pdf_failure() -> Self {
        Self::new_with_pdf_extractor(PdfExtractorMode::Fail)
    }

    pub fn new_with_pdf_extractor<T>(pdf_text_extractor: T) -> Self
    where
        T: PdfTextExtractor + 'static,
    {
        Self {
            next_paper_id: "paper-1".to_string(),
            pdf_text_extractor: Arc::new(pdf_text_extractor),
        }
    }

    fn import_text(
        &self,
        text: String,
        title: Option<String>,
        source_ref: Option<String>,
        source_type: PaperSourceType,
    ) -> Result<ImportResult, ImportPipelineError> {
        if text.trim().is_empty() {
            return match source_type {
                PaperSourceType::Pdf => Err(ImportPipelineError::OcrRequired),
                _ => Err(ImportPipelineError::EmptyInput),
            };
        }

        let paper = Paper {
            id: PaperId(self.next_paper_id.clone()),
            title,
            source_type,
            source_ref,
            status: PaperStatus::ReadyText,
            parse_version: 1,
        };

        let segments: Vec<String> = text
            .split("\n\n")
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        if segments.is_empty() {
            return Err(ImportPipelineError::NoBlocksProduced);
        }

        let blocks: Vec<Block> = segments
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
                .map_err(ImportPipelineError::Domain)
            })
            .collect::<Result<_, _>>()?;

        let reading_progress = ReadingProgress {
            paper_id: paper.id.clone(),
            current_block_index: 0,
            current_block_id: Some(blocks[0].id.clone()),
            updated_at_ms: 0,
        };

        Ok(ImportResult {
            paper,
            blocks,
            reading_progress,
        })
    }
}

impl ImportPipeline for DefaultImportPipeline {
    fn import(&self, request: ImportRequest) -> Result<ImportResult, ImportPipelineError> {
        match request {
            ImportRequest::PasteText {
                text,
                title,
                source_ref,
            } => self.import_text(text, title, source_ref, PaperSourceType::Paste),
            ImportRequest::PdfDocument {
                bytes,
                title,
                source_ref,
            } => {
                let text = self
                    .pdf_text_extractor
                    .extract_text(&bytes)
                    .map_err(|_| ImportPipelineError::PdfExtractionFailed)?;
                self.import_text(text, title, source_ref, PaperSourceType::Pdf)
            }
        }
    }
}
