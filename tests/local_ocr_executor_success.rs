use paper_immersive_reader::orchestration::{
    LocalOcrExecutor, PaddleOcrRunOutput, PaddleOcrRunner,
};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus, PaperId,
};
use std::cell::RefCell;

#[test]
fn run_job_marks_job_processing_runs_ocr_and_saves_completed_result() {
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
    let runner = StubPaddleOcrRunner::new("recognized text");
    let mut executor = LocalOcrExecutor::new(repository, runner, || 1_700_000_000_100);

    let completed_job = executor.run_job(&job_id).expect("job should complete");

    assert_eq!(
        completed_job,
        OcrJob {
            id: job_id.clone(),
            paper_id: paper_id.clone(),
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Completed,
            requested_at_ms: 1_700_000_000_000,
            started_at_ms: Some(1_700_000_000_100),
            finished_at_ms: Some(1_700_000_000_100),
            error_message: None,
        }
    );

    assert_eq!(executor.runner().invocations(), vec![job_id.clone()]);
    assert_eq!(
        executor.repository().saved_results(),
        vec![OcrJobResult {
            paper_id,
            text: "recognized text".to_string(),
        }]
    );
    assert_eq!(
        executor.repository().get_job(&job_id),
        Some(&OcrJob {
            id: job_id,
            paper_id: PaperId("paper-1".to_string()),
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Completed,
            requested_at_ms: 1_700_000_000_000,
            started_at_ms: Some(1_700_000_000_100),
            finished_at_ms: Some(1_700_000_000_100),
            error_message: None,
        })
    );
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
    fn run(&self, job: &OcrJob) -> Result<PaddleOcrRunOutput, String> {
        self.invocations.borrow_mut().push(job.id.clone());
        Ok(PaddleOcrRunOutput {
            text: self.text.clone(),
        })
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
