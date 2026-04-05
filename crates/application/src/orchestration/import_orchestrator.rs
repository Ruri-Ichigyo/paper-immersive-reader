use crate::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{OcrJobId, PaperId};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};

pub struct ImportOrchestrator<P, Store = (), S = ()> {
    pipeline: P,
    store: Store,
    ocr_job_service: S,
}

impl<P> ImportOrchestrator<P> {
    pub fn new(pipeline: P) -> Self {
        Self {
            pipeline,
            store: (),
            ocr_job_service: (),
        }
    }
}

impl<P, Store> ImportOrchestrator<P, Store> {
    pub fn new_with_store(pipeline: P, store: Store) -> Self {
        Self {
            pipeline,
            store,
            ocr_job_service: (),
        }
    }
}

impl<P, S> ImportOrchestrator<P, (), S> {
    pub fn new_with_ocr_job_service(pipeline: P, ocr_job_service: S) -> Self {
        Self {
            pipeline,
            store: (),
            ocr_job_service,
        }
    }
}

impl<P, Store, S> ImportOrchestrator<P, Store, S> {
    pub fn new_with_store_and_ocr_job_service(
        pipeline: P,
        store: Store,
        ocr_job_service: S,
    ) -> Self {
        Self {
            pipeline,
            store,
            ocr_job_service,
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
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

impl<P> ImportOrchestrator<P, JsonAppStateStore, ()>
where
    P: ImportPipeline,
{
    pub fn import(
        &mut self,
        request: ImportRequest,
    ) -> Result<ImportOrchestrationResult, ImportPipelineError> {
        let pdf_bytes = match &request {
            ImportRequest::PdfDocument { bytes, .. } => Some(bytes.clone()),
            _ => None,
        };
        let result = self.pipeline.import(request)?;
        if let Some(bytes) = pdf_bytes {
            self.store
                .persist_managed_pdf_asset(&result.paper.id, &bytes)
                .map_err(|_| ImportPipelineError::PersistenceFailed)?;
        }
        self.store
            .save_import_result(result)
            .map_err(|_| ImportPipelineError::PersistenceFailed)?;
        Ok(ImportOrchestrationResult::ImportedText)
    }
}

impl<P, JobIdFactory, PaperIdFactory, TimestampFactory>
    ImportOrchestrator<
        P,
        JsonAppStateStore,
        crate::orchestration::OcrJobService<JobIdFactory, PaperIdFactory, TimestampFactory>,
    >
where
    P: ImportPipeline,
    JobIdFactory: FnMut() -> OcrJobId,
    PaperIdFactory: FnMut() -> PaperId,
    TimestampFactory: FnMut() -> u64,
{
    pub fn import(
        &mut self,
        request: ImportRequest,
    ) -> Result<ImportOrchestrationResult, ImportPipelineError> {
        let pdf_request = match &request {
            ImportRequest::PdfDocument {
                bytes,
                title,
                source_ref,
            } => Some((bytes.clone(), title.clone(), source_ref.clone())),
            _ => None,
        };

        match self.pipeline.import(request) {
            Ok(result) => {
                if let Some((bytes, _, _)) = pdf_request {
                    self.store
                        .persist_managed_pdf_asset(&result.paper.id, &bytes)
                        .map_err(|_| ImportPipelineError::PersistenceFailed)?;
                }
                self.store
                    .save_import_result(result)
                    .map_err(|_| ImportPipelineError::PersistenceFailed)?;
                Ok(ImportOrchestrationResult::ImportedText)
            }
            Err(ImportPipelineError::OcrRequired) => {
                let (bytes, title, source_ref) =
                    pdf_request.ok_or(ImportPipelineError::OcrRequired)?;
                let paper = self
                    .ocr_job_service
                    .build_ready_ocr_paper(title, source_ref);
                let job = self.ocr_job_service.build_queued_job(paper.id.clone());
                let paper_id = paper.id.clone();
                let job_id = job.id.clone();

                self.store
                    .persist_managed_pdf_asset(&paper.id, &bytes)
                    .map_err(|_| ImportPipelineError::PersistenceFailed)?;
                self.store
                    .save_paper_and_job(paper, job)
                    .map_err(|_| ImportPipelineError::PersistenceFailed)?;

                Ok(ImportOrchestrationResult::QueuedOcr { paper_id, job_id })
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOrchestrationResult {
    ImportedText,
    QueuedOcr { paper_id: PaperId, job_id: OcrJobId },
}
