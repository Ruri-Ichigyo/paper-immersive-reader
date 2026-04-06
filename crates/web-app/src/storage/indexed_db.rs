use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use paper_immersive_reader_core::domain::{
    Annotation, Paper, PaperId, ReadingProgress, VocabularyItem,
};

use super::annotations::AnnotationRecords;
use super::papers::PaperRecords;
use super::schema::{AiRecord, PaperBundle, ReaderSettings};
use super::settings::SettingsRecords;
use super::vocabulary::VocabularyRecords;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    StatePoisoned,
}

#[derive(Debug, Clone, Default)]
pub struct WebStorage {
    state: Arc<Mutex<StorageState>>,
}

#[derive(Debug, Clone, Default)]
struct StorageState {
    papers: PaperRecords,
    annotations: AnnotationRecords,
    vocabulary: VocabularyRecords,
    settings: SettingsRecords,
    ai_records_by_paper: BTreeMap<String, Vec<AiRecord>>,
}

impl WebStorage {
    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn save_paper_bundle(
        &self,
        paper: Paper,
        blocks: Vec<paper_immersive_reader_core::domain::Block>,
        reading_progress: ReadingProgress,
    ) -> Result<(), StorageError> {
        self.lock_state()?.papers.save_bundle(PaperBundle {
            paper,
            blocks,
            reading_progress,
        });
        Ok(())
    }

    pub fn load_paper_bundle(
        &self,
        paper_id: &PaperId,
    ) -> Result<Option<PaperBundle>, StorageError> {
        Ok(self.lock_state()?.papers.load_bundle(paper_id))
    }

    pub fn list_papers(&self) -> Result<Vec<Paper>, StorageError> {
        Ok(self.lock_state()?.papers.list_papers())
    }

    pub fn save_pdf_asset(&self, paper_id: &PaperId, bytes: Vec<u8>) -> Result<(), StorageError> {
        self.lock_state()?.papers.save_pdf_asset(paper_id, bytes);
        Ok(())
    }

    pub fn load_pdf_asset(&self, paper_id: &PaperId) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.lock_state()?.papers.load_pdf_asset(paper_id))
    }

    pub fn save_annotation(&self, annotation: Annotation) -> Result<(), StorageError> {
        self.lock_state()?.annotations.save(annotation);
        Ok(())
    }

    pub fn list_annotations_for_paper(
        &self,
        paper_id: &PaperId,
    ) -> Result<Vec<Annotation>, StorageError> {
        Ok(self.lock_state()?.annotations.list_for_paper(paper_id))
    }

    pub fn upsert_vocabulary_lookup(
        &self,
        item: VocabularyItem,
    ) -> Result<VocabularyItem, StorageError> {
        Ok(self.lock_state()?.vocabulary.upsert_lookup(item))
    }

    pub fn list_vocabulary_for_paper(
        &self,
        paper_id: &PaperId,
    ) -> Result<Vec<VocabularyItem>, StorageError> {
        Ok(self.lock_state()?.vocabulary.list_for_paper(paper_id))
    }

    pub fn save_settings(&self, settings: ReaderSettings) -> Result<(), StorageError> {
        self.lock_state()?.settings.save(settings);
        Ok(())
    }

    pub fn load_settings(&self) -> Result<ReaderSettings, StorageError> {
        Ok(self.lock_state()?.settings.load())
    }

    pub fn save_ai_record(&self, record: AiRecord) -> Result<(), StorageError> {
        self.lock_state()?
            .ai_records_by_paper
            .entry(record.paper_id.0.clone())
            .or_default()
            .push(record);
        Ok(())
    }

    pub fn list_ai_records_for_paper(
        &self,
        paper_id: &PaperId,
    ) -> Result<Vec<AiRecord>, StorageError> {
        Ok(self
            .lock_state()?
            .ai_records_by_paper
            .get(&paper_id.0)
            .cloned()
            .unwrap_or_default())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, StorageState>, StorageError> {
        self.state.lock().map_err(|_| StorageError::StatePoisoned)
    }
}
