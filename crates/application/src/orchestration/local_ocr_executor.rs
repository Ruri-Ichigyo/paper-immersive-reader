use crate::orchestration::{PaddleOcrRunOutput, PaddleOcrRunner};
use crate::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{OcrJob, OcrJobId, OcrJobResult, PaperStatus};
use paper_immersive_reader_importing::segment_text_for_existing_paper;

pub struct LocalOcrExecutor<Runner, TimestampFactory> {
    store: JsonAppStateStore,
    runner: Runner,
    now_ms: TimestampFactory,
}

impl<Runner, TimestampFactory> LocalOcrExecutor<Runner, TimestampFactory>
where
    Runner: PaddleOcrRunner,
    TimestampFactory: FnMut() -> u64,
{
    pub fn new(store: JsonAppStateStore, runner: Runner, now_ms: TimestampFactory) -> Self {
        Self {
            store,
            runner,
            now_ms,
        }
    }

    pub fn run_job(&mut self, job_id: &OcrJobId) -> Option<OcrJob> {
        let queued_job = self.store.get_job(job_id)?;
        let started_at_ms = (self.now_ms)();
        let processing_job = queued_job.mark_processing(started_at_ms).ok()?;
        self.store.save_job(processing_job.clone()).ok()?;
        let pdf_path = self.store.managed_pdf_asset_path(&processing_job.paper_id);

        match self.runner.run(&processing_job, &pdf_path) {
            Ok(PaddleOcrRunOutput { text }) => {
                let finished_at_ms = (self.now_ms)();
                let paper = self.store.get_paper(&processing_job.paper_id)?;
                let artifacts =
                    match segment_text_for_existing_paper(&paper, text.clone(), finished_at_ms) {
                        Ok(artifacts) => artifacts,
                        Err(error) => {
                            let failed_job = processing_job
                                .mark_failed(
                                    finished_at_ms,
                                    format!("failed to segment OCR text: {error:?}"),
                                )
                                .ok()?;
                            self.store.save_job(failed_job).ok()?;
                            return None;
                        }
                    };
                let completed_job = processing_job.mark_completed(finished_at_ms).ok()?;
                let mut updated_paper = paper;
                updated_paper.status = PaperStatus::ReadyText;
                self.store
                    .save_completed_ocr(
                        completed_job.clone(),
                        OcrJobResult {
                            paper_id: completed_job.paper_id.clone(),
                            text,
                        },
                        artifacts,
                        updated_paper,
                    )
                    .ok()?;
                Some(completed_job)
            }
            Err(error_message) => {
                let finished_at_ms = (self.now_ms)();
                let failed_job = processing_job
                    .mark_failed(finished_at_ms, error_message)
                    .ok()?;
                self.store.save_job(failed_job).ok()?;
                None
            }
        }
    }

    pub fn run_next_job(&mut self) -> Option<OcrJob> {
        let queued_job = self.store.first_queued_job()?;
        self.run_job(&queued_job.id)
    }

    pub fn store(&self) -> &JsonAppStateStore {
        &self.store
    }

    pub fn runner(&self) -> &Runner {
        &self.runner
    }
}
