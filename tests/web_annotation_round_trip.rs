use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

#[test]
fn read_route_persists_annotations_with_text_anchors() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let paper_id = PaperId("paper-annotation-1".to_string());
    let block = Block::try_new(
        BlockId("block-annotation-1".to_string()),
        paper_id.clone(),
        0,
        "Entropy measures uncertainty in a system.".to_string(),
        None,
    )
    .expect("block should be valid");

    storage
        .save_paper_bundle(
            Paper {
                id: paper_id.clone(),
                title: Some("Annotations".to_string()),
                source_type: PaperSourceType::Paste,
                source_ref: None,
                status: PaperStatus::ReadyText,
                parse_version: 1,
            },
            vec![block.clone()],
            ReadingProgress {
                paper_id: paper_id.clone(),
                current_block_index: 0,
                current_block_id: Some(block.id.clone()),
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
        .save_highlight(Some("key term".to_string()), vec!["important".to_string()])
        .expect("annotation should save");

    let reloaded = paper_immersive_reader_web_app::routes::read::ReadRouteModel::load(
        storage.clone(),
        paper_id.clone(),
    )
    .expect("route should reload");
    let annotations = storage
        .list_annotations_for_paper(&paper_id)
        .expect("annotation list should load");
    let view = reloaded.view();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].paper_id, paper_id);
    assert_eq!(annotations[0].block_id, block.id);
    assert_eq!(annotations[0].quote, "Entropy");
    assert_eq!(annotations[0].prefix, None);
    assert_eq!(annotations[0].suffix, Some(" measures uncert".to_string()));
    assert_eq!(annotations[0].start_offset, Some(0));
    assert_eq!(annotations[0].end_offset, Some(7));
    assert_eq!(view.annotations.len(), 1);
}
