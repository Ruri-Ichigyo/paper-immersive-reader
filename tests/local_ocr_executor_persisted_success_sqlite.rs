use paper_immersive_reader_application::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

struct SuccessRunner;

impl PaddleOcrRunner for SuccessRunner {
    fn run(
        &self,
        _job: &OcrJob,
        _pdf_path: &std::path::Path,
    ) -> Result<PaddleOcrRunOutput, String> {
        Ok(PaddleOcrRunOutput {
            text: "first\n\nsecond".to_string(),
        })
    }
}

#[test]
fn run_job_persists_completed_job_transition_details_and_reloads_from_sqlite() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-ocr-success-sqlite");
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

    let mut executor = LocalOcrExecutor::new(store, SuccessRunner, || 200);
    let completed = executor
        .run_job(&OcrJobId("job-1".to_string()))
        .expect("job should complete");

    assert_eq!(completed.status, OcrJobStatus::Completed);
    assert_eq!(completed.started_at_ms, Some(200));
    assert_eq!(completed.finished_at_ms, Some(200));
    assert_eq!(completed.error_message, None);

    let reloaded = SqliteAppStateStore::open(&db_path).expect("store should reload");
    let snapshot = reloaded.snapshot().expect("snapshot should load");
    assert_eq!(snapshot.ocr_job_results.len(), 1);
    assert_eq!(snapshot.ocr_job_results[0].text, "first\n\nsecond");
    assert_eq!(snapshot.blocks.len(), 2);
    assert_eq!(snapshot.blocks[0].text, "first");
    assert_eq!(snapshot.blocks[1].text, "second");
    assert_eq!(snapshot.reading_progress.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyText);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Completed);
    assert_eq!(snapshot.ocr_jobs[0].started_at_ms, Some(200));
    assert_eq!(snapshot.ocr_jobs[0].finished_at_ms, Some(200));
    assert_eq!(snapshot.ocr_jobs[0].error_message, None);
}
