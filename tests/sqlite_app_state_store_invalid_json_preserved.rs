use paper_immersive_reader_application::persistence::SqliteAppStateStore;

#[test]
fn sqlite_store_preserves_invalid_legacy_json_on_migration_failure() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-sqlite-invalid-json");
    let _ = std::fs::remove_dir_all(&temp_root);
    let database_dir = temp_root.join("database");
    std::fs::create_dir_all(&database_dir).expect("database dir should exist");

    let json_path = database_dir.join("app-state.json");
    let db_path = database_dir.join("app-state.db");
    let migrated_json_path = database_dir.join("app-state.migrated.json");

    std::fs::write(&json_path, "{ invalid json").expect("invalid json should be written");

    let result = SqliteAppStateStore::open_or_migrate(&db_path);

    assert!(result.is_err());
    assert!(json_path.exists());
    assert!(!migrated_json_path.exists());
}
