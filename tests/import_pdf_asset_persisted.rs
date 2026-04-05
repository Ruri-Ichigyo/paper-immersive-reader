use paper_immersive_reader_application::orchestration::ImportOrchestrator;
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn pdf_import_persists_managed_pdf_asset_under_database_assets() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-cli-assets");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let state_path = temp_root.join("app-state.json");

    let store = JsonAppStateStore::load_or_default(&state_path).expect("store should load");
    let mut orchestrator = ImportOrchestrator::new_with_store(
        DefaultImportPipeline::new_for_tests_with_pdf_text("hello"),
        store,
    );

    orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF fake asset".to_vec(),
            title: Some("paper".to_string()),
            source_ref: Some("original.pdf".to_string()),
        })
        .expect("import should succeed");

    let asset_path = temp_root.join("assets").join("paper-1.pdf");
    assert!(asset_path.exists());
    assert_eq!(
        std::fs::read(&asset_path).expect("asset bytes"),
        b"%PDF fake asset".to_vec()
    );
}
