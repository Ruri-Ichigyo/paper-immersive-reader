use paper_immersive_reader_core::domain::{
    AnchoringStatus, Annotation, AnnotationId, AnnotationType, Block, BlockId, PaperId,
    PaperSourceType, PaperStatus, ReadingProgress,
};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::schema::StoredPaper;
use paper_immersive_reader_web_app::storage::AppStorage;

fn make_paper(id: &str) -> StoredPaper {
    StoredPaper {
        paper: paper_immersive_reader_core::domain::Paper {
            id: PaperId(id.to_string()),
            title: Some("Test".to_string()),
            source_type: PaperSourceType::Paste,
            source_ref: None,
            status: PaperStatus::ReadyText,
            parse_version: 0,
        },
        created_at_ms: 1000,
        updated_at_ms: 1000,
    }
}

#[test]
fn delete_paper_removes_paper_and_related_data() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("p-del".to_string());

    store.save_paper(&make_paper("p-del")).unwrap();
    store
        .save_blocks(&[Block {
            id: BlockId("b-1".to_string()),
            paper_id: pid.clone(),
            order_index: 0,
            text: "hello".to_string(),
            block_hash: None,
        }])
        .unwrap();
    store
        .save_progress(&ReadingProgress {
            paper_id: pid.clone(),
            current_block_index: 0,
            current_block_id: Some(BlockId("b-1".to_string())),
            updated_at_ms: 1000,
        })
        .unwrap();
    store
        .save_annotation(&Annotation {
            id: AnnotationId("a-1".to_string()),
            paper_id: pid.clone(),
            block_id: BlockId("b-1".to_string()),
            annotation_type: AnnotationType::Highlight,
            quote: "hello".to_string(),
            prefix: None,
            suffix: None,
            start_offset: None,
            end_offset: None,
            note: None,
            tags: vec![],
            anchoring_status: AnchoringStatus::Anchored,
        })
        .unwrap();

    // Verify data exists
    assert!(store.load_paper(&pid).unwrap().is_some());
    assert_eq!(store.load_blocks(&pid).unwrap().len(), 1);
    assert!(store.load_progress(&pid).unwrap().is_some());
    assert_eq!(store.load_annotations(&pid).unwrap().len(), 1);

    // Delete
    store.delete_paper(&pid).unwrap();

    // Verify everything is gone
    assert!(store.load_paper(&pid).unwrap().is_none());
    assert!(store.load_blocks(&pid).unwrap().is_empty());
    assert!(store.load_progress(&pid).unwrap().is_none());
    assert!(store.load_annotations(&pid).unwrap().is_empty());
}

#[test]
fn delete_paper_not_found() {
    let mut store = InMemoryStore::new();
    let result = store.delete_paper(&PaperId("missing".to_string()));
    assert!(result.is_err());
}
