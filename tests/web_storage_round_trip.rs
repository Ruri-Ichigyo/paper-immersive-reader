use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};

#[test]
fn web_storage_round_trips_paper_bundle_by_paper_id() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let paper_id = PaperId("paper-web-1".to_string());
    let paper = Paper {
        id: paper_id.clone(),
        title: Some("Local First Reader".to_string()),
        source_type: PaperSourceType::Paste,
        source_ref: Some("paste://note".to_string()),
        status: PaperStatus::ReadyText,
        parse_version: 1,
    };
    let block = Block::try_new(
        BlockId("block-web-1".to_string()),
        paper_id.clone(),
        0,
        "A persisted reading block.".to_string(),
        Some("hash-1".to_string()),
    )
    .expect("block should be valid");
    let progress = ReadingProgress {
        paper_id: paper_id.clone(),
        current_block_index: 0,
        current_block_id: Some(block.id.clone()),
        updated_at_ms: 1_712_431_200_000,
    };

    storage
        .save_paper_bundle(paper.clone(), vec![block.clone()], progress.clone())
        .expect("save should succeed");

    let reloaded = storage
        .load_paper_bundle(&paper_id)
        .expect("load should succeed")
        .expect("bundle should exist");

    assert_eq!(reloaded.paper, paper);
    assert_eq!(reloaded.blocks, vec![block]);
    assert_eq!(reloaded.reading_progress, progress);
}
