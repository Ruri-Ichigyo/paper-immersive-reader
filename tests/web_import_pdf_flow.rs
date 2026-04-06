use paper_immersive_reader_core::domain::PaperStatus;

struct StubPdfBridge;

impl paper_immersive_reader_web_app::import::PdfTextBridge for StubPdfBridge {
    fn extract_text(
        &self,
        _bytes: &[u8],
    ) -> Result<String, paper_immersive_reader_web_app::import::WebImportError> {
        Ok("Extracted introduction.\n\nReadable conclusion.".to_string())
    }
}

#[test]
fn web_import_service_persists_pdf_asset_and_ready_text() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let service = paper_immersive_reader_web_app::import::WebImportService::new_with_pdf_bridge(
        storage.clone(),
        StubPdfBridge,
    );

    let result = service
        .import_pdf_document(paper_immersive_reader_web_app::import::PdfImportRequest {
            bytes: b"%PDF-1.4 fake".to_vec(),
            title: Some("PDF paper".to_string()),
            source_ref: Some("file://paper.pdf".to_string()),
            imported_at_ms: 1_712_431_260_000,
        })
        .expect("pdf import should succeed");

    let bundle = storage
        .load_paper_bundle(&result.paper_id)
        .expect("load should succeed")
        .expect("paper bundle should exist");
    let asset = storage
        .load_pdf_asset(&result.paper_id)
        .expect("asset load should succeed")
        .expect("asset should exist");

    assert_eq!(asset, b"%PDF-1.4 fake".to_vec());
    assert_eq!(bundle.paper.status, PaperStatus::ReadyText);
    assert_eq!(bundle.blocks.len(), 2);
}
