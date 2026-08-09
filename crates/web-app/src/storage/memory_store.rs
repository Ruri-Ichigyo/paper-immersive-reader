use std::collections::HashMap;

use paper_immersive_reader_core::domain::{
    Annotation, AnnotationId, Block, PaperId, PaperStatus, ReadingProgress, VocabularyItem,
};

use super::schema::{AiRecord, ReadingStats, Settings, StoredPaper};
use super::{AppStorage, StorageError};

#[derive(Debug, Clone, Default)]
pub struct InMemoryStore {
    papers: HashMap<String, StoredPaper>,
    blocks: HashMap<String, Vec<Block>>,
    progress: HashMap<String, ReadingProgress>,
    annotations: HashMap<String, Vec<Annotation>>,
    vocabulary: HashMap<String, VocabularyItem>,
    ai_records: HashMap<String, Vec<AiRecord>>,
    reading_stats: HashMap<String, ReadingStats>,
    settings: Option<Settings>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AppStorage for InMemoryStore {
    fn save_paper(&mut self, paper: &StoredPaper) -> Result<(), StorageError> {
        self.papers.insert(paper.paper.id.0.clone(), paper.clone());
        Ok(())
    }

    fn load_paper(&self, id: &PaperId) -> Result<Option<StoredPaper>, StorageError> {
        Ok(self.papers.get(&id.0).cloned())
    }

    fn list_papers(&self) -> Result<Vec<StoredPaper>, StorageError> {
        Ok(self.papers.values().cloned().collect())
    }

    fn update_paper_status(
        &mut self,
        id: &PaperId,
        status: PaperStatus,
    ) -> Result<(), StorageError> {
        match self.papers.get_mut(&id.0) {
            Some(stored) => {
                stored.paper.status = status;
                stored.updated_at_ms = current_time_ms();
                Ok(())
            }
            None => Err(StorageError::NotFound),
        }
    }

    fn delete_paper(&mut self, id: &PaperId) -> Result<(), StorageError> {
        if self.papers.remove(&id.0).is_none() {
            return Err(StorageError::NotFound);
        }
        // Cascade delete related data
        self.blocks.remove(&id.0);
        self.progress.remove(&id.0);
        self.annotations.remove(&id.0);
        self.ai_records.remove(&id.0);
        self.reading_stats.remove(&id.0);
        Ok(())
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
        let list = self.annotations.entry(key).or_default();
        if let Some(existing) = list.iter_mut().find(|a| a.id == annotation.id) {
            *existing = annotation.clone();
        } else {
            list.push(annotation.clone());
        }
        Ok(())
    }

    fn load_annotations(&self, paper_id: &PaperId) -> Result<Vec<Annotation>, StorageError> {
        Ok(self
            .annotations
            .get(&paper_id.0)
            .cloned()
            .unwrap_or_default())
    }

    fn load_annotation(&self, id: &AnnotationId) -> Result<Option<Annotation>, StorageError> {
        for list in self.annotations.values() {
            for a in list {
                if a.id == *id {
                    return Ok(Some(a.clone()));
                }
            }
        }
        Ok(None)
    }

    fn delete_annotation(&mut self, id: &AnnotationId) -> Result<(), StorageError> {
        for list in self.annotations.values_mut() {
            if let Some(pos) = list.iter().position(|a| a.id == *id) {
                list.remove(pos);
                return Ok(());
            }
        }
        Err(StorageError::NotFound)
    }

    fn update_annotation(&mut self, annotation: &Annotation) -> Result<(), StorageError> {
        for list in self.annotations.values_mut() {
            if let Some(existing) = list.iter_mut().find(|a| a.id == annotation.id) {
                *existing = annotation.clone();
                return Ok(());
            }
        }
        Err(StorageError::NotFound)
    }

    fn upsert_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError> {
        let key = item.term_lower.clone();
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

    fn update_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError> {
        let key = item.term_lower.clone();
        self.vocabulary.insert(key, item.clone());
        Ok(())
    }

    fn list_vocabulary(&self) -> Result<Vec<VocabularyItem>, StorageError> {
        Ok(self.vocabulary.values().cloned().collect())
    }

    fn save_ai_record(&mut self, record: &AiRecord) -> Result<(), StorageError> {
        let key = record.paper_id.0.clone();
        let list = self.ai_records.entry(key).or_default();
        if let Some(existing) = list.iter_mut().find(|r| r.id == record.id) {
            *existing = record.clone();
        } else {
            list.push(record.clone());
        }
        Ok(())
    }

    fn load_ai_records(&self, paper_id: &PaperId) -> Result<Vec<AiRecord>, StorageError> {
        Ok(self
            .ai_records
            .get(&paper_id.0)
            .cloned()
            .unwrap_or_default())
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError> {
        self.settings = Some(settings.clone());
        Ok(())
    }

    fn load_settings(&self) -> Result<Settings, StorageError> {
        Ok(self.settings.clone().unwrap_or_default())
    }

    fn save_reading_stats(&mut self, stats: &ReadingStats) -> Result<(), StorageError> {
        self.reading_stats
            .insert(stats.paper_id.0.clone(), stats.clone());
        Ok(())
    }

    fn load_reading_stats(&self, paper_id: &PaperId) -> Result<Option<ReadingStats>, StorageError> {
        Ok(self.reading_stats.get(&paper_id.0).cloned())
    }

    fn list_reading_stats(&self) -> Result<Vec<ReadingStats>, StorageError> {
        Ok(self.reading_stats.values().cloned().collect())
    }
}

fn current_time_ms() -> u64 {
    crate::util::current_time_ms()
}
