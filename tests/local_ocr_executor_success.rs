use paper_immersive_reader::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};
use std::cell::RefCell;

#[test]
fn run_job_marks_job_processing_runs_ocr_and_saves_completed_result() {
    let job_id = OcrJobId("job-1".to_string());
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-root-ocr-success.json");
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
        id: job_id.clone(),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1_700_000_000_000,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store.save().expect("seed state should save");

    let runner = StubPaddleOcrRunner::new("recognized text");
    let mut executor = LocalOcrExecutor::new(store, runner, || 1_700_000_000_100);

    let completed_job = executor.run_job(&job_id).expect("job should complete");

    assert_eq!(completed_job.status, OcrJobStatus::Completed);
    assert_eq!(executor.runner().invocations(), vec![job_id.clone()]);
    let snapshot = executor.store().snapshot();
    assert_eq!(snapshot.ocr_job_results.len(), 1);
    assert_eq!(snapshot.papers[0].status, PaperStatus::ReadyText);
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Completed);
}

struct StubPaddleOcrRunner {
    text: String,
    invocations: RefCell<Vec<OcrJobId>>,
}

impl StubPaddleOcrRunner {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            invocations: RefCell::new(Vec::new()),
        }
    }

    fn invocations(&self) -> Vec<OcrJobId> {
        self.invocations.borrow().clone()
    }
}

impl PaddleOcrRunner for StubPaddleOcrRunner {
    fn run(&self, job: &OcrJob, _pdf_path: &std::path::Path) -> Result<PaddleOcrRunOutput, String> {
        self.invocations.borrow_mut().push(job.id.clone());
        Ok(PaddleOcrRunOutput {
            text: self.text.clone(),
        })
    }
}
