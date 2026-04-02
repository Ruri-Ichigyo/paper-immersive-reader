use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus, PaperId,
};

struct NoopRepo {
    jobs: Vec<OcrJob>,
    results: Vec<OcrJobResult>,
}

impl OcrJobRepository for NoopRepo {
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

    fn save_result(&mut self, result: OcrJobResult) {
        self.results.push(result);
    }
}

#[test]
fn noop_repo_can_implement_ocr_job_repository_trait() {
    let paper_id = PaperId("paper-1".to_string());
    let job_id = OcrJobId("job-1".to_string());
    let job = OcrJob {
        id: job_id.clone(),
        paper_id: paper_id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    };
    let result = OcrJobResult {
        paper_id: paper_id.clone(),
        text: "recognized text".to_string(),
    };

    let mut repo = NoopRepo {
        jobs: Vec::new(),
        results: Vec::new(),
    };

    repo.create_job(job);
    repo.save_result(result.clone());

    assert_eq!(repo.get_job(&job_id).map(|job| &job.id), Some(&job_id));
    assert_eq!(repo.list_jobs_for_paper(&paper_id).len(), 1);
    assert_eq!(repo.results, vec![result]);
}
