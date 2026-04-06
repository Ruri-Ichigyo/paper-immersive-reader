use super::{BlockId, PaperId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyItem {
    pub id: VocabularyId,
    pub term: String,
    pub paper_id_first_seen: PaperId,
    pub block_id_first_seen: Option<BlockId>,
    pub lookup_count: u32,
    pub first_seen_at_ms: u64,
    pub last_seen_at_ms: u64,
    pub note: Option<String>,
    pub learned: bool,
}
