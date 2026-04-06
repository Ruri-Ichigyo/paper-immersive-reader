use paper_immersive_reader_application::orchestration::{
    ImportOrchestrationResult, ImportOrchestrator,
};
use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::PaperStatus;
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn import_orchestrator_persists_embedded_text_import_result_in_sqlite() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-import-sqlite-success");
    let _ = std::fs::remove_dir_all(&temp_root);
    let db_path = temp_root.join("database").join("app-state.db");

    let store = SqliteAppStateStore::open(&db_path).expect("store should open");
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
    let snapshot = orchestrator
        .store()
        .snapshot()
        .expect("snapshot should load");
    assert_eq!(snapshot.papers.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyText);
    assert_eq!(snapshot.blocks.len(), 2);
    assert_eq!(snapshot.reading_progress.len(), 1);
}
