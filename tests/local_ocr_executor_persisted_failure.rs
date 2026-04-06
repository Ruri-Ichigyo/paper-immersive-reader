use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, PaddleOcrRunner};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

struct FailingRunner;

impl PaddleOcrRunner for FailingRunner {
    fn run(
        &self,
        _job: &OcrJob,
        _pdf_path: &std::path::Path,
    ) -> Result<paper_immersive_reader_application::orchestration::PaddleOcrRunOutput, String> {
        Err("ocr failed".to_string())
    }
}

#[test]
fn run_job_persists_failed_job_and_keeps_paper_ready_ocr() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-ocr-failure.json");
    let _ = std::fs::remove_file(&temp_path);
    let mut store =
        JsonAppStateStore::load_migration_input(&temp_path).expect("store should load");
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store.save().expect("seed state should save");

    let mut executor = LocalOcrExecutor::new(store, FailingRunner, || 200);
    let result = executor.run_job(&OcrJobId("job-1".to_string()));

    assert!(result.is_none());
    let snapshot = executor.store().snapshot();
    assert_eq!(snapshot.ocr_job_results.len(), 0);
    assert_eq!(snapshot.blocks.len(), 0);
    assert_eq!(snapshot.reading_progress.len(), 0);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyOcr);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Failed);
}
