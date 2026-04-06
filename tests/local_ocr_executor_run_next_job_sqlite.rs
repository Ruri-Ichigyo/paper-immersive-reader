use paper_immersive_reader_application::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

struct SuccessRunner;

impl PaddleOcrRunner for SuccessRunner {
    fn run(&self, job: &OcrJob, _pdf_path: &std::path::Path) -> Result<PaddleOcrRunOutput, String> {
        Ok(PaddleOcrRunOutput {
            text: format!("text for {}", job.id.0),
        })
    }
}

#[test]
fn run_next_job_executes_the_first_queued_job_in_sqlite() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-ocr-next-sqlite");
    let _ = std::fs::remove_dir_all(&temp_root);
    let db_path = temp_root.join("database").join("app-state.db");
    let mut store = SqliteAppStateStore::open(&db_path).expect("store should open");

    let paper_1 = Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };
    let paper_2 = Paper {
        id: PaperId("paper-2".to_string()),
        title: Some("scan-2".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan-2.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };
    store
        .save_paper_and_job(
            paper_1.clone(),
            OcrJob {
                id: OcrJobId("job-1".to_string()),
                paper_id: paper_1.id.clone(),
                engine: OcrEngine::PaddleOcr,
                status: OcrJobStatus::Queued,
                requested_at_ms: 1,
                started_at_ms: None,
                finished_at_ms: None,
                error_message: None,
            },
        )
        .expect("first seed should save");
    store
        .save_paper_and_job(
            paper_2.clone(),
            OcrJob {
                id: OcrJobId("job-2".to_string()),
                paper_id: paper_2.id.clone(),
                engine: OcrEngine::PaddleOcr,
                status: OcrJobStatus::Queued,
                requested_at_ms: 2,
                started_at_ms: None,
                finished_at_ms: None,
                error_message: None,
            },
        )
        .expect("second seed should save");
    store
        .persist_managed_pdf_asset(&paper_1.id, b"%PDF fake 1")
        .expect("first asset should save");
    store
        .persist_managed_pdf_asset(&paper_2.id, b"%PDF fake 2")
        .expect("second asset should save");

    let mut executor = LocalOcrExecutor::new(store, SuccessRunner, || 200);
    let completed = executor
        .run_next_job()
        .expect("first queued job should run");

    assert_eq!(completed.id, OcrJobId("job-1".to_string()));
    assert_eq!(
        executor.store().first_queued_job().map(|job| job.id),
        Some(OcrJobId("job-2".to_string()))
    );
}
