use crate::domain::PaperId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrJobId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrEngine {
    PaddleOcr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrJobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrJobTransitionError {
    InvalidStatusTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrJob {
    pub id: OcrJobId,
    pub paper_id: PaperId,
    pub engine: OcrEngine,
    pub status: OcrJobStatus,
    pub requested_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
    pub error_message: Option<String>,
}

impl OcrJob {
    pub fn mark_processing(self, started_at_ms: u64) -> Result<Self, OcrJobTransitionError> {
        if self.status != OcrJobStatus::Queued {
            return Err(OcrJobTransitionError::InvalidStatusTransition);
        }

        Ok(Self {
            status: OcrJobStatus::Processing,
            started_at_ms: Some(started_at_ms),
            ..self
        })
    }

    pub fn mark_completed(self, finished_at_ms: u64) -> Result<Self, OcrJobTransitionError> {
        if self.status != OcrJobStatus::Processing {
            return Err(OcrJobTransitionError::InvalidStatusTransition);
        }

        Ok(Self {
            status: OcrJobStatus::Completed,
            finished_at_ms: Some(finished_at_ms),
            error_message: None,
            ..self
        })
    }

    pub fn mark_failed(
        self,
        finished_at_ms: u64,
        error_message: String,
    ) -> Result<Self, OcrJobTransitionError> {
        if self.status != OcrJobStatus::Processing {
            return Err(OcrJobTransitionError::InvalidStatusTransition);
        }

        Ok(Self {
            status: OcrJobStatus::Failed,
            finished_at_ms: Some(finished_at_ms),
            error_message: Some(error_message),
            ..self
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrJobResult {
    pub paper_id: PaperId,
    pub text: String,
}

pub trait OcrJobRepository {
    fn create_job(&mut self, job: OcrJob);
    fn get_job(&self, id: &OcrJobId) -> Option<&OcrJob>;
    fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<&OcrJob>;
    fn save_result(&mut self, result: OcrJobResult);
}
