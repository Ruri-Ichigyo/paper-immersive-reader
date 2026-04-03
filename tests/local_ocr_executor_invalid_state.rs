use paper_immersive_reader::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus, PaperId,
};
use std::cell::RefCell;

#[test]
fn run_job_returns_none_when_job_is_missing() {
    let repository = InMemoryOcrJobRepository::default();
    let runner = StubPaddleOcrRunner::default();
    let mut executor = LocalOcrExecutor::new(repository, runner, || 1_700_000_000_100);
    let missing_job_id = OcrJobId("missing-job".to_string());

    let result = executor.run_job(&missing_job_id);

    assert_eq!(result, None);
    assert!(executor.runner().invocations().is_empty());
    assert!(executor.repository().saved_results().is_empty());
}

#[test]
fn run_job_returns_none_when_job_is_not_queued() {
    let job_id = OcrJobId("job-1".to_string());
    let paper_id = PaperId("paper-1".to_string());
    let completed_job = OcrJob {
        id: job_id.clone(),
        paper_id: paper_id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Completed,
        requested_at_ms: 1_700_000_000_000,
        started_at_ms: Some(1_700_000_000_100),
        finished_at_ms: Some(1_700_000_000_200),
        error_message: None,
    };
    let repository = InMemoryOcrJobRepository::with_job(completed_job.clone());
    let runner = StubPaddleOcrRunner::default();
    let mut executor = LocalOcrExecutor::new(repository, runner, || 1_700_000_000_300);

    let result = executor.run_job(&job_id);

    assert_eq!(result, None);
    assert!(executor.runner().invocations().is_empty());
    assert!(executor.repository().saved_results().is_empty());
    assert_eq!(executor.repository().get_job(&job_id), Some(&completed_job));
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
    fn run(&self, job: &OcrJob) -> Result<PaddleOcrRunOutput, String> {
        self.invocations.borrow_mut().push(job.id.clone());
        Ok(PaddleOcrRunOutput {
            text: "recognized text".to_string(),
        })
    }
}

#[derive(Default)]
struct InMemoryOcrJobRepository {
    jobs: Vec<OcrJob>,
    results: Vec<OcrJobResult>,
}

impl InMemoryOcrJobRepository {
    fn with_job(job: OcrJob) -> Self {
        Self {
            jobs: vec![job],
            results: Vec::new(),
        }
    }

    fn saved_results(&self) -> Vec<OcrJobResult> {
        self.results.clone()
    }
}

impl OcrJobRepository for InMemoryOcrJobRepository {
    fn create_job(&mut self, job: OcrJob) {
        self.jobs.push(job);
    }

    fn get_job(&self, id: &OcrJobId) -> Option<&OcrJob> {
        self.jobs.iter().find(|job| &job.id == id)
    }

    fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<&OcrJob> {
        self.jobs
            .iter()
            .filter(|job| &job.paper_id == paper_id)
            .collect()
    }

    fn replace_job(&mut self, job: OcrJob) {
        if let Some(existing) = self.jobs.iter_mut().find(|existing| existing.id == job.id) {
            *existing = job;
        } else {
            self.jobs.push(job);
        }
    }

    fn save_result(&mut self, result: OcrJobResult) {
        self.results.push(result);
    }
}
