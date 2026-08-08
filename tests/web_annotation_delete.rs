use paper_immersive_reader_core::domain::{
    AnchoringStatus, Annotation, AnnotationId, AnnotationType, BlockId, PaperId,
};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

fn make_annotation(id: &str) -> Annotation {
    Annotation {
        id: AnnotationId(id.to_string()),
        paper_id: PaperId("p-1".to_string()),
        block_id: BlockId("b-1".to_string()),
        annotation_type: AnnotationType::Highlight,
        quote: "some quoted text".to_string(),
        prefix: None,
        suffix: None,
        start_offset: None,
        end_offset: None,
        note: None,
        tags: vec![],
        anchoring_status: AnchoringStatus::Anchored,
    }
}

#[test]
fn delete_annotation_removes_it() {
    let mut store = InMemoryStore::new();
    let ann = make_annotation("ann-1");
    store.save_annotation(&ann).unwrap();
    assert_eq!(
        store
            .load_annotations(&PaperId("p-1".to_string()))
            .unwrap()
            .len(),
        1
    );

    store
        .delete_annotation(&AnnotationId("ann-1".to_string()))
        .unwrap();
    assert_eq!(
        store
            .load_annotations(&PaperId("p-1".to_string()))
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn delete_annotation_not_found() {
    let mut store = InMemoryStore::new();
    let result = store.delete_annotation(&AnnotationId("missing".to_string()));
    assert!(result.is_err());
}

#[test]
fn delete_annotation_only_removes_target() {
    let mut store = InMemoryStore::new();
    store.save_annotation(&make_annotation("ann-1")).unwrap();
    store.save_annotation(&make_annotation("ann-2")).unwrap();
    store.save_annotation(&make_annotation("ann-3")).unwrap();

    store
        .delete_annotation(&AnnotationId("ann-2".to_string()))
        .unwrap();

    let remaining = store.load_annotations(&PaperId("p-1".to_string())).unwrap();
    assert_eq!(remaining.len(), 2);
    assert!(remaining.iter().any(|a| a.id.0 == "ann-1"));
    assert!(remaining.iter().any(|a| a.id.0 == "ann-3"));
    assert!(!remaining.iter().any(|a| a.id.0 == "ann-2"));
}
