use paper_immersive_reader_core::domain::{
    Annotation, AnnotationId, AnnotationType, BlockId, PaperId,
};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

fn make_annotation(id: &str, paper_id: &str, quote: &str) -> Annotation {
    Annotation {
        id: AnnotationId(id.to_string()),
        paper_id: PaperId(paper_id.to_string()),
        block_id: BlockId("block-1".to_string()),
        annotation_type: AnnotationType::Highlight,
        quote: quote.to_string(),
        prefix: None,
        suffix: None,
        start_offset: Some(0),
        end_offset: Some(quote.len() as u32),
        note: None,
        tags: vec![],
        anchoring_status: paper_immersive_reader_core::domain::AnchoringStatus::Anchored,
    }
}

#[test]
fn save_annotation_upsert_replaces_existing() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("paper-1".to_string());

    let ann1 = make_annotation("ann-1", "paper-1", "first quote");
    store.save_annotation(&ann1).unwrap();

    let anns = store.load_annotations(&pid).unwrap();
    assert_eq!(anns.len(), 1);
    assert_eq!(anns[0].quote, "first quote");

    // Save again with same ID but different quote -> should replace, not append
    let ann1_updated = make_annotation("ann-1", "paper-1", "updated quote");
    store.save_annotation(&ann1_updated).unwrap();

    let anns = store.load_annotations(&pid).unwrap();
    assert_eq!(anns.len(), 1, "upsert should not create duplicate");
    assert_eq!(anns[0].quote, "updated quote");
}

#[test]
fn save_annotation_different_ids_both_persisted() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("paper-1".to_string());

    store
        .save_annotation(&make_annotation("ann-1", "paper-1", "first"))
        .unwrap();
    store
        .save_annotation(&make_annotation("ann-2", "paper-1", "second"))
        .unwrap();

    let anns = store.load_annotations(&pid).unwrap();
    assert_eq!(anns.len(), 2);
}

#[test]
fn save_annotation_then_update_preserves_count() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("paper-1".to_string());

    store
        .save_annotation(&make_annotation("ann-1", "paper-1", "original"))
        .unwrap();
    store
        .save_annotation(&make_annotation("ann-2", "paper-1", "other"))
        .unwrap();

    // update_annotation should also not create duplicates
    let mut ann1 = make_annotation("ann-1", "paper-1", "via update");
    ann1.note = Some("added note".to_string());
    store.update_annotation(&ann1).unwrap();

    let anns = store.load_annotations(&pid).unwrap();
    assert_eq!(anns.len(), 2);
    let ann1_loaded = anns.iter().find(|a| a.id.0 == "ann-1").unwrap();
    assert_eq!(ann1_loaded.quote, "via update");
    assert_eq!(ann1_loaded.note, Some("added note".to_string()));
}
