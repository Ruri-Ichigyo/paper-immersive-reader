use paper_immersive_reader_core::domain::{Block, BlockId, PaperId};
use paper_immersive_reader_web_app::reading::session::ReadingSession;
use paper_immersive_reader_web_app::reading::NavigationAction;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn navigation_updates_stored_block_index() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-nav".to_string());
    let blocks: Vec<Block> = (0..3)
        .map(|i| {
            Block::try_new(
                BlockId(format!("b-{}", i)),
                paper_id.clone(),
                i,
                format!("Block {}", i),
                None,
            )
            .unwrap()
        })
        .collect();
    store.save_blocks(&blocks).unwrap();

    // Load session
    let mut session = ReadingSession::load(&store, paper_id.clone())
        .unwrap()
        .unwrap();
    assert_eq!(session.current_index(), 0);

    // Navigate next
    let moved = session.navigate(NavigationAction::Next, &mut store).unwrap();
    assert!(moved);
    assert_eq!(session.current_index(), 1);

    // Verify stored progress
    let progress = store.load_progress(&paper_id).unwrap().unwrap();
    assert_eq!(progress.current_block_index, 1);

    // Navigate back
    let moved = session.navigate(NavigationAction::Previous, &mut store).unwrap();
    assert!(moved);
    assert_eq!(session.current_index(), 0);
}

#[test]
fn navigation_at_boundaries_returns_false() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-boundary".to_string());
    let blocks: Vec<Block> = (0..2)
        .map(|i| {
            Block::try_new(
                BlockId(format!("bb-{}", i)),
                paper_id.clone(),
                i,
                format!("Block {}", i),
                None,
            )
            .unwrap()
        })
        .collect();
    store.save_blocks(&blocks).unwrap();

    // Start at block 0, go previous
    let mut session = ReadingSession::load(&store, paper_id.clone())
        .unwrap()
        .unwrap();
    let moved = session
        .navigate(NavigationAction::Previous, &mut store)
        .unwrap();
    assert!(!moved, "should not move before first block");
    assert_eq!(session.current_index(), 0);

    // Go to last block, try next
    session.navigate(NavigationAction::Next, &mut store).unwrap();
    assert_eq!(session.current_index(), 1);
    let moved = session.navigate(NavigationAction::Next, &mut store).unwrap();
    assert!(!moved, "should not move past last block");
    assert_eq!(session.current_index(), 1);
}
