use paper_immersive_reader_core::domain::PaperStatus;

#[test]
fn web_import_service_persists_pasted_text_into_local_storage() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let service = paper_immersive_reader_web_app::import::WebImportService::new(storage.clone());

    let result = service
        .import_paste_text(paper_immersive_reader_web_app::import::PasteImportRequest {
            text: "First paragraph.\n\nSecond paragraph.".to_string(),
            title: Some("Pasted note".to_string()),
            source_ref: Some("paste://clipboard".to_string()),
            imported_at_ms: 1_712_431_200_000,
        })
        .expect("paste import should succeed");

    let papers = storage.list_papers().expect("list should succeed");
    let bundle = storage
        .load_paper_bundle(&result.paper_id)
        .expect("load should succeed")
        .expect("paper bundle should exist");

    assert_eq!(papers.len(), 1);
    assert_eq!(bundle.paper.status, PaperStatus::ReadyText);
    assert!(!bundle.blocks.is_empty());
    assert_eq!(bundle.reading_progress.current_block_index, 0);
}
