#[path = "support/cli.rs"]
mod cli_support;

use paper_immersive_reader_application::persistence::SqliteAppStateStore;
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

#[test]
fn cli_ocr_run_executes_only_the_requested_sqlite_job() {
    let temp_root = cli_support::create_temp_workspace("paper-immersive-reader-cli-run-job-sqlite");
    let db_path = temp_root.join("database").join("app-state.db");

    let mut store = SqliteAppStateStore::open(&db_path).expect("store should open");
    let paper_1_id = PaperId("paper-1".to_string());
    let paper_2_id = PaperId("paper-2".to_string());
    store
        .save_paper_and_job(
            Paper {
                id: paper_1_id.clone(),
                title: Some("scan-1".to_string()),
                source_type: PaperSourceType::Pdf,
                source_ref: Some("scan-1.pdf".to_string()),
                status: PaperStatus::ReadyOcr,
                parse_version: 1,
            },
            OcrJob {
                id: OcrJobId("job-1".to_string()),
                paper_id: paper_1_id.clone(),
                engine: OcrEngine::PaddleOcr,
                status: OcrJobStatus::Queued,
                requested_at_ms: 1,
                started_at_ms: None,
                finished_at_ms: None,
                error_message: None,
            },
        )
        .expect("first seed should save");
    store
        .save_paper_and_job(
            Paper {
                id: paper_2_id.clone(),
                title: Some("scan-2".to_string()),
                source_type: PaperSourceType::Pdf,
                source_ref: Some("scan-2.pdf".to_string()),
                status: PaperStatus::ReadyOcr,
                parse_version: 1,
            },
            OcrJob {
                id: OcrJobId("job-2".to_string()),
                paper_id: paper_2_id.clone(),
                engine: OcrEngine::PaddleOcr,
                status: OcrJobStatus::Queued,
                requested_at_ms: 2,
                started_at_ms: None,
                finished_at_ms: None,
                error_message: None,
            },
        )
        .expect("second seed should save");
    store
        .persist_managed_pdf_asset(&paper_1_id, b"%PDF asset 1")
        .expect("first asset should persist");
    store
        .persist_managed_pdf_asset(&paper_2_id, b"%PDF asset 2")
        .expect("second asset should persist");

    cli_support::write_executable_script(
        &temp_root.join("scripts").join("run_paddle_ocr.sh"),
        "#!/bin/sh\nprintf 'recognized text\\n'\n",
    );

    let output = cli_support::run_cli(&temp_root, &["ocr", "run", "job-2"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Completed OCR job job-2"));

    let snapshot = SqliteAppStateStore::open(&db_path)
        .expect("store should reload")
        .snapshot()
        .expect("snapshot should load");
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Queued);
    assert_eq!(snapshot.ocr_jobs[1].status, OcrJobStatus::Completed);
}
