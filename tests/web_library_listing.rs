use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_web_app::import::WebImportService;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn library_lists_papers_with_status_and_progress() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    // Import two papers
    let paper1 = service
        .import_paste(
            "First paper content.\n\nSecond paragraph.".to_string(),
            Some("Paper Alpha".to_string()),
        )
        .unwrap();
    let paper2 = service
        .import_paste(
            "Another paper.\n\nWith paragraphs.\n\nThree blocks.".to_string(),
            Some("Paper Beta".to_string()),
        )
        .unwrap();

    let store = service.into_store();
    let papers = store.list_papers().unwrap();

    assert_eq!(papers.len(), 2);

    // Each paper has title, source type, and status
    let alpha = papers.iter().find(|p| p.paper.id == paper1.id).unwrap();
    assert_eq!(alpha.paper.title.as_deref(), Some("Paper Alpha"));
    assert_eq!(alpha.paper.source_type, PaperSourceType::Paste);
    assert_eq!(alpha.paper.status, PaperStatus::ReadyText);

    let beta = papers.iter().find(|p| p.paper.id == paper2.id).unwrap();
    assert_eq!(beta.paper.title.as_deref(), Some("Paper Beta"));
    assert_eq!(beta.paper.source_type, PaperSourceType::Paste);
    assert_eq!(beta.paper.status, PaperStatus::ReadyText);

    // Each paper should have progress
    for p in &papers {
        let progress = store.load_progress(&p.paper.id).unwrap();
        assert!(progress.is_some(), "paper should have reading progress");
    }
}

#[test]
fn library_initially_empty() {
    let store = InMemoryStore::new();
    let papers = store.list_papers().unwrap();
    assert!(papers.is_empty(), "new library should have no papers");
}
