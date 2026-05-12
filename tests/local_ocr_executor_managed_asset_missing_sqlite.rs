use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, ScriptPaddleOcrRunner};
use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

#[test]
fn run_job_fails_when_managed_pdf_asset_is_missing_in_sqlite() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-cli-missing-asset-sqlite");
    let _ = std::fs::remove_dir_all(&temp_root);
    let db_path = temp_root.join("database").join("app-state.db");

    let mut store = SqliteAppStateStore::open(&db_path).expect("store should open");
    let paper = Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("original.pdf".to_string()),
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
        .save_paper_and_job(paper, job)
        .expect("seed state should save");

    let runner = ScriptPaddleOcrRunner::new(std::path::PathBuf::from("scripts/run_paddle_ocr.sh"));
    let mut executor = LocalOcrExecutor::new(store, runner, || 200);

    let result = executor.run_job(&OcrJobId("job-1".to_string()));

    assert!(result.is_none());

    let reloaded = SqliteAppStateStore::open(&db_path).expect("store should reload");
    let snapshot = reloaded.snapshot().expect("snapshot should load");
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Failed);
    assert_eq!(snapshot.ocr_jobs[0].started_at_ms, Some(200));
    assert_eq!(snapshot.ocr_jobs[0].finished_at_ms, Some(200));
    assert!(snapshot.ocr_jobs[0]
        .error_message
        .as_deref()
        .expect("missing asset error should be present")
        .contains("managed pdf asset missing"));
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyOcr);
    assert_eq!(snapshot.ocr_job_results.len(), 0);
    assert_eq!(snapshot.blocks.len(), 0);
    assert_eq!(snapshot.reading_progress.len(), 0);
}
