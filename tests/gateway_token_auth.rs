use paper_immersive_reader_gateway::routes::{self, GatewayState};
use paper_immersive_reader_gateway::services::ai_client::StubAiClient;
use reqwest::StatusCode;

fn no_token_state() -> GatewayState {
    GatewayState {
        expected_token: None,
    }
}

fn token_state(token: &str) -> GatewayState {
    GatewayState {
        expected_token: Some(token.to_string()),
    }
}

async fn spawn_server(state: GatewayState) -> String {
    let app = routes::build_router(StubAiClient, state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{}", addr)
}

// --- Health endpoint tests (Task 8) ---

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let base = spawn_server(no_token_state()).await;

    let client = reqwest::Client::new();
    let resp = client.get(format!("{}/health", base)).send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

// --- Token middleware tests (Task 7) ---

#[tokio::test]
async fn no_token_required_when_gateway_token_unset() {
    let base = spawn_server(no_token_state()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/translate", base))
        .json(&serde_json::json!({
            "text": "hello",
            "target_language": "zh"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn valid_token_passes_auth() {
    let base = spawn_server(token_state("secret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/translate", base))
        .header("Authorization", "Bearer secret")
        .json(&serde_json::json!({
            "text": "hello",
            "target_language": "zh"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn missing_token_returns_401() {
    let base = spawn_server(token_state("secret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/translate", base))
        .json(&serde_json::json!({
            "text": "hello",
            "target_language": "zh"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_token_returns_401() {
    let base = spawn_server(token_state("secret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/translate", base))
        .header("Authorization", "Bearer wrong")
        .json(&serde_json::json!({
            "text": "hello",
            "target_language": "zh"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn token_required_for_explain_endpoint() {
    let base = spawn_server(token_state("secret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/explain", base))
        .json(&serde_json::json!({
            "text": "epistemology",
            "explanation_type": "term",
            "language": "en"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_endpoint_does_not_require_token() {
    // /health is outside the /api/* group, so token middleware does not apply.
    let base = spawn_server(token_state("secret")).await;

    let client = reqwest::Client::new();
    let resp = client.get(format!("{}/health", base)).send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}
