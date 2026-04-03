use paper_immersive_reader_core::domain::{OcrJobId, PaperId};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};

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

impl<P, R, JobIdFactory, PaperIdFactory, TimestampFactory>
    ImportOrchestrator<
        P,
        crate::orchestration::OcrJobService<R, JobIdFactory, PaperIdFactory, TimestampFactory>,
    >
where
    P: ImportPipeline,
    R: paper_immersive_reader_core::domain::OcrJobRepository,
    JobIdFactory: FnMut() -> OcrJobId,
    PaperIdFactory: FnMut() -> PaperId,
    TimestampFactory: FnMut() -> u64,
{
    pub fn import(
        &mut self,
        request: ImportRequest,
    ) -> Result<ImportOrchestrationResult, ImportPipelineError> {
        match self.pipeline.import(request) {
            Ok(_) => Ok(ImportOrchestrationResult::ImportedText),
            Err(ImportPipelineError::OcrRequired) => {
                let (paper_id, job_id) = self.ocr_job_service.queue_paddle_ocr();
                Ok(ImportOrchestrationResult::QueuedOcr { paper_id, job_id })
            }
            Err(err) => Err(err),
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOrchestrationResult {
    ImportedText,
    QueuedOcr { paper_id: PaperId, job_id: OcrJobId },
}
