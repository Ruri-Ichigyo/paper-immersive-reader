use paper_immersive_reader::orchestration::{
    ImportOrchestrationResult, ImportOrchestrator, OcrJobService,
};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus, PaperId,
};
use paper_immersive_reader_importing::{ImportPipeline, ImportPipelineError, ImportRequest};
use std::cell::RefCell;

#[test]
fn import_orchestrator_queues_ocr_when_pipeline_requires_it() {
    let repository = InMemoryOcrJobRepository::default();
    let ocr_job_service = OcrJobService::new(
        repository,
        || OcrJobId("job-1".to_string()),
        || PaperId("paper-ocr-1".to_string()),
        || 1_700_000_000_000,
    );
    let mut orchestrator =
        ImportOrchestrator::new_with_ocr_job_service(OcrRequiredPipeline, ocr_job_service);

    let result = orchestrator
        .import(ImportRequest::PdfDocument {
            bytes: b"fake-pdf".to_vec(),
            title: Some("scanned paper".to_string()),
            source_ref: Some("scan.pdf".to_string()),
        })
        .expect("orchestrator should queue OCR");

    assert_eq!(
        result,
        ImportOrchestrationResult::QueuedOcr {
            paper_id: PaperId("paper-ocr-1".to_string()),
            job_id: OcrJobId("job-1".to_string()),
        }
    );

    let jobs = orchestrator
        .ocr_job_service()
        .list_jobs_for_paper(&PaperId("paper-ocr-1".to_string()));
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        jobs[0],
        OcrJob {
            id: OcrJobId("job-1".to_string()),
            paper_id: PaperId("paper-ocr-1".to_string()),
            engine: OcrEngine::PaddleOcr,
            status: OcrJobStatus::Queued,
            requested_at_ms: 1_700_000_000_000,
            started_at_ms: None,
            finished_at_ms: None,
            error_message: None,
        }
    );
}

struct OcrRequiredPipeline;

impl ImportPipeline for OcrRequiredPipeline {
    fn import(
        &self,
        _request: ImportRequest,
    ) -> Result<paper_immersive_reader_importing::ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::OcrRequired)
    }
}

#[derive(Default)]
struct InMemoryOcrJobRepository {
    jobs: RefCell<Vec<OcrJob>>,
    results: RefCell<Vec<OcrJobResult>>,
}

impl OcrJobRepository for InMemoryOcrJobRepository {
    fn create_job(&mut self, job: OcrJob) {
        self.jobs.get_mut().push(job);
    }

    fn get_job(&self, id: &OcrJobId) -> Option<&OcrJob> {
        let job = self.jobs.borrow().iter().find(|job| &job.id == id).cloned();
        job.map(|job| Box::leak(Box::new(job)) as &OcrJob)
    }

    fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<&OcrJob> {
        self.jobs
            .borrow()
            .iter()
            .filter(|job| &job.paper_id == paper_id)
            .cloned()
            .map(|job| Box::leak(Box::new(job)) as &OcrJob)
            .collect()
    }

    fn replace_job(&mut self, job: OcrJob) {
        let jobs = self.jobs.get_mut();
        if let Some(existing) = jobs.iter_mut().find(|existing| existing.id == job.id) {
            *existing = job;
        } else {
            jobs.push(job);
        }
    }

    fn save_result(&mut self, result: OcrJobResult) {
        self.results.get_mut().push(result);
    }
}
