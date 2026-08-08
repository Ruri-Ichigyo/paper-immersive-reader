use paper_immersive_reader_core::domain::{BlockId, PaperId, VocabularyId, VocabularyItem};
use paper_immersive_reader_web_app::storage::{memory_store::InMemoryStore, AppStorage};

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[test]
fn vocabulary_upsert_increments_lookup_count() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-1".to_string());
    let block_id = BlockId("b-1".to_string());
    let now = current_time_ms();

    // First lookup
    let item = VocabularyItem {
        id: VocabularyId("v-1".to_string()),
        term: "epistemology".to_string(),
        term_lower: "epistemology".to_string(),
        paper_id_first_seen: paper_id.clone(),
        block_id_first_seen: block_id.clone(),
        lookup_count: 1,
        first_seen_at_ms: now,
        last_seen_at_ms: now,
        note: None,
        learned: false,
    };
    store.upsert_vocabulary(&item).unwrap();

    // Second lookup of the same term
    let item2 = VocabularyItem {
        id: VocabularyId("v-2".to_string()),
        term: "epistemology".to_string(),
        term_lower: "epistemology".to_string(),
        paper_id_first_seen: PaperId("p-2".to_string()),
        block_id_first_seen: BlockId("b-3".to_string()),
        lookup_count: 1,
        first_seen_at_ms: now + 1000,
        last_seen_at_ms: now + 1000,
        note: Some("branch of philosophy".to_string()),
        learned: false,
    };
    store.upsert_vocabulary(&item2).unwrap();

    // Should exist only once with lookup_count incremented
    let entries = store.list_vocabulary().unwrap();
    assert_eq!(entries.len(), 1, "term should not be duplicated");
    assert_eq!(entries[0].term, "epistemology");
    assert_eq!(
        entries[0].lookup_count, 2,
        "lookup_count should be incremented"
    );
    assert_eq!(
        entries[0].paper_id_first_seen, paper_id,
        "should keep first-seen paper"
    );
    assert_eq!(
        entries[0].note.as_deref(),
        Some("branch of philosophy"),
        "note from later lookup should be preserved"
    );
}
