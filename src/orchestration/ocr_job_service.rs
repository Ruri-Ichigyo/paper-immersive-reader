use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobStatus, PaperId,
};

pub struct OcrJobService<R, JobIdFactory, PaperIdFactory, TimestampFactory> {
    repository: R,
    next_job_id: JobIdFactory,
    next_paper_id: PaperIdFactory,
    now_ms: TimestampFactory,
}

impl<R, JobIdFactory, PaperIdFactory, TimestampFactory>
    OcrJobService<R, JobIdFactory, PaperIdFactory, TimestampFactory>
where
    R: OcrJobRepository,
    JobIdFactory: FnMut() -> OcrJobId,
    PaperIdFactory: FnMut() -> PaperId,
    TimestampFactory: FnMut() -> u64,
{
    pub fn new(
        repository: R,
        next_job_id: JobIdFactory,
        next_paper_id: PaperIdFactory,
        now_ms: TimestampFactory,
    ) -> Self {
        Self {
            repository,
            next_job_id,
            next_paper_id,
            now_ms,
        }
    }

    pub fn queue_paddle_ocr(&mut self) -> (PaperId, OcrJobId) {
        let paper_id = (self.next_paper_id)();
        let job_id = (self.next_job_id)();
        let requested_at_ms = (self.now_ms)();

        self.repository.create_job(OcrJob {
            id: job_id.clone(),
            paper_id: paper_id.clone(),
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Queued,
            requested_at_ms,
            started_at_ms: None,
            finished_at_ms: None,
            error_message: None,
        });

        (paper_id, job_id)
    }

    pub fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<OcrJob> {
        self.repository
            .list_jobs_for_paper(paper_id)
            .into_iter()
            .cloned()
            .collect()
    }
}
