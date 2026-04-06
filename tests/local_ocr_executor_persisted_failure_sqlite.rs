use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, PaddleOcrRunner};
use paper_immersive_reader_application::persistence::SqliteAppStateStore;
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
fn run_job_persists_failed_job_and_keeps_paper_ready_ocr_in_sqlite() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-ocr-failure-sqlite");
    let _ = std::fs::remove_dir_all(&temp_root);
    let db_path = temp_root.join("database").join("app-state.db");
    let mut store = SqliteAppStateStore::open(&db_path).expect("store should open");

    let paper = Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };
    let job = OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: paper.id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    };
    store
        .save_paper_and_job(paper.clone(), job)
        .expect("seed state should save");
    store
        .persist_managed_pdf_asset(&paper.id, b"%PDF fake")
        .expect("managed asset should save");

    let mut executor = LocalOcrExecutor::new(store, FailingRunner, || 200);
    let result = executor.run_job(&OcrJobId("job-1".to_string()));

    assert!(result.is_none());
    let snapshot = executor.store().snapshot().expect("snapshot should load");
    assert_eq!(snapshot.ocr_job_results.len(), 0);
    assert_eq!(snapshot.blocks.len(), 0);
    assert_eq!(snapshot.reading_progress.len(), 0);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyOcr);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Failed);
}
