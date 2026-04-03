use paper_immersive_reader::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus, PaperId,
};
use std::cell::RefCell;

#[test]
fn run_job_marks_processing_runner_failure_and_persists_failed_job() {
    let job_id = OcrJobId("job-1".to_string());
    let paper_id = PaperId("paper-1".to_string());
    let repository = InMemoryOcrJobRepository::with_job(OcrJob {
        id: job_id.clone(),
        paper_id: paper_id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1_700_000_000_000,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    let runner = StubFailingPaddleOcrRunner::new("paddleocr crashed");
    let mut timestamps = vec![1_700_000_000_100, 1_700_000_000_200].into_iter();
    let mut executor = LocalOcrExecutor::new(repository, runner, move || {
        timestamps.next().expect("timestamp should exist")
    });

    let result = executor.run_job(&job_id);

    assert_eq!(result, None);
    assert_eq!(executor.runner().invocations(), vec![job_id.clone()]);
    assert_eq!(
        executor.repository().saved_results(),
        Vec::<OcrJobResult>::new()
    );
    assert_eq!(
        executor.repository().get_job(&job_id),
        Some(&OcrJob {
            id: job_id,
            paper_id,
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Failed,
            requested_at_ms: 1_700_000_000_000,
            started_at_ms: Some(1_700_000_000_100),
            finished_at_ms: Some(1_700_000_000_200),
            error_message: Some("paddleocr crashed".to_string()),
        })
    );
}

struct StubFailingPaddleOcrRunner {
    message: String,
    invocations: RefCell<Vec<OcrJobId>>,
}

impl StubFailingPaddleOcrRunner {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            invocations: RefCell::new(Vec::new()),
        }
    }

    fn invocations(&self) -> Vec<OcrJobId> {
        self.invocations.borrow().clone()
    }
}

impl PaddleOcrRunner for StubFailingPaddleOcrRunner {
    fn run(&self, job: &OcrJob) -> Result<PaddleOcrRunOutput, String> {
        self.invocations.borrow_mut().push(job.id.clone());
        Err(self.message.clone())
    }
}

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
