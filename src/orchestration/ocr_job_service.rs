use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobStatus, PaperId,
};

pub struct OcrJobService<R, IdFactory, TimestampFactory> {
    repository: R,
    next_job_id: IdFactory,
    now_ms: TimestampFactory,
}

impl<R, IdFactory, TimestampFactory> OcrJobService<R, IdFactory, TimestampFactory>
where
    R: OcrJobRepository,
    IdFactory: FnMut() -> OcrJobId,
    TimestampFactory: FnMut() -> u64,
{
    pub fn new(repository: R, next_job_id: IdFactory, now_ms: TimestampFactory) -> Self {
        Self {
            repository,
            next_job_id,
            now_ms,
        }
    }

    pub fn queue_paddle_ocr(&mut self, paper_id: PaperId) -> OcrJobId {
        let job_id = (self.next_job_id)();
        let requested_at_ms = (self.now_ms)();

        self.repository.create_job(OcrJob {
            id: job_id.clone(),
            paper_id,
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Queued,
            requested_at_ms,
            started_at_ms: None,
            finished_at_ms: None,
            error_message: None,
        });

        job_id
    }

    pub fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<OcrJob> {
        self.repository
            .list_jobs_for_paper(paper_id)
            .into_iter()
            .cloned()
            .collect()
    }
}
