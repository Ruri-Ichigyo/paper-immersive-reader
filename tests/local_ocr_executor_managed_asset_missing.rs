use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, ScriptPaddleOcrRunner};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

#[test]
fn run_job_fails_when_managed_pdf_asset_is_missing() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-cli-missing-asset");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let state_path = temp_root.join("app-state.json");

    let mut store =
        JsonAppStateStore::load_migration_input(&state_path).expect("store should load");
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("original.pdf".to_string()),
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

    let runner = ScriptPaddleOcrRunner::new(std::path::PathBuf::from("scripts/run_paddle_ocr.sh"));
    let mut executor = LocalOcrExecutor::new(store, runner, || 200);

    let result = executor.run_job(&OcrJobId("job-1".to_string()));

    assert!(result.is_none());
    assert_eq!(
        executor.store().snapshot().ocr_jobs[0].status,
        OcrJobStatus::Failed
    );
    assert_eq!(
        executor.store().snapshot().papers[0].status,
        PaperStatus::ReadyOcr
    );
}
