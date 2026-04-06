use paper_immersive_reader_application::persistence::{JsonAppStateStore, SqliteAppStateStore};
use paper_immersive_reader_core::domain::{
    OcrEngine, OcrJob, OcrJobId, OcrJobStatus, Paper, PaperId, PaperSourceType, PaperStatus,
};

#[test]
fn sqlite_store_migrates_legacy_json_and_renames_source_file() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-sqlite-migrates-json");
    let _ = std::fs::remove_dir_all(&temp_root);
    let database_dir = temp_root.join("database");
    std::fs::create_dir_all(&database_dir).expect("database dir should exist");

    let json_path = database_dir.join("app-state.json");
    let db_path = database_dir.join("app-state.db");
    let migrated_json_path = database_dir.join("app-state.migrated.json");

    let mut json_store =
        JsonAppStateStore::load_migration_input(&json_path).expect("json store should load");
    json_store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("Legacy paper".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("legacy.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    json_store.snapshot_mut().ocr_jobs.push(OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 123,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    });
    json_store.save().expect("legacy json should save");

    let store = SqliteAppStateStore::open_or_migrate(&db_path).expect("migration should succeed");

    assert!(db_path.exists());
    assert!(migrated_json_path.exists());
    assert!(!json_path.exists());
    assert_eq!(
        store
            .get_paper(&PaperId("paper-1".to_string()))
            .map(|paper| paper.title),
        Some(Some("Legacy paper".to_string()))
    );
    assert_eq!(
        store.first_queued_job().map(|job| job.id),
        Some(OcrJobId("job-1".to_string()))
    );
}
