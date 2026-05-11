use paper_immersive_reader_web_app::import::pdf_bridge::StubPdfBridge;
use paper_immersive_reader_web_app::import::WebImportService;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn pdf_extraction_failure_leaves_no_orphan_papers() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    // Force extraction failure
    let extractor = StubPdfBridge::failing();
    let result = service.import_pdf(
        b"%PDF-1.4 corrupt content".to_vec(),
        Some("Broken PDF".to_string()),
        &extractor,
    );

    assert!(
        result.is_err(),
        "import should fail when pdf extraction fails"
    );

    // Store should not have any papers from the failed import
    let store = service.into_store();
    let papers = store.list_papers().unwrap();
    assert!(
        papers.is_empty(),
        "failed import should not leave orphan papers"
    );
}

#[test]
fn empty_pdf_text_returns_error() {
    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    let extractor = StubPdfBridge::with_text("");
    let result = service.import_pdf(b"%PDF-1.4 empty".to_vec(), None, &extractor);

    assert!(result.is_err(), "empty extracted text should be an error");
}
