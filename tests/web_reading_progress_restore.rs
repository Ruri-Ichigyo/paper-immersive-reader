use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

#[test]
fn reading_route_restores_non_zero_progress_on_reload() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let paper_id = PaperId("paper-read-restore-1".to_string());
    let first_block = Block::try_new(
        BlockId("block-read-restore-1".to_string()),
        paper_id.clone(),
        0,
        "First block".to_string(),
        None,
    )
    .expect("first block should be valid");
    let second_block = Block::try_new(
        BlockId("block-read-restore-2".to_string()),
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
            vec![first_block, second_block.clone()],
            ReadingProgress {
                paper_id: paper_id.clone(),
                current_block_index: 1,
                current_block_id: Some(second_block.id.clone()),
                updated_at_ms: 10,
            },
        )
        .expect("seed should succeed");

    let route =
        paper_immersive_reader_web_app::routes::read::ReadRouteModel::load(storage, paper_id)
            .expect("route should load");

    let view = route.view();

    assert_eq!(view.current_block_index, 1);
    assert_eq!(view.current_block_id, second_block.id);
    assert_eq!(view.block_text, "Second block");
}
