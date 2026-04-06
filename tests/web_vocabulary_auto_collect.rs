use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

#[test]
fn lookup_events_upsert_vocabulary_and_are_visible_in_route() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let paper_id = PaperId("paper-vocabulary-1".to_string());
    let block = Block::try_new(
        BlockId("block-vocabulary-1".to_string()),
        paper_id.clone(),
        0,
        "Entropy matters for thermodynamics.".to_string(),
        None,
    )
    .expect("block should be valid");

    storage
        .save_paper_bundle(
            Paper {
                id: paper_id.clone(),
                title: Some("Vocabulary".to_string()),
                source_type: PaperSourceType::Paste,
                source_ref: None,
                status: PaperStatus::ReadyText,
                parse_version: 1,
            },
            vec![block],
            ReadingProgress {
                paper_id: paper_id.clone(),
                current_block_index: 0,
                current_block_id: Some(BlockId("block-vocabulary-1".to_string())),
                updated_at_ms: 10,
            },
        )
        .expect("seed should succeed");

    let mut route = paper_immersive_reader_web_app::routes::read::ReadRouteModel::load(
        storage.clone(),
        paper_id.clone(),
    )
    .expect("route should load");

    route
        .select_current_block_range(0, 7)
        .expect("selection should succeed");
    route
        .record_lookup_event(100)
        .expect("first lookup should save");
    route
        .select_current_block_range(0, 7)
        .expect("selection should succeed");
    route
        .record_lookup_event(200)
        .expect("second lookup should merge");

    let vocabulary = paper_immersive_reader_web_app::routes::vocabulary::VocabularyRouteModel::new(
        storage, paper_id,
    )
    .entries()
    .expect("vocabulary entries should load");

    assert_eq!(vocabulary.len(), 1);
    assert_eq!(vocabulary[0].term, "entropy");
    assert_eq!(vocabulary[0].lookup_count, 2);
}
