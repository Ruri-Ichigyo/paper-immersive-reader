use paper_immersive_reader_core::domain::{BlockId, PaperId};
use paper_immersive_reader_web_app::ai::client::WebAiClient;
use paper_immersive_reader_web_app::ai::{AiContext, AiRequestType};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
use paper_immersive_reader_web_app::storage::AppStorage;

#[test]
fn translate_selection_persists_ai_record() {
    let store = InMemoryStore::new();
    let mut client = WebAiClient::new(store, "http://localhost:3000".to_string());

    let ctx = AiContext {
        paper_id: PaperId("p-tl".to_string()),
        block_id: BlockId("b-tl".to_string()),
        selected_text: "epistemology".to_string(),
        block_text: "The study of epistemology concerns...".to_string(),
    };

    let result = client
        .request(
            AiRequestType::Translate {
                source_language: "en".to_string(),
                target_language: "zh".to_string(),
            },
            &ctx,
        )
        .unwrap();

    assert_eq!(result.record_type, "translate");
    assert_eq!(result.input, "epistemology");
    assert!(!result.output.is_empty());
    assert_eq!(
        result.normalized_term.as_deref(),
        Some("epistemology"),
        "single word should produce normalized term"
    );

    // Persist and verify
    client.persist_result(&ctx, &result).unwrap();

    let records = client.store_mut().load_ai_records(&ctx.paper_id).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].record_type, "translate");
    assert_eq!(records[0].input, "epistemology");
}
