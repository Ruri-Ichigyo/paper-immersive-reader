use crate::orchestration::{PaddleOcrRunOutput, PaddleOcrRunner};
use paper_immersive_reader_core::domain::{OcrJob, OcrJobId, OcrJobRepository, OcrJobResult};

pub struct LocalOcrExecutor<R, Runner, TimestampFactory> {
    repository: R,
    runner: Runner,
    now_ms: TimestampFactory,
}

impl<R, Runner, TimestampFactory> LocalOcrExecutor<R, Runner, TimestampFactory>
where
    R: OcrJobRepository,
    Runner: PaddleOcrRunner,
    TimestampFactory: FnMut() -> u64,
{
    pub fn new(repository: R, runner: Runner, now_ms: TimestampFactory) -> Self {
        Self {
            repository,
            runner,
            now_ms,
        }
    }

    pub fn run_job(&mut self, job_id: &OcrJobId) -> Option<OcrJob> {
        let queued_job = self.repository.get_job(job_id)?.clone();
        let started_at_ms = (self.now_ms)();
        let processing_job = queued_job.mark_processing(started_at_ms).ok()?;
        self.repository.replace_job(processing_job.clone());

        match self.runner.run(&processing_job) {
            Ok(PaddleOcrRunOutput { text }) => {
                self.repository.save_result(OcrJobResult {
                    paper_id: processing_job.paper_id.clone(),
                    text,
                });

                let finished_at_ms = (self.now_ms)();
                let completed_job = processing_job.mark_completed(finished_at_ms).ok()?;
                self.repository.replace_job(completed_job.clone());
                Some(completed_job)
            }
            Err(error_message) => {
                let finished_at_ms = (self.now_ms)();
                let failed_job = processing_job
                    .mark_failed(finished_at_ms, error_message)
                    .ok()?;
                self.repository.replace_job(failed_job);
                None
            }
        }
    }

    pub fn run_next_job(&mut self) -> Option<OcrJob> {
        None
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    pub fn runner(&self) -> &Runner {
        &self.runner
    }
}
