#[path = "support/cli.rs"]
mod cli_support;

use paper_immersive_reader_application::persistence::{JsonAppStateStore, SqliteAppStateStore};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

#[test]
fn cli_ocr_run_next_auto_migrates_legacy_json_before_running_job() {
    let temp_root = cli_support::create_temp_workspace("paper-immersive-reader-cli-auto-migrate");
    let json_path = temp_root.join("database").join("app-state.json");
    let db_path = temp_root.join("database").join("app-state.db");
    let migrated_json_path = temp_root.join("database").join("app-state.migrated.json");

    let mut store = JsonAppStateStore::load_migration_input(&json_path).expect("store should load");
    let paper_id = PaperId("paper-1".to_string());
    store.snapshot_mut().papers.push(Paper {
        id: paper_id.clone(),
        title: Some("scan".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: paper_id.clone(),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    store
        .persist_managed_pdf_asset(&paper_id, b"%PDF fake asset")
        .expect("asset should persist");
    store.save().expect("legacy json should save");

    cli_support::write_executable_script(
        &temp_root.join("scripts").join("run_paddle_ocr.sh"),
        "#!/bin/sh\nprintf 'recognized text\\n'\n",
    );

    let output = cli_support::run_cli(&temp_root, &["ocr", "run-next"]);

    assert!(output.status.success());
    assert!(db_path.exists());
    assert!(migrated_json_path.exists());

    let snapshot = SqliteAppStateStore::open(&db_path)
        .expect("store should reload")
        .snapshot()
        .expect("snapshot should load");
    assert_eq!(snapshot.ocr_jobs[0].status, OcrJobStatus::Completed);
}
