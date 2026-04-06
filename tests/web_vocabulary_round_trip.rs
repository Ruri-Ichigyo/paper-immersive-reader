use paper_immersive_reader_core::domain::{BlockId, PaperId, VocabularyId, VocabularyItem};

#[test]
fn web_storage_merges_duplicate_vocabulary_lookups() {
    let storage = paper_immersive_reader_web_app::storage::WebStorage::in_memory();
    let paper_id = PaperId("paper-web-1".to_string());
    let first = VocabularyItem {
        id: VocabularyId("vocab-1".to_string()),
        term: "entropy".to_string(),
        paper_id_first_seen: paper_id.clone(),
        block_id_first_seen: Some(BlockId("block-web-1".to_string())),
        lookup_count: 1,
        first_seen_at_ms: 1_712_431_200_000,
        last_seen_at_ms: 1_712_431_200_000,
        note: None,
        learned: false,
    };

    storage
        .upsert_vocabulary_lookup(first.clone())
        .expect("first lookup should succeed");

    let merged = storage
        .upsert_vocabulary_lookup(VocabularyItem {
            id: VocabularyId("vocab-2".to_string()),
            term: "Entropy".to_string(),
            paper_id_first_seen: paper_id.clone(),
            block_id_first_seen: Some(BlockId("block-web-2".to_string())),
            lookup_count: 1,
            first_seen_at_ms: 1_712_431_200_100,
            last_seen_at_ms: 1_712_431_260_000,
            note: Some("second lookup".to_string()),
            learned: false,
        })
        .expect("duplicate lookup should merge");

    let reloaded = storage
        .list_vocabulary_for_paper(&paper_id)
        .expect("list should succeed");

    assert_eq!(reloaded.len(), 1);
    assert_eq!(merged.id, first.id);
    assert_eq!(reloaded[0].lookup_count, 2);
    assert_eq!(reloaded[0].first_seen_at_ms, first.first_seen_at_ms);
    assert_eq!(reloaded[0].last_seen_at_ms, 1_712_431_260_000);
    assert_eq!(reloaded[0].term, "entropy");
}
