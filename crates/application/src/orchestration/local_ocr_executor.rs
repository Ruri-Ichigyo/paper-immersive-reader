use crate::orchestration::{PaddleOcrRunOutput, PaddleOcrRunner};
use crate::persistence::{JsonAppStateStore, SqliteAppStateStore};
use paper_immersive_reader_core::domain::{
    OcrJob, OcrJobId, OcrJobResult, Paper, PaperId, PaperStatus,
};
use paper_immersive_reader_importing::{segment_text_for_existing_paper, TextImportArtifacts};

pub trait LocalOcrStateStore {
    fn get_job(&self, job_id: &OcrJobId) -> Option<OcrJob>;
    fn get_paper(&self, paper_id: &PaperId) -> Option<Paper>;
    fn save_job(&mut self, job: OcrJob) -> bool;
    fn save_completed_ocr(
        &mut self,
        completed_job: OcrJob,
        ocr_result: OcrJobResult,
        artifacts: TextImportArtifacts,
        updated_paper: Paper,
    ) -> bool;
    fn first_queued_job(&self) -> Option<OcrJob>;
    fn managed_pdf_asset_path(&self, paper_id: &PaperId) -> std::path::PathBuf;

    fn claim_job(&mut self, job_id: &OcrJobId, started_at_ms: u64) -> Option<OcrJob> {
        let queued_job = self.get_job(job_id)?;
        let processing_job = queued_job.mark_processing(started_at_ms).ok()?;
        self.save_job(processing_job.clone())
            .then_some(processing_job)
    }

    fn claim_next_queued_job(&mut self, started_at_ms: u64) -> Option<OcrJob> {
        let queued_job = self.first_queued_job()?;
        self.claim_job(&queued_job.id, started_at_ms)
    }
}

impl LocalOcrStateStore for JsonAppStateStore {
    fn get_job(&self, job_id: &OcrJobId) -> Option<OcrJob> {
        JsonAppStateStore::get_job(self, job_id)
    }

    fn get_paper(&self, paper_id: &PaperId) -> Option<Paper> {
        JsonAppStateStore::get_paper(self, paper_id)
    }

    fn save_job(&mut self, job: OcrJob) -> bool {
        JsonAppStateStore::save_job(self, job).is_ok()
    }

    fn save_completed_ocr(
        &mut self,
        completed_job: OcrJob,
        ocr_result: OcrJobResult,
        artifacts: TextImportArtifacts,
        updated_paper: Paper,
    ) -> bool {
        JsonAppStateStore::save_completed_ocr(
            self,
            completed_job,
            ocr_result,
            artifacts,
            updated_paper,
        )
        .is_ok()
    }

    fn first_queued_job(&self) -> Option<OcrJob> {
        JsonAppStateStore::first_queued_job(self)
    }

    fn managed_pdf_asset_path(&self, paper_id: &PaperId) -> std::path::PathBuf {
        JsonAppStateStore::managed_pdf_asset_path(self, paper_id)
    }
}

impl LocalOcrStateStore for SqliteAppStateStore {
    fn get_job(&self, job_id: &OcrJobId) -> Option<OcrJob> {
        SqliteAppStateStore::get_job(self, job_id)
    }

    fn get_paper(&self, paper_id: &PaperId) -> Option<Paper> {
        SqliteAppStateStore::get_paper(self, paper_id)
    }

    fn save_job(&mut self, job: OcrJob) -> bool {
        SqliteAppStateStore::save_job(self, job).is_ok()
    }

    fn save_completed_ocr(
        &mut self,
        completed_job: OcrJob,
        ocr_result: OcrJobResult,
        artifacts: TextImportArtifacts,
        updated_paper: Paper,
    ) -> bool {
        SqliteAppStateStore::save_completed_ocr(
            self,
            completed_job,
            ocr_result,
            artifacts,
            updated_paper,
        )
        .is_ok()
    }

    fn first_queued_job(&self) -> Option<OcrJob> {
        SqliteAppStateStore::first_queued_job(self)
    }

    fn managed_pdf_asset_path(&self, paper_id: &PaperId) -> std::path::PathBuf {
        SqliteAppStateStore::managed_pdf_asset_path(self, paper_id)
    }

    fn claim_job(&mut self, job_id: &OcrJobId, started_at_ms: u64) -> Option<OcrJob> {
        SqliteAppStateStore::claim_job(self, job_id, started_at_ms)
            .ok()
            .flatten()
    }

    fn claim_next_queued_job(&mut self, started_at_ms: u64) -> Option<OcrJob> {
        SqliteAppStateStore::claim_next_queued_job(self, started_at_ms)
            .ok()
            .flatten()
    }
}

pub struct LocalOcrExecutor<Store, Runner, TimestampFactory> {
    store: Store,
    runner: Runner,
    now_ms: TimestampFactory,
}

impl<Store, Runner, TimestampFactory> LocalOcrExecutor<Store, Runner, TimestampFactory>
where
    Store: LocalOcrStateStore,
    Runner: PaddleOcrRunner,
    TimestampFactory: FnMut() -> u64,
{
    pub fn new(store: Store, runner: Runner, now_ms: TimestampFactory) -> Self {
        Self {
            store,
            runner,
            now_ms,
        }
    }

    pub fn run_job(&mut self, job_id: &OcrJobId) -> Option<OcrJob> {
        let started_at_ms = (self.now_ms)();
        let processing_job = self.store.claim_job(job_id, started_at_ms)?;
        self.run_claimed_job(processing_job)
    }

    pub fn run_next_job(&mut self) -> Option<OcrJob> {
        let started_at_ms = (self.now_ms)();
        let processing_job = self.store.claim_next_queued_job(started_at_ms)?;
        self.run_claimed_job(processing_job)
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn runner(&self) -> &Runner {
        &self.runner
    }

    fn run_claimed_job(&mut self, processing_job: OcrJob) -> Option<OcrJob> {
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
                            if !self.store.save_job(failed_job) {
                                return None;
                            }
                            return None;
                        }
                    };
                let completed_job = processing_job.mark_completed(finished_at_ms).ok()?;
                let mut updated_paper = paper;
                updated_paper.status = PaperStatus::ReadyText;
                if !self.store.save_completed_ocr(
                    completed_job.clone(),
                    OcrJobResult {
                        paper_id: completed_job.paper_id.clone(),
                        text,
                    },
                    artifacts,
                    updated_paper,
                ) {
                    return None;
                }
                Some(completed_job)
            }
            Err(error_message) => {
                let finished_at_ms = (self.now_ms)();
                let failed_job = processing_job
                    .mark_failed(finished_at_ms, error_message)
                    .ok()?;
                if !self.store.save_job(failed_job) {
                    return None;
                }
                None
            }
        }
    }
}
