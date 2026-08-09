use paper_immersive_reader_core::domain::{BlockId, PaperId};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::schema::AiRecord;
use paper_immersive_reader_web_app::storage::AppStorage;

fn make_record(id: &str, paper_id: &str, output: &str) -> AiRecord {
    AiRecord {
        id: id.to_string(),
        paper_id: PaperId(paper_id.to_string()),
        block_id: BlockId("block-1".to_string()),
        record_type: "translate".to_string(),
        input: "hello".to_string(),
        output: output.to_string(),
        language: "zh".to_string(),
        created_at_ms: 1000,
    }
}

#[test]
fn save_ai_record_upsert_replaces_existing() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("paper-1".to_string());

    let rec1 = make_record("ai-1", "paper-1", "first translation");
    store.save_ai_record(&rec1).unwrap();

    let records = store.load_ai_records(&pid).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].output, "first translation");

    // Save again with same ID but different output -> should replace, not append
    let rec1_updated = make_record("ai-1", "paper-1", "updated translation");
    store.save_ai_record(&rec1_updated).unwrap();

    let records = store.load_ai_records(&pid).unwrap();
    assert_eq!(records.len(), 1, "upsert should not create duplicate");
    assert_eq!(records[0].output, "updated translation");
}

#[test]
fn save_ai_record_different_ids_both_persisted() {
    let mut store = InMemoryStore::new();
    let pid = PaperId("paper-1".to_string());

    store
        .save_ai_record(&make_record("ai-1", "paper-1", "first"))
        .unwrap();
    store
        .save_ai_record(&make_record("ai-2", "paper-1", "second"))
        .unwrap();

    let records = store.load_ai_records(&pid).unwrap();
    assert_eq!(records.len(), 2);
}
