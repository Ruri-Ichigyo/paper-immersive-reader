mod annotations;
mod indexed_db;
mod papers;
mod schema;
mod settings;
mod vocabulary;

pub use indexed_db::{StorageError, WebStorage};
pub use schema::{AiRecord, AiRecordId, AiRecordType, PaperBundle, ReaderSettings};
