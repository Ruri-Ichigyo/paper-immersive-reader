use paper_immersive_reader_core::domain::{Annotation, AnnotationId, BlockId, PaperId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedAnnotation {
    pub annotation_id: AnnotationId,
    pub paper_id: PaperId,
    pub block_id: BlockId,
    pub quote: String,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

impl From<Annotation> for RenderedAnnotation {
    fn from(value: Annotation) -> Self {
        Self {
            annotation_id: value.id,
            paper_id: value.paper_id,
            block_id: value.block_id,
            quote: value.quote,
            note: value.note,
            tags: value.tags,
        }
    }
}
