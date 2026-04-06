use paper_immersive_reader_application::persistence::{JsonAppStateStore, DEFAULT_APP_DB_PATH};
use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};

#[test]
fn json_app_state_store_remains_available_for_migration_input_while_sqlite_is_default_runtime() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-json-app-state-smoke.json");
    let _ = std::fs::remove_file(&temp_path);

    let mut store = JsonAppStateStore::load_migration_input(&temp_path).expect("store should load");
    assert!(store.snapshot().papers.is_empty());
    assert_eq!(DEFAULT_APP_DB_PATH, "database/app-state.db");

    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("persisted".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.save().expect("store should save");

    let reloaded =
        JsonAppStateStore::load_migration_input(&temp_path).expect("store should reload");
    assert_eq!(reloaded.snapshot().papers.len(), 1);
}
