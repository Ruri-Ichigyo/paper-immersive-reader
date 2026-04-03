use paper_immersive_reader::orchestration::ImportOrchestrationResult;

#[test]
fn import_orchestration_result_type_is_exported() {
    let _ = ImportOrchestrationResult::ImportedText;
    let _ = ImportOrchestrationResult::QueuedOcr {
        paper_id: paper_immersive_reader_core::domain::PaperId("paper-1".to_string()),
        job_id: paper_immersive_reader_core::domain::OcrJobId("job-1".to_string()),
    };
}
