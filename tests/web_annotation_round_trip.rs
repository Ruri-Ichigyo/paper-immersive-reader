use paper_immersive_reader_core::domain::{
    AnchoringStatus, AnnotationType, BlockId, PaperId,
};
use paper_immersive_reader_web_app::annotations::{
    create_annotation, save_and_reload_annotation, AnnotationInput,
};
use paper_immersive_reader_web_app::selection::TextSelection;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn annotation_survives_save_and_reload() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-ann".to_string());
    let block_id = BlockId("b-ann".to_string());
    let block_text = "This is the full text of the block. The selected portion is in the middle. And more text follows.";

    let selection = TextSelection {
        block_id: block_id.clone(),
        text: "selected portion".to_string(),
        start_offset: 44,
        end_offset: 60,
    };

    let annotation = create_annotation(
        paper_id.clone(),
        block_id.clone(),
        block_text,
        &selection,
        AnnotationInput {
            annotation_type: AnnotationType::Highlight,
            note: Some("Important concept".to_string()),
            tags: vec!["key-term".to_string()],
        },
    );

    assert_eq!(annotation.quote, "selected portion");
    assert_eq!(annotation.annotation_type, AnnotationType::Highlight);
    assert_eq!(annotation.anchoring_status, AnchoringStatus::Anchored);
    assert!(annotation.prefix.is_some());
    assert!(annotation.suffix.is_some());

    // Save and reload
    let persisted = save_and_reload_annotation(&mut store, &annotation).unwrap();
    assert!(persisted, "annotation should persist and reload");

    // Load all annotations for the paper
    let annotations = store.load_annotations(&paper_id).unwrap();
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].quote, "selected portion");
    assert_eq!(annotations[0].paper_id, paper_id);
    assert_eq!(annotations[0].block_id, block_id);
    assert_eq!(annotations[0].tags, vec!["key-term"]);
}
