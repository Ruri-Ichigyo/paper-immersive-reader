use paper_immersive_reader_core::domain::{Annotation, Block, VocabularyItem};
use serde::Serialize;

use crate::storage::schema::{ReadingStats, StoredPaper};

/// Export bundle matching PRD FR-35/FR-36:
/// - Includes: original text (blocks), annotations, vocabulary
/// - Excludes: translations, term explanations, AI Q&A logs (ai_records)
#[derive(Debug, Clone, Serialize)]
pub struct ExportBundle {
    pub paper: StoredPaper,
    pub blocks: Vec<Block>,
    pub annotations: Vec<Annotation>,
    pub vocabulary: Vec<VocabularyItem>,
    pub reading_stats: Option<ReadingStats>,
}

impl ExportBundle {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Generate a filename for the export.
    pub fn filename(&self) -> String {
        let title = self
            .paper
            .paper
            .title
            .as_deref()
            .unwrap_or("untitled")
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .take(60)
            .collect::<String>();
        format!("{}_export.json", title)
    }
}
