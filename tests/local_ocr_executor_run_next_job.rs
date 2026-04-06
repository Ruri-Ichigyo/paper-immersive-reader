use paper_immersive_reader_application::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
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
fn run_next_job_executes_the_first_queued_job() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-ocr-next.json");
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
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-2".to_string()),
        title: Some("scan-2".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan-2.pdf".to_string()),
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
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-2".to_string()),
        paper_id: PaperId("paper-2".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 2,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store.save().expect("seed state should save");

    let mut executor = LocalOcrExecutor::new(store, SuccessRunner, || 200);
    let completed = executor
        .run_next_job()
        .expect("first queued job should run");

    assert_eq!(completed.id, OcrJobId("job-1".to_string()));
}
