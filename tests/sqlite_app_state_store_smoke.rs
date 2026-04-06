use paper_immersive_reader_application::persistence::{SqliteAppStateStore, DEFAULT_APP_DB_PATH};

#[test]
fn sqlite_store_creates_database_and_default_schema() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-sqlite-smoke");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");
    let db_path = temp_root.join("app-state.db");

    let store = SqliteAppStateStore::open(&db_path).expect("sqlite store should open");

    assert!(db_path.exists());
    assert_eq!(DEFAULT_APP_DB_PATH, "database/app-state.db");
    assert!(store.first_queued_job().is_none());
}
