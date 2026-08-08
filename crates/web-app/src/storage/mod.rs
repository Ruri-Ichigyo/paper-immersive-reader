#[cfg(target_arch = "wasm32")]
pub mod indexed_db_store;
pub mod memory_store;
pub mod schema;

use paper_immersive_reader_core::domain::{
    Annotation, AnnotationId, Block, PaperId, ReadingProgress, VocabularyItem,
};
use schema::{AiRecord, ReadingStats, Settings, StoredPaper};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    NotFound,
    Duplicate,
    IoError(String),
}

pub trait AppStorage {
    // Papers
    fn save_paper(&mut self, paper: &StoredPaper) -> Result<(), StorageError>;
    fn load_paper(&self, id: &PaperId) -> Result<Option<StoredPaper>, StorageError>;
    fn list_papers(&self) -> Result<Vec<StoredPaper>, StorageError>;
    fn update_paper_status(
        &mut self,
        id: &PaperId,
        status: paper_immersive_reader_core::domain::PaperStatus,
    ) -> Result<(), StorageError>;
    fn delete_paper(&mut self, id: &PaperId) -> Result<(), StorageError>;

    // Blocks
    fn save_blocks(&mut self, blocks: &[Block]) -> Result<(), StorageError>;
    fn load_blocks(&self, paper_id: &PaperId) -> Result<Vec<Block>, StorageError>;

    // Reading progress
    fn save_progress(&mut self, progress: &ReadingProgress) -> Result<(), StorageError>;
    fn load_progress(&self, paper_id: &PaperId) -> Result<Option<ReadingProgress>, StorageError>;

    // Annotations
    fn save_annotation(&mut self, annotation: &Annotation) -> Result<(), StorageError>;
    fn load_annotations(&self, paper_id: &PaperId) -> Result<Vec<Annotation>, StorageError>;
    fn load_annotation(&self, id: &AnnotationId) -> Result<Option<Annotation>, StorageError>;
    fn delete_annotation(&mut self, id: &AnnotationId) -> Result<(), StorageError>;
    fn update_annotation(&mut self, annotation: &Annotation) -> Result<(), StorageError>;

    // Vocabulary
    fn upsert_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError>;
    fn update_vocabulary(&mut self, item: &VocabularyItem) -> Result<(), StorageError>;
    fn list_vocabulary(&self) -> Result<Vec<VocabularyItem>, StorageError>;

    // AI records
    fn save_ai_record(&mut self, record: &AiRecord) -> Result<(), StorageError>;
    fn load_ai_records(&self, paper_id: &PaperId) -> Result<Vec<AiRecord>, StorageError>;

    // Settings
    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError>;
    fn load_settings(&self) -> Result<Settings, StorageError>;

    // Reading statistics
    fn save_reading_stats(&mut self, stats: &ReadingStats) -> Result<(), StorageError>;
    fn load_reading_stats(&self, paper_id: &PaperId) -> Result<Option<ReadingStats>, StorageError>;
    fn list_reading_stats(&self) -> Result<Vec<ReadingStats>, StorageError>;
}
