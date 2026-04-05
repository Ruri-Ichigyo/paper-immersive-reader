use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

pub struct OcrJobService<JobIdFactory, PaperIdFactory, TimestampFactory> {
    next_job_id: JobIdFactory,
    next_paper_id: PaperIdFactory,
    now_ms: TimestampFactory,
}

impl<JobIdFactory, PaperIdFactory, TimestampFactory>
    OcrJobService<JobIdFactory, PaperIdFactory, TimestampFactory>
where
    JobIdFactory: FnMut() -> OcrJobId,
    PaperIdFactory: FnMut() -> PaperId,
    TimestampFactory: FnMut() -> u64,
{
    pub fn new(
        next_job_id: JobIdFactory,
        next_paper_id: PaperIdFactory,
        now_ms: TimestampFactory,
    ) -> Self {
        Self {
            next_job_id,
            next_paper_id,
            now_ms,
        }
    }

    pub fn build_ready_ocr_paper(
        &mut self,
        title: Option<String>,
        source_ref: Option<String>,
    ) -> Paper {
        let paper_id = (self.next_paper_id)();
        Paper {
            id: paper_id,
            title,
            source_type: PaperSourceType::Pdf,
            source_ref,
            status: PaperStatus::ReadyOcr,
            parse_version: 1,
        }
    }

    pub fn build_queued_job(&mut self, paper_id: PaperId) -> OcrJob {
        let job_id = (self.next_job_id)();
        OcrJob {
            id: job_id,
            paper_id: paper_id.clone(),
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Queued,
            requested_at_ms: (self.now_ms)(),
            started_at_ms: None,
            finished_at_ms: None,
            error_message: None,
        }
    }
}
