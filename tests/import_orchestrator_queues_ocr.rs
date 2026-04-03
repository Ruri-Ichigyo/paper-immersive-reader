use paper_immersive_reader::orchestration::{ImportOrchestrationResult, ImportOrchestrator, OcrJobService};
use paper_immersive_reader_core::domain::{
    Block, BlockId, OcrEngine, OcrJob, OcrJobId, OcrJobRepository, OcrJobResult, OcrJobStatus,
    Paper, PaperId, PaperSourceType, PaperStatus, ReadingProgress,
};
use paper_immersive_reader_importing::{
    ImportPipeline, ImportPipelineError, ImportRequest, ImportResult,
};

#[test]
fn import_orchestrator_queues_ocr_job_when_pipeline_requires_ocr() {
    let paper_id = PaperId("paper-ocr-1".to_string());
    let job_id = OcrJobId("job-ocr-1".to_string());
    let requested_at_ms = 1_717_171_717;

    let pipeline = OcrRequiredPipeline {
        paper_id: paper_id.clone(),
    };
    let repository = InMemoryOcrJobRepository::default();
    let expected_job_id = job_id.clone();
    let job_service = OcrJobService::new(repository, move || job_id.clone(), move || requested_at_ms);
    let mut orchestrator = ImportOrchestrator::new_with_ocr_job_service(pipeline, job_service);

    let result = orchestrator
        .import(ImportRequest::PasteText {
            text: "scanned pdf placeholder".to_string(),
            title: Some("Needs OCR".to_string()),
            source_ref: Some("file:///tmp/paper.pdf".to_string()),
        })
        .expect("orchestrator should queue OCR when requested");

    assert_eq!(
        result,
        ImportOrchestrationResult::QueuedOcr {
            paper_id: paper_id.clone(),
            job_id: expected_job_id.clone(),
        }
    );

    let jobs = orchestrator.ocr_job_service().list_jobs_for_paper(&paper_id);
    assert_eq!(jobs.len(), 1);

    let job = &jobs[0];
    assert_eq!(job.id, expected_job_id);
    assert_eq!(job.paper_id, paper_id);
    assert_eq!(job.engine, OcrEngine::PaddleOcr);
    assert_eq!(job.status, OcrJobStatus::Queued);
    assert_eq!(job.requested_at_ms, requested_at_ms);
    assert_eq!(job.started_at_ms, None);
    assert_eq!(job.finished_at_ms, None);
    assert_eq!(job.error_message, None);
}

#[test]
fn import_orchestrator_returns_imported_text_when_pipeline_succeeds_with_ocr_service_present() {
    let repository = InMemoryOcrJobRepository::default();
    let job_service = OcrJobService::new(repository, || OcrJobId("unused-job".to_string()), || 999);
    let mut orchestrator = ImportOrchestrator::new_with_ocr_job_service(SuccessPipeline, job_service);

    let result = orchestrator
        .import(ImportRequest::PasteText {
            text: "plain text".to_string(),
            title: Some("Ready".to_string()),
            source_ref: None,
        })
        .expect("successful imports should still work");

    assert_eq!(result, ImportOrchestrationResult::ImportedText);
    assert!(orchestrator
        .ocr_job_service()
        .list_jobs_for_paper(&PaperId("paper-ready-1".to_string()))
        .is_empty());
}

#[test]
fn import_orchestrator_keeps_non_ocr_errors_as_errors() {
    let repository = InMemoryOcrJobRepository::default();
    let job_service = OcrJobService::new(repository, || OcrJobId("unused-job".to_string()), || 999);
    let mut orchestrator = ImportOrchestrator::new_with_ocr_job_service(EmptyInputPipeline, job_service);

    let error = orchestrator
        .import(ImportRequest::PasteText {
            text: "".to_string(),
            title: None,
            source_ref: None,
        })
        .expect_err("non-OCR pipeline errors should pass through");

    assert_eq!(error, ImportPipelineError::EmptyInput);
    assert!(orchestrator
        .ocr_job_service()
        .list_jobs_for_paper(&PaperId("paper-ignored".to_string()))
        .is_empty());
}

struct OcrRequiredPipeline {
    paper_id: PaperId,
}

impl ImportPipeline for OcrRequiredPipeline {
    fn import(&self, _request: ImportRequest) -> Result<ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::OcrRequired {
            paper_id: self.paper_id.clone(),
        })
    }
}

struct SuccessPipeline;

impl ImportPipeline for SuccessPipeline {
    fn import(&self, _request: ImportRequest) -> Result<ImportResult, ImportPipelineError> {
        Ok(ImportResult {
            paper: Paper {
                id: PaperId("paper-ready-1".to_string()),
                title: Some("Ready".to_string()),
                source_type: PaperSourceType::Paste,
                source_ref: None,
                status: PaperStatus::ReadyText,
                parse_version: 1,
            },
            blocks: vec![
                Block::try_new(
                    BlockId("block-ready-1".to_string()),
                    PaperId("paper-ready-1".to_string()),
                    0,
                    "plain text".to_string(),
                    None,
                )
                .expect("block should be valid"),
            ],
            reading_progress: ReadingProgress {
                paper_id: PaperId("paper-ready-1".to_string()),
                current_block_index: 0,
                current_block_id: Some(BlockId("block-ready-1".to_string())),
                updated_at_ms: 0,
            },
        })
    }
}

struct EmptyInputPipeline;

impl ImportPipeline for EmptyInputPipeline {
    fn import(&self, _request: ImportRequest) -> Result<ImportResult, ImportPipelineError> {
        Err(ImportPipelineError::EmptyInput)
    }
}

#[derive(Default)]
struct InMemoryOcrJobRepository {
    jobs: Vec<OcrJob>,
    results: Vec<OcrJobResult>,
}

impl OcrJobRepository for InMemoryOcrJobRepository {
    fn create_job(&mut self, job: OcrJob) {
        self.jobs.push(job);
    }

    fn get_job(&self, id: &OcrJobId) -> Option<&OcrJob> {
        self.jobs.iter().find(|job| &job.id == id)
    }

    fn list_jobs_for_paper(&self, paper_id: &PaperId) -> Vec<&OcrJob> {
        self.jobs
            .iter()
            .filter(|job| &job.paper_id == paper_id)
            .collect()
    }

    fn save_result(&mut self, result: OcrJobResult) {
        self.results.push(result);
    }
}
