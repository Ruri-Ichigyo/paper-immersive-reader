use paper_immersive_reader::orchestration::{
    ImportOrchestrationResult, ImportOrchestrator, OcrJobService,
};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{OcrJobId, OcrJobStatus, PaperId, PaperStatus};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};

#[test]
fn import_orchestrator_queues_ocr_when_pipeline_requires_it() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-ocr-queued-root-test.json");
    let _ = std::fs::remove_file(&temp_path);

    let store = JsonAppStateStore::load_migration_input(&temp_path).expect("store should load");
    let service = OcrJobService::new(
        || OcrJobId("job-1".to_string()),
        || PaperId("paper-ocr-1".to_string()),
        || 1_700_000_000_000,
    );
    let mut orchestrator =
        ImportOrchestrator::new_with_store_and_ocr_job_service(OcrRequiredPipeline, store, service);

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"fake-pdf".to_vec(),
            title: Some("scanned paper".to_string()),
            source_ref: Some("scan.pdf".to_string()),
        })
        .expect("orchestrator should queue OCR");

    assert_eq!(
        result,
        ImportOrchestrationResult::QueuedOcr {
            paper_id: PaperId("paper-ocr-1".to_string()),
            job_id: OcrJobId("job-1".to_string()),
        }
    );

    let snapshot = orchestrator.store().snapshot();
    assert_eq!(snapshot.papers.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyOcr);
    assert_eq!(snapshot.ocr_jobs.len(), 1);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Queued);
}

struct OcrRequiredPipeline;

impl ImportPipeline for OcrRequiredPipeline {
    fn import(
        &self,
        _request: ImportRequest,
    ) -> Result<paper_immersive_reader_importing::ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::OcrRequired)
    }
}
