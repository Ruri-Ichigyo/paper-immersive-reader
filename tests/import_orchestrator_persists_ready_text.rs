use paper_immersive_reader_application::orchestration::{
    ImportOrchestrationResult, ImportOrchestrator,
};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn import_orchestrator_persists_embedded_text_import_result() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-import-success.json");
    let _ = std::fs::remove_file(&temp_path);

    let store = JsonAppStateStore::load_migration_input(&temp_path).expect("store should load");
    let mut orchestrator = ImportOrchestrator::new_with_store(
        DefaultImportPipeline::new_for_tests_with_pdf_text("hello\n\nworld"),
        store,
    );

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"fake-pdf".to_vec(),
            title: Some("paper".to_string()),
            source_ref: Some("paper.pdf".to_string()),
        })
        .expect("import should succeed");

    assert_eq!(result, ImportOrchestrationResult::ImportedText);
    let snapshot = orchestrator.store().snapshot();
    assert_eq!(snapshot.papers.len(), 1);
    assert_eq!(snapshot.blocks.len(), 2);
    assert_eq!(snapshot.reading_progress.len(), 1);
}
