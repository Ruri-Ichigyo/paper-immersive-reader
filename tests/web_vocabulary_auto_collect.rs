use paper_immersive_reader_core::domain::{BlockId, PaperId, VocabularyId, VocabularyItem};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[test]
fn lookup_auto_collects_and_merges_vocabulary() {
    let mut store = InMemoryStore::new();
    let paper_id = PaperId("p-vocab".to_string());
    let block_id = BlockId("b-vocab".to_string());
    let now = current_time_ms();

    // Simulate first lookup of a term
    let item1 = VocabularyItem {
        id: VocabularyId("v-100".to_string()),
        term: "hermeneutics".to_string(),
        term_lower: "hermeneutics".to_string(),
        paper_id_first_seen: paper_id.clone(),
        block_id_first_seen: block_id.clone(),
        lookup_count: 1,
        first_seen_at_ms: now,
        last_seen_at_ms: now,
        note: None,
        learned: false,
    };
    store.upsert_vocabulary(&item1).unwrap();

    // Simulate second lookup of the same term
    let item2 = VocabularyItem {
        id: VocabularyId("v-200".to_string()),
        term: "hermeneutics".to_string(),
        term_lower: "hermeneutics".to_string(),
        paper_id_first_seen: PaperId("p-other".to_string()),
        block_id_first_seen: BlockId("b-other".to_string()),
        lookup_count: 1,
        first_seen_at_ms: now + 5000,
        last_seen_at_ms: now + 5000,
        note: Some("theory of interpretation".to_string()),
        learned: false,
    };
    store.upsert_vocabulary(&item2).unwrap();

    // Verify single entry with merged data
    let entries = store.list_vocabulary().unwrap();
    assert_eq!(entries.len(), 1, "should merge duplicate terms");
    assert_eq!(entries[0].term, "hermeneutics");
    assert_eq!(entries[0].lookup_count, 2);
    assert_eq!(
        entries[0].paper_id_first_seen, paper_id,
        "first-seen paper preserved"
    );
    assert_eq!(
        entries[0].last_seen_at_ms,
        now + 5000,
        "last-seen time updated"
    );
    assert_eq!(
        entries[0].note.as_deref(),
        Some("theory of interpretation"),
        "note from later lookup preserved"
    );
}

#[test]
fn vocabulary_list_shows_all_collected_terms() {
    let mut store = InMemoryStore::new();
    let now = current_time_ms();

    let terms = ["ontology", "epistemology", "phenomenology"];
    for (i, term) in terms.iter().enumerate() {
        let item = VocabularyItem {
            id: VocabularyId(format!("v-{}", i)),
            term: term.to_string(),
            term_lower: term.to_lowercase(),
            paper_id_first_seen: PaperId("p-vlist".to_string()),
            block_id_first_seen: BlockId("b-vlist".to_string()),
            lookup_count: 1,
            first_seen_at_ms: now,
            last_seen_at_ms: now,
            note: None,
            learned: false,
        };
        store.upsert_vocabulary(&item).unwrap();
    }

    let entries = store.list_vocabulary().unwrap();
    assert_eq!(entries.len(), 3);
    let mut terms_found: Vec<String> = entries.iter().map(|e| e.term.clone()).collect();
    terms_found.sort();
    assert_eq!(
        terms_found,
        vec!["epistemology", "ontology", "phenomenology"]
    );
}
