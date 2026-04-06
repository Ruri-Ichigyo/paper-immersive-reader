use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

#[test]
fn reading_route_moves_between_blocks_and_persists_progress() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let paper_id = PaperId("paper-read-1".to_string());
    let first_block = Block::try_new(
        BlockId("block-read-1".to_string()),
        paper_id.clone(),
        0,
        "First block".to_string(),
        None,
    )
    .expect("first block should be valid");
    let second_block = Block::try_new(
        BlockId("block-read-2".to_string()),
        paper_id.clone(),
        1,
        "Second block".to_string(),
        None,
    )
    .expect("second block should be valid");

    storage
        .save_paper_bundle(
            Paper {
                id: paper_id.clone(),
                title: Some("Reader".to_string()),
                source_type: PaperSourceType::Paste,
                source_ref: None,
                status: PaperStatus::ReadyText,
                parse_version: 1,
            },
            vec![first_block.clone(), second_block.clone()],
            ReadingProgress {
                paper_id: paper_id.clone(),
                current_block_index: 0,
                current_block_id: Some(first_block.id.clone()),
                updated_at_ms: 10,
            },
        )
        .expect("seed should succeed");

    let mut route = paper_immersive_reader_web_app::routes::read::ReadRouteModel::load(
        storage.clone(),
        paper_id.clone(),
    )
    .expect("route should load");

    route.go_next(20).expect("next should succeed");
    let after_next = storage
        .load_paper_bundle(&paper_id)
        .expect("load should succeed")
        .expect("bundle should exist");

    assert_eq!(after_next.reading_progress.current_block_index, 1);
    assert_eq!(
        after_next.reading_progress.current_block_id,
        Some(second_block.id.clone())
    );

    route.go_previous(30).expect("previous should succeed");
    let after_previous = storage
        .load_paper_bundle(&paper_id)
        .expect("load should succeed")
        .expect("bundle should exist");

    assert_eq!(after_previous.reading_progress.current_block_index, 0);
    assert_eq!(
        after_previous.reading_progress.current_block_id,
        Some(first_block.id)
    );
}
