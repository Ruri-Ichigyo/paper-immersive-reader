pub mod annotation;
pub mod block;
pub mod error;
pub mod ocr_job;
pub mod paper;
pub mod reading_progress;
pub mod vocabulary;

pub use annotation::{AnchoringStatus, Annotation, AnnotationId, AnnotationType};
pub use block::{Block, BlockId};
pub use error::DomainError;
pub use ocr_job::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus,
    OcrJobTransitionError,
};
pub use paper::{Paper, PaperId, PaperSourceType, PaperStatus};
pub use reading_progress::ReadingProgress;
pub use vocabulary::{VocabularyId, VocabularyItem};
