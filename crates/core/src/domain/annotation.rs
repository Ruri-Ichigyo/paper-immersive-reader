use super::block::BlockId;
use super::paper::PaperId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationType {
    Highlight,
    Underline,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchoringStatus {
    Anchored,
    Unanchored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub paper_id: PaperId,
    pub block_id: BlockId,
    pub annotation_type: AnnotationType,
    pub quote: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub start_offset: Option<u32>,
    pub end_offset: Option<u32>,
    pub note: Option<String>,
    pub tags: Vec<String>,
    pub anchoring_status: AnchoringStatus,
}
