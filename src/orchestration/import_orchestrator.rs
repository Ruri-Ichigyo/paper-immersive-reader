use paper_immersive_reader_core::domain::{OcrJobId, PaperId};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};

use crate::orchestration::OcrJobService;

pub struct ImportOrchestrator<P, S = ()> {
    pipeline: P,
    ocr_job_service: S,
}

impl<P> ImportOrchestrator<P> {
    pub fn new(pipeline: P) -> Self {
        Self {
            pipeline,
            ocr_job_service: (),
        }
    }
}

impl<P, S> ImportOrchestrator<P, S> {
    pub fn new_with_ocr_job_service(pipeline: P, ocr_job_service: S) -> Self {
        Self {
            pipeline,
            ocr_job_service,
        }
    }

    pub fn ocr_job_service(&self) -> &S {
        &self.ocr_job_service
    }
}

impl<P> ImportOrchestrator<P>
where
    P: ImportPipeline,
{
    pub fn import(
        &self,
        request: ImportRequest,
    ) -> Result<ImportOrchestrationResult, ImportPipelineError> {
        self.pipeline.import(request)?;
        Ok(ImportOrchestrationResult::ImportedText)
    }
}

impl<P, R, IdFactory, TimestampFactory>
    ImportOrchestrator<P, OcrJobService<R, IdFactory, TimestampFactory>>
where
    P: ImportPipeline,
    R: paper_immersive_reader_core::domain::OcrJobRepository,
    IdFactory: FnMut() -> OcrJobId,
    TimestampFactory: FnMut() -> u64,
{
    pub fn import(
        &mut self,
        request: ImportRequest,
    ) -> Result<ImportOrchestrationResult, ImportPipelineError> {
        match self.pipeline.import(request) {
            Ok(_) => Ok(ImportOrchestrationResult::ImportedText),
            Err(ImportPipelineError::OcrRequired { paper_id }) => {
                let job_id = self.ocr_job_service.queue_paddle_ocr(paper_id.clone());
                Ok(ImportOrchestrationResult::QueuedOcr { paper_id, job_id })
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOrchestrationResult {
    ImportedText,
    QueuedOcr { paper_id: PaperId, job_id: OcrJobId },
}
