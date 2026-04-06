use paper_immersive_reader_application::orchestration::{
    ImportOrchestrationResult, ImportOrchestrator, OcrJobService,
};
use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::{OcrJobId, OcrJobStatus, PaperId, PaperStatus};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};

struct OcrRequiredPipeline;

impl ImportPipeline for OcrRequiredPipeline {
    fn import(
        &self,
        _request: ImportRequest,
    ) -> Result<paper_immersive_reader_importing::ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::OcrRequired)
    }
}

#[test]
fn import_orchestrator_persists_real_paper_and_queued_job_for_ocr_in_sqlite() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-ocr-queued-sqlite");
    let _ = std::fs::remove_dir_all(&temp_root);
    let db_path = temp_root.join("database").join("app-state.db");

    let store = SqliteAppStateStore::open(&db_path).expect("store should open");
    let service = OcrJobService::new(
        || OcrJobId("job-1".to_string()),
        || PaperId("paper-1".to_string()),
        || 100,
    );
    let mut orchestrator =
        ImportOrchestrator::new_with_store_and_ocr_job_service(OcrRequiredPipeline, store, service);

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF fake".to_vec(),
            title: Some("scan".to_string()),
            source_ref: Some("scan.pdf".to_string()),
        })
        .expect("should queue OCR");

    assert_eq!(
        result,
        ImportOrchestrationResult::QueuedOcr {
            paper_id: PaperId("paper-1".to_string()),
            job_id: OcrJobId("job-1".to_string()),
        }
    );

    let snapshot = orchestrator
        .store()
        .snapshot()
        .expect("snapshot should load");
    assert_eq!(snapshot.papers.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyOcr);
    assert_eq!(snapshot.ocr_jobs.len(), 1);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Queued);

    let asset_path = temp_root
        .join("database")
        .join("assets")
        .join("paper-1.pdf");
    assert!(asset_path.exists());
    assert_eq!(
        std::fs::read(asset_path).expect("asset bytes"),
        b"%PDF fake".to_vec()
    );
}
