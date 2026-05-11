use paper_immersive_reader_core::domain::{BlockId, PaperId};
use paper_immersive_reader_web_app::ai::client::WebAiClient;
use paper_immersive_reader_web_app::ai::{AiContext, AiError, AiRequestType};
use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;

#[test]
fn offline_client_returns_error_for_ai_requests() {
    let store = InMemoryStore::new();
    let mut client = WebAiClient::new_offline(store);

    assert!(!client.is_online());

    let ctx = AiContext {
        paper_id: PaperId("p-off".to_string()),
        block_id: BlockId("b-off".to_string()),
        selected_text: "test".to_string(),
        block_text: "test block text".to_string(),
    };

    // Translate should fail offline
    let result = client.request(
        AiRequestType::Translate {
            source_language: "en".to_string(),
            target_language: "zh".to_string(),
        },
        &ctx,
    );
    assert!(matches!(result, Err(AiError::Offline)));

    // Explain should also fail offline
    let result = client.request(
        AiRequestType::ExplainTerm {
            language: "en".to_string(),
        },
        &ctx,
    );
    assert!(matches!(result, Err(AiError::Offline)));
}

#[test]
fn coming_back_online_restores_ai_availability() {
    let store = InMemoryStore::new();
    let mut client = WebAiClient::new_offline(store);

    // Go online
    client.set_online(true);
    assert!(client.is_online());

    let ctx = AiContext {
        paper_id: PaperId("p-on".to_string()),
        block_id: BlockId("b-on".to_string()),
        selected_text: "phenomenology".to_string(),
        block_text: "Phenomenology is the study of...".to_string(),
    };

    let result = client.request(
        AiRequestType::Translate {
            source_language: "en".to_string(),
            target_language: "zh".to_string(),
        },
        &ctx,
    );
    assert!(result.is_ok());
}
