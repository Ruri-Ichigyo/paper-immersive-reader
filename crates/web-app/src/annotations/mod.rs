pub mod anchors;

use paper_immersive_reader_core::domain::{
    AnchoringStatus, Annotation, AnnotationId, AnnotationType, BlockId, PaperId,
};

use crate::selection::TextSelection;
use crate::storage::AppStorage;
use crate::util::short_id;

use self::anchors::compute_anchors;

#[derive(Debug, Clone)]
pub struct AnnotationInput {
    pub annotation_type: AnnotationType,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

pub fn create_annotation(
    paper_id: PaperId,
    block_id: BlockId,
    block_text: &str,
    selection: &TextSelection,
    input: AnnotationInput,
) -> Annotation {
    let (prefix, suffix) = compute_anchors(block_text, selection);
    let anchoring_status = if prefix.is_some() || suffix.is_some() {
        AnchoringStatus::Anchored
    } else {
        AnchoringStatus::Unanchored
    };

    Annotation {
        id: AnnotationId(short_id("ann")),
        paper_id,
        block_id,
        annotation_type: input.annotation_type,
        quote: selection.text.clone(),
        prefix,
        suffix,
        start_offset: Some(selection.start_offset),
        end_offset: Some(selection.end_offset),
        note: input.note,
        tags: input.tags,
        anchoring_status,
    }
}

pub fn save_and_reload_annotation<S: AppStorage>(
    store: &mut S,
    annotation: &Annotation,
) -> Result<bool, crate::storage::StorageError> {
    store.save_annotation(annotation)?;
    let reloaded = store.load_annotation(&annotation.id)?;
    Ok(reloaded.is_some())
}
