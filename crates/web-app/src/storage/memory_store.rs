use std::collections::HashMap;

use paper_immersive_reader_core::domain::{
    Annotation, AnnotationId, Block, PaperId, PaperStatus, ReadingProgress,
    VocabularyItem,
};

use super::schema::{AiRecord, Settings, StoredPaper};
use super::{AppStorage, StorageError};

#[derive(Debug, Clone, Default)]
pub struct InMemoryStore {
    papers: HashMap<String, StoredPaper>,
    blocks: HashMap<String, Vec<Block>>,
    progress: HashMap<String, ReadingProgress>,
    annotations: HashMap<String, Vec<Annotation>>,
    vocabulary: HashMap<String, VocabularyItem>,
    ai_records: HashMap<String, Vec<AiRecord>>,
    settings: Option<Settings>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AppStorage for InMemoryStore {
    fn save_paper(&mut self, paper: &StoredPaper) -> Result<(), StorageError> {
        self.papers
            .insert(paper.paper.id.0.clone(), paper.clone());
        Ok(())
    }

    fn load_paper(&self, id: &PaperId) -> Result<Option<StoredPaper>, StorageError> {
        Ok(self.papers.get(&id.0).cloned())
    }

    fn list_papers(&self) -> Result<Vec<StoredPaper>, StorageError> {
        Ok(self.papers.values().cloned().collect())
    }

    fn update_paper_status(&mut self, id: &PaperId, status: PaperStatus) -> Result<(), StorageError> {
        match self.papers.get_mut(&id.0) {
            Some(stored) => {
                stored.paper.status = status;
                stored.updated_at_ms = current_time_ms();
                Ok(())
            }
            None => Err(StorageError::NotFound),
        }
    }

    fn save_blocks(&mut self, blocks: &[Block]) -> Result<(), StorageError> {
        if blocks.is_empty() {
            return Ok(());
        }
        let paper_id = blocks[0].paper_id.0.clone();
        self.blocks.insert(paper_id, blocks.to_vec());
        Ok(())
    }

    fn load_blocks(&self, paper_id: &PaperId) -> Result<Vec<Block>, StorageError> {
        Ok(self.blocks.get(&paper_id.0).cloned().unwrap_or_default())
    }

    fn save_progress(&mut self, progress: &ReadingProgress) -> Result<(), StorageError> {
        self.progress
            .insert(progress.paper_id.0.clone(), progress.clone());
        Ok(())
    }

    fn load_progress(&self, paper_id: &PaperId) -> Result<Option<ReadingProgress>, StorageError> {
        Ok(self.progress.get(&paper_id.0).cloned())
    }

    fn save_annotation(&mut self, annotation: &Annotation) -> Result<(), StorageError> {
        let key = annotation.paper_id.0.clone();
        self.annotations
            .entry(key)
            .or_default()
            .push(annotation.clone());
        Ok(())
    }

    fn load_annotations(&self, paper_id: &PaperId) -> Result<Vec<Annotation>, StorageError> {
        Ok(self.annotations.get(&paper_id.0).cloned().unwrap_or_default())
    }

    fn load_annotation(
        &self,
        id: &AnnotationId,
    ) -> Result<Option<Annotation>, StorageError> {
        for list in self.annotations.values() {
            for a in list {
                if a.id == *id {
                    return Ok(Some(a.clone()));
                }
            }
        }
        Ok(None)
    }

    fn upsert_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError> {
        let key = item.term.to_lowercase();
        match self.vocabulary.get_mut(&key) {
            Some(existing) => {
                existing.lookup_count += 1;
                existing.last_seen_at_ms = item.last_seen_at_ms;
                if item.note.is_some() {
                    existing.note = item.note.clone();
                }
            }
            None => {
                self.vocabulary.insert(key, item.clone());
            }
        }
        Ok(())
    }

    fn list_vocabulary(&self) -> Result<Vec<VocabularyItem>, StorageError> {
        Ok(self.vocabulary.values().cloned().collect())
    }

    fn save_ai_record(&mut self, record: &AiRecord) -> Result<(), StorageError> {
        let key = record.paper_id.0.clone();
        self.ai_records.entry(key).or_default().push(record.clone());
        Ok(())
    }

    fn load_ai_records(&self, paper_id: &PaperId) -> Result<Vec<AiRecord>, StorageError> {
        Ok(self.ai_records.get(&paper_id.0).cloned().unwrap_or_default())
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError> {
        self.settings = Some(settings.clone());
        Ok(())
    }

    fn load_settings(&self) -> Result<Settings, StorageError> {
        Ok(self.settings.clone().unwrap_or_default())
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
