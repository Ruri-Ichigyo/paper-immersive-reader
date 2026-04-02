use paper_immersive_reader_core::domain::{OcrEngine, OcrJob, OcrJobId, OcrJobStatus, PaperId};

#[test]
fn ocr_job_can_be_constructed() {
    let job = OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    };

    assert_eq!(job.id, OcrJobId("job-1".to_string()));
    assert_eq!(job.paper_id, PaperId("paper-1".to_string()));
    assert_eq!(job.engine, OcrEngine::PaddleOcr);
    assert_eq!(job.status, OcrJobStatus::Queued);
    assert_eq!(job.requested_at_ms, 1);
    assert_eq!(job.started_at_ms, None);
    assert_eq!(job.finished_at_ms, None);
    assert_eq!(job.error_message, None);
}
