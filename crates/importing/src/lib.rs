use paper_immersive_reader_core::domain::{
    Block, BlockId, DomainError, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

pub enum ImportRequest {
    PasteText {
        text: String,
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
    Domain(DomainError),
}

pub trait ImportPipeline {
    fn import(&self, request: ImportRequest) -> Result<ImportResult, ImportPipelineError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultImportPipeline {
    next_paper_id: String,
}

impl DefaultImportPipeline {
    pub fn new_for_tests() -> Self {
        Self {
            next_paper_id: "paper-1".to_string(),
        }
    }
}

impl ImportPipeline for DefaultImportPipeline {
    fn import(&self, request: ImportRequest) -> Result<ImportResult, ImportPipelineError> {
        match request {
            ImportRequest::PasteText {
                text,
                title,
                source_ref,
            } => {
                if text.trim().is_empty() {
                    return Err(ImportPipelineError::EmptyInput);
                }

                let paper = Paper {
                    id: PaperId(self.next_paper_id.clone()),
                    title,
                    source_type: PaperSourceType::Paste,
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
    }
}
