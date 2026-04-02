use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, OcrJobTransitionError, PaperId,
};

fn queued_job() -> OcrJob {
    OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    }
}

#[test]
fn queued_job_can_transition_to_processing() {
    let updated = queued_job().mark_processing(10).unwrap();

    assert_eq!(updated.status, OcrJobStatus::Processing);
    assert_eq!(updated.started_at_ms, Some(10));
    assert_eq!(updated.finished_at_ms, None);
    assert_eq!(updated.error_message, None);
}

#[test]
fn processing_job_can_transition_to_completed() {
    let job = queued_job().mark_processing(10).unwrap();

    let updated = job.mark_completed(20).unwrap();

    assert_eq!(updated.status, OcrJobStatus::Completed);
    assert_eq!(updated.started_at_ms, Some(10));
    assert_eq!(updated.finished_at_ms, Some(20));
    assert_eq!(updated.error_message, None);
}

#[test]
fn processing_job_can_transition_to_failed() {
    let job = queued_job().mark_processing(10).unwrap();

    let updated = job
        .mark_failed(20, "paddleocr crashed".to_string())
        .unwrap();

    assert_eq!(updated.status, OcrJobStatus::Failed);
    assert_eq!(updated.started_at_ms, Some(10));
    assert_eq!(updated.finished_at_ms, Some(20));
    assert_eq!(updated.error_message, Some("paddleocr crashed".to_string()));
}

#[test]
fn rejects_invalid_status_transitions() {
    let processing = queued_job().mark_processing(10).unwrap();
    let completed = processing.clone().mark_completed(20).unwrap();
    let failed = processing
        .clone()
        .mark_failed(30, "ocr failed".to_string())
        .unwrap();

    assert_eq!(
        queued_job().mark_completed(20),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
    assert_eq!(
        queued_job().mark_failed(20, "ocr failed".to_string()),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
    assert_eq!(
        processing.clone().mark_processing(11),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
    assert_eq!(
        completed.mark_processing(21),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
    assert_eq!(
        failed.mark_processing(31),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
    assert_eq!(
        processing
            .clone()
            .mark_completed(20)
            .unwrap()
            .mark_failed(21, "ocr failed".to_string()),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
    assert_eq!(
        processing
            .mark_failed(20, "ocr failed".to_string())
            .unwrap()
            .mark_completed(21),
        Err(OcrJobTransitionError::InvalidStatusTransition)
    );
}
