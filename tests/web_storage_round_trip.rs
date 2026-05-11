use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};
use paper_immersive_reader_web_app::storage::{
    memory_store::InMemoryStore, schema::StoredPaper, AppStorage,
};

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[test]
fn paper_block_progress_round_trip_through_in_memory_store() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-1".to_string());

    // Save paper
    let paper = Paper {
        id: paper_id.clone(),
        title: Some("Test Paper".to_string()),
        source_type: PaperSourceType::Paste,
        source_ref: None,
        status: PaperStatus::ReadyText,
        parse_version: 1,
    };
    let now = current_time_ms();
    let stored = StoredPaper {
        paper: paper.clone(),
        created_at_ms: now,
        updated_at_ms: now,
    };
    store.save_paper(&stored).unwrap();

    // Reload paper
    let loaded = store.load_paper(&paper_id).unwrap().unwrap();
    assert_eq!(loaded.paper.title.as_deref(), Some("Test Paper"));
    assert_eq!(loaded.paper.status, PaperStatus::ReadyText);

    // Save blocks
    let block1 = Block::try_new(
        BlockId("b-1".to_string()),
        paper_id.clone(),
        0,
        "First block".to_string(),
        None,
    )
    .unwrap();
    let block2 = Block::try_new(
        BlockId("b-2".to_string()),
        paper_id.clone(),
        1,
        "Second block".to_string(),
        None,
    )
    .unwrap();
    store.save_blocks(&[block1, block2]).unwrap();

    // Reload blocks
    let blocks = store.load_blocks(&paper_id).unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].order_index, 0);
    assert_eq!(blocks[1].order_index, 1);

    // Save progress
    let progress = ReadingProgress {
        paper_id: paper_id.clone(),
        current_block_index: 1,
        current_block_id: Some(BlockId("b-2".to_string())),
        updated_at_ms: current_time_ms(),
    };
    store.save_progress(&progress).unwrap();

    // Reload progress
    let loaded_progress = store.load_progress(&paper_id).unwrap().unwrap();
    assert_eq!(loaded_progress.current_block_index, 1);
    assert_eq!(
        loaded_progress
            .current_block_id
            .as_ref()
            .map(|b| b.0.as_str()),
        Some("b-2")
    );

    // List papers
    let papers = store.list_papers().unwrap();
    assert_eq!(papers.len(), 1);
    assert_eq!(papers[0].paper.id, paper_id);
}
