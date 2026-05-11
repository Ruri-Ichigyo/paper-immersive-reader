use paper_immersive_reader_core::domain::{Block, BlockId, PaperId, ReadingProgress};
use paper_immersive_reader_web_app::reading::session::ReadingSession;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[test]
fn reload_restores_last_saved_block() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-restore".to_string());

    // Seed blocks
    let blocks: Vec<Block> = (0..5)
        .map(|i| {
            Block::try_new(
                BlockId(format!("rb-{}", i)),
                paper_id.clone(),
                i,
                format!("Restore block {}", i),
                None,
            )
            .unwrap()
        })
        .collect();
    store.save_blocks(&blocks).unwrap();

    // Seed progress at block 3
    let existing_progress = ReadingProgress {
        paper_id: paper_id.clone(),
        current_block_index: 3,
        current_block_id: Some(BlockId("rb-3".to_string())),
        updated_at_ms: current_time_ms(),
    };
    store.save_progress(&existing_progress).unwrap();

    // Load session and verify restore
    let session = ReadingSession::load(&store, paper_id.clone())
        .unwrap()
        .unwrap();
    assert_eq!(
        session.current_index(),
        3,
        "should restore to block 3, not block 0"
    );
    assert!(
        session.current_block().is_some(),
        "should have a current block"
    );
    assert_eq!(session.total_blocks(), 5);
    assert!(session.progress_percent() > 50);
}
