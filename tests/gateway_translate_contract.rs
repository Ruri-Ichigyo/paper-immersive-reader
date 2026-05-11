use paper_immersive_reader_gateway::routes;
use paper_immersive_reader_gateway::services::ai_client::StubAiClient;
use reqwest::StatusCode;

#[tokio::test]
async fn translate_endpoint_returns_structured_response() {
    let app = routes::build_router(StubAiClient);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{}/api/translate", addr))
        .json(&serde_json::json!({
            "text": "epistemology",
            "source_language": "en",
            "target_language": "zh",
            "context": "philosophy text"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["translation"].is_string(), "translation must be present");
    assert_eq!(body["source_text"], "epistemology");
    assert_eq!(body["target_language"], "zh");
    assert!(
        body["normalized_term"].is_string(),
        "single-word input should produce normalized term"
    );
}

#[tokio::test]
async fn translate_endpoint_handles_multi_word_input() {
    let app = routes::build_router(StubAiClient);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{}/api/translate", addr))
        .json(&serde_json::json!({
            "text": "the nature of being",
            "target_language": "zh"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(!body["translation"].as_str().unwrap().is_empty());
}
