use paper_immersive_reader_core::domain::PaperId;
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::schema::ReadingStats;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn reading_stats_default_to_zero() {
    let store = InMemoryStore::new();
    let pid = PaperId("p-1".to_string());
    assert!(store.load_reading_stats(&pid).unwrap().is_none());
}

#[test]
fn reading_stats_save_and_load() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("p-1".to_string());
    let stats = ReadingStats {
        paper_id: pid.clone(),
        total_reading_ms: 120_000,
        session_started_ms: Some(1_000),
        session_count: 3,
    };
    store.save_reading_stats(&stats).unwrap();

    let loaded = store.load_reading_stats(&pid).unwrap().unwrap();
    assert_eq!(loaded.total_reading_ms, 120_000);
    assert_eq!(loaded.session_count, 3);
    assert_eq!(loaded.session_started_ms, Some(1_000));
}

#[test]
fn reading_stats_overwrite_on_second_save() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("p-1".to_string());

    let stats = ReadingStats {
        paper_id: pid.clone(),
        total_reading_ms: 60_000,
        session_started_ms: Some(1_000),
        session_count: 1,
    };
    store.save_reading_stats(&stats).unwrap();

    // Simulate end of session: accumulate and clear session_started_ms.
    let mut updated = stats.clone();
    updated.total_reading_ms += 30_000;
    updated.session_count = 2;
    updated.session_started_ms = None;
    store.save_reading_stats(&updated).unwrap();

    let loaded = store.load_reading_stats(&pid).unwrap().unwrap();
    assert_eq!(loaded.total_reading_ms, 90_000);
    assert_eq!(loaded.session_count, 2);
    assert_eq!(loaded.session_started_ms, None);
}

#[test]
fn list_reading_stats_returns_all() {
    let mut store = InMemoryStore::new();
    for i in 0..3 {
        let pid = PaperId(format!("p-{}", i));
        let stats = ReadingStats::new(pid);
        store.save_reading_stats(&stats).unwrap();
    }
    let all = store.list_reading_stats().unwrap();
    assert_eq!(all.len(), 3);
}
