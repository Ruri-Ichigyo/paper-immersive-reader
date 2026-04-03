use paper_immersive_reader::orchestration::{ImportOrchestrationResult, ImportOrchestrator};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportRequest};

#[test]
fn import_orchestrator_returns_imported_text_when_pipeline_succeeds() {
    let orchestrator = ImportOrchestrator::new(DefaultImportPipeline::new_for_tests());

    let result = orchestrator
        .import(ImportRequest::PasteText {
            text: "hello".to_string(),
            title: Some("paper".to_string()),
            source_ref: None,
        })
        .expect("orchestrator should succeed");

    assert!(matches!(result, ImportOrchestrationResult::ImportedText));
}
