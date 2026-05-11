use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_web_app::import::WebImportService;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn paste_import_creates_paper_blocks_and_progress() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    let text = "First paragraph of the paper.\n\nSecond paragraph with more content.\n\nThird paragraph here.";
    let paper = service.import_paste(text.to_string(), Some("Test Paper".to_string())).unwrap();

    assert_eq!(paper.title.as_deref(), Some("Test Paper"));
    assert_eq!(paper.source_type, PaperSourceType::Paste);
    assert_eq!(paper.status, PaperStatus::ReadyText);

    let store = service.into_store();

    // Blocks are created
    let blocks = store.load_blocks(&paper.id).unwrap();
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].order_index, 0);
    assert_eq!(blocks[1].order_index, 1);

    // Progress starts at block 0
    let progress = store.load_progress(&paper.id).unwrap().unwrap();
    assert_eq!(progress.current_block_index, 0);

    // Paper appears in library listing
    let papers = store.list_papers().unwrap();
    assert_eq!(papers.len(), 1);
    assert_eq!(papers[0].paper.id, paper.id);
}

#[test]
fn paste_import_rejects_empty_input() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    let result = service.import_paste("   \n\n  ".to_string(), None);
    assert!(result.is_err());
}
