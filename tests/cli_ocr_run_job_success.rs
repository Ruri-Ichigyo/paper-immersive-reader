#[path = "support/cli.rs"]
mod cli_support;

use paper_immersive_reader_application::persistence::JsonAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

#[test]
fn cli_ocr_run_executes_only_the_requested_job() {
    let temp_root = cli_support::create_temp_workspace("paper-immersive-reader-cli-run-job");
    let state_path = temp_root.join("database").join("app-state.json");

    let mut store = JsonAppStateStore::load_or_default(&state_path).expect("store should load");
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("scan-1".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan-1.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-2".to_string()),
        title: Some("scan-2".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan-2.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-2".to_string()),
        paper_id: PaperId("paper-2".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 2,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store
        .persist_managed_pdf_asset(&PaperId("paper-1".to_string()), b"%PDF asset 1")
        .expect("first asset should persist");
    store
        .persist_managed_pdf_asset(&PaperId("paper-2".to_string()), b"%PDF asset 2")
        .expect("second asset should persist");
    store.save().expect("store should save");

    cli_support::write_executable_script(
        &temp_root.join("scripts").join("run_paddle_ocr.sh"),
        "#!/bin/sh\nprintf 'recognized text\\n'\n",
    );

    let output = cli_support::run_cli(&temp_root, &["ocr", "run", "job-2"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Completed OCR job job-2"));

    let reloaded = JsonAppStateStore::load_or_default(&state_path).expect("store should reload");
    assert_eq!(reloaded.snapshot().ocr_jobs[0].status, OcrJobStatus::Queued);
    assert_eq!(
        reloaded.snapshot().ocr_jobs[1].status,
        OcrJobStatus::Completed
    );
}
