/// Verify that update_vocabulary on InMemoryStore preserves the learned flag and note.
#[test]
fn update_vocabulary_preserves_learned_and_note() {
    use paper_immersive_reader_core::domain::{BlockId, PaperId, VocabularyId, VocabularyItem};
    use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
    use paper_immersive_reader_web_app::storage::AppStorage;

    let mut store = InMemoryStore::new();

    let item = VocabularyItem {
        id: VocabularyId("vocab-1".to_string()),
        term: "algorithm".to_string(),
        paper_id_first_seen: PaperId("paper-1".to_string()),
        block_id_first_seen: BlockId("block-1".to_string()),
        lookup_count: 1,
        first_seen_at_ms: 1000,
        last_seen_at_ms: 1000,
        note: None,
        learned: false,
    };
    store.upsert_vocabulary(&item).unwrap();

    // Now update with learned=true and a note
    let mut updated = item.clone();
    updated.learned = true;
    updated.note = Some("A step-by-step procedure".to_string());
    store.update_vocabulary(&updated).unwrap();

    let vocab = store.list_vocabulary().unwrap();
    assert_eq!(vocab.len(), 1);
    assert!(vocab[0].learned);
    assert_eq!(vocab[0].note.as_deref(), Some("A step-by-step procedure"));
    // lookup_count should be unchanged (update, not upsert)
    assert_eq!(vocab[0].lookup_count, 1);
}

/// Verify that update_vocabulary overwrites learned back to false.
#[test]
fn update_vocabulary_toggles_learned_off() {
    use paper_immersive_reader_core::domain::{BlockId, PaperId, VocabularyId, VocabularyItem};
    use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
    use paper_immersive_reader_web_app::storage::AppStorage;

    let mut store = InMemoryStore::new();

    let item = VocabularyItem {
        id: VocabularyId("vocab-1".to_string()),
        term: "neural".to_string(),
        paper_id_first_seen: PaperId("paper-1".to_string()),
        block_id_first_seen: BlockId("block-1".to_string()),
        lookup_count: 3,
        first_seen_at_ms: 1000,
        last_seen_at_ms: 2000,
        note: None,
        learned: false,
    };
    store.upsert_vocabulary(&item).unwrap();

    // Mark as learned
    let mut learned = item.clone();
    learned.learned = true;
    store.update_vocabulary(&learned).unwrap();
    assert!(store.list_vocabulary().unwrap()[0].learned);

    // Unmark
    let mut unlearned = item.clone();
    unlearned.learned = false;
    store.update_vocabulary(&unlearned).unwrap();
    assert!(!store.list_vocabulary().unwrap()[0].learned);
}
