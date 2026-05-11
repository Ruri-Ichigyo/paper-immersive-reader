use paper_immersive_reader_gateway::routes;
use paper_immersive_reader_gateway::services::ai_client::StubAiClient;
use reqwest::StatusCode;

#[tokio::test]
async fn explain_term_endpoint_returns_structured_response() {
    let app = routes::build_router(StubAiClient);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{}/api/explain", addr))
        .json(&serde_json::json!({
            "text": "ontology",
            "explanation_type": "term",
            "language": "en",
            "context": "Heidegger's concept of ontology differs from..."
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["explanation"].is_string(),
        "explanation must be present"
    );
    assert!(
        body["background"].is_string() || body["background"].is_null(),
        "background is optional"
    );
    assert!(
        body["caution"].is_null() || body["caution"].is_string(),
        "caution is optional"
    );
}

#[tokio::test]
async fn explain_sentence_endpoint_includes_caution() {
    let app = routes::build_router(StubAiClient);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{}/api/explain", addr))
        .json(&serde_json::json!({
            "text": "Being is that which is at issue for Dasein.",
            "explanation_type": "sentence",
            "language": "en"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["explanation"].is_string());
    assert!(
        body["caution"].is_string(),
        "sentence explanation should have caution"
    );
}
