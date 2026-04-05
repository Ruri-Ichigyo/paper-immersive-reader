use paper_immersive_reader::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};
use std::cell::RefCell;

#[test]
fn run_job_returns_none_when_job_is_missing() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-root-ocr-missing.json");
    let _ = std::fs::remove_file(&temp_path);
    let store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    let runner = StubPaddleOcrRunner::default();
    let mut executor = LocalOcrExecutor::new(store, runner, || 1_700_000_000_100);
    let missing_job_id = OcrJobId("missing-job".to_string());

    let result = executor.run_job(&missing_job_id);

    assert_eq!(result, None);
    assert!(executor.runner().invocations().is_empty());
    assert!(executor.store().snapshot().ocr_job_results.is_empty());
}

#[test]
fn run_job_returns_none_when_job_is_not_queued() {
    let job_id = OcrJobId("job-1".to_string());
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-root-ocr-not-queued.json");
    let _ = std::fs::remove_file(&temp_path);
    let mut store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyText,
        parse_version: 1,
    });
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: job_id.clone(),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Completed,
        requested_at_ms: 1_700_000_000_000,
        started_at_ms: Some(1_700_000_000_100),
        finished_at_ms: Some(1_700_000_000_200),
        error_message: None,
    });
    store.save().expect("seed state should save");

    let runner = StubPaddleOcrRunner::default();
    let mut executor = LocalOcrExecutor::new(store, runner, || 1_700_000_000_300);

    let result = executor.run_job(&job_id);

    assert_eq!(result, None);
    assert!(executor.runner().invocations().is_empty());
    assert!(executor.store().snapshot().ocr_job_results.is_empty());
    assert_eq!(
        executor.store().snapshot().ocr_jobs[0].status,
        OcrJobStatus::Completed
    );
}

#[derive(Default)]
struct StubPaddleOcrRunner {
    invocations: RefCell<Vec<OcrJobId>>,
}

impl StubPaddleOcrRunner {
    fn invocations(&self) -> Vec<OcrJobId> {
        self.invocations.borrow().clone()
    }
}

impl PaddleOcrRunner for StubPaddleOcrRunner {
    fn run(&self, job: &OcrJob, _pdf_path: &std::path::Path) -> Result<PaddleOcrRunOutput, String> {
        self.invocations.borrow_mut().push(job.id.clone());
        Ok(PaddleOcrRunOutput {
            text: "recognized text".to_string(),
        })
    }
}
