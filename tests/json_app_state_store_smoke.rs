use paper_immersive_reader_application::persistence::{JsonAppStateStore, DEFAULT_APP_STATE_PATH};
use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};

#[test]
fn json_app_state_store_loads_missing_file_as_empty_and_round_trips_papers() {
    let temp_path = std::env::temp_dir().join("paper-immersive-reader-json-app-state-smoke.json");
    let _ = std::fs::remove_file(&temp_path);

    let mut store = JsonAppStateStore::load_or_default(&temp_path).expect("store should load");
    assert!(store.snapshot().papers.is_empty());
    assert_eq!(DEFAULT_APP_STATE_PATH, "database/app-state.json");

    store.snapshot_mut().papers.push(Paper {
        id: PaperId("paper-1".to_string()),
        title: Some("persisted".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    });
    store.save().expect("store should save");

    let reloaded = JsonAppStateStore::load_or_default(&temp_path).expect("store should reload");
    assert_eq!(reloaded.snapshot().papers.len(), 1);
}
