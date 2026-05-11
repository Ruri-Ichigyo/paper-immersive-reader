use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_web_app::import::pdf_bridge::StubPdfBridge;
use paper_immersive_reader_web_app::import::WebImportService;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn pdf_import_extracts_text_and_produces_ready_paper() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    let pdf_bytes = b"%PDF-1.4 fake pdf content".to_vec();
    let extractor = StubPdfBridge::with_text(
        "Extracted section one.\n\nExtracted section two.\n\nExtracted section three.",
    );

    let paper = service
        .import_pdf(pdf_bytes, Some("PDF Paper".to_string()), &extractor)
        .unwrap();

    assert_eq!(paper.title.as_deref(), Some("PDF Paper"));
    assert_eq!(paper.source_type, PaperSourceType::Pdf);
    assert_eq!(paper.status, PaperStatus::ReadyText);

    let store = service.into_store();

    let blocks = store.load_blocks(&paper.id).unwrap();
    assert_eq!(blocks.len(), 3, "should produce blocks from extracted text");

    let progress = store.load_progress(&paper.id).unwrap().unwrap();
    assert_eq!(progress.current_block_index, 0);
}

#[test]
fn pdf_import_fails_when_extraction_produces_no_text() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    let extractor = StubPdfBridge::failing();
    let result = service.import_pdf(b"%PDF-1.4 empty".to_vec(), None, &extractor);

    assert!(result.is_err());
}
