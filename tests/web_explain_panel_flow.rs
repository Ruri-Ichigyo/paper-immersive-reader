use paper_immersive_reader_core::domain::{BlockId, PaperId};
use paper_immersive_reader_web_app::ai::client::WebAiClient;
use paper_immersive_reader_web_app::ai::{AiContext, AiRequestType};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;

#[test]
fn explain_term_returns_structured_result() {
    let store = InMemoryStore::new();
    let mut client = WebAiClient::new(store, "http://localhost:3000".to_string());

    let ctx = AiContext {
        paper_id: PaperId("p-explain".to_string()),
        block_id: BlockId("b-explain".to_string()),
        selected_text: "ontology".to_string(),
        block_text: "Heidegger's ontology...".to_string(),
    };

    let result = client
        .request(
            AiRequestType::ExplainTerm {
                language: "en".to_string(),
            },
            &ctx,
        )
        .unwrap();

    assert_eq!(result.record_type, "explain_term");
    assert!(!result.output.is_empty());
    assert_eq!(result.language, "en");

    // Verify single-word explain produces normalized term
    assert_eq!(result.normalized_term.as_deref(), Some("ontology"));
}

#[test]
fn explain_sentence_returns_result_in_configured_language() {
    let store = InMemoryStore::new();
    let mut client = WebAiClient::new(store, "http://localhost:3000".to_string());

    let ctx = AiContext {
        paper_id: PaperId("p-explain-s".to_string()),
        block_id: BlockId("b-explain-s".to_string()),
        selected_text: "Dasein is being-in-the-world.".to_string(),
        block_text: "Dasein is being-in-the-world according to Heidegger.".to_string(),
    };

    let result = client
        .request(
            AiRequestType::ExplainSentence {
                language: "zh".to_string(),
            },
            &ctx,
        )
        .unwrap();

    assert_eq!(result.record_type, "explain_sentence");
    assert_eq!(result.language, "zh");
    assert!(!result.output.is_empty());
    // Multi-word input should not produce a normalized term
    assert_eq!(result.normalized_term, None);
}
