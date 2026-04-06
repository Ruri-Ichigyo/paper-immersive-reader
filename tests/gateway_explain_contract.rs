use http_body_util::BodyExt;
use tower::ServiceExt;

struct StubAiClient;

#[async_trait::async_trait]
impl paper_immersive_reader_gateway::services::ai_client::AiClient for StubAiClient {
    async fn translate(
        &self,
        _request: paper_immersive_reader_gateway::services::ai_client::TranslateCommand,
    ) -> Result<
        paper_immersive_reader_gateway::services::ai_client::TranslateResult,
        paper_immersive_reader_gateway::services::ai_client::AiClientError,
    > {
        unreachable!("explain contract test should not call translate")
    }

    async fn explain(
        &self,
        _request: paper_immersive_reader_gateway::services::ai_client::ExplainCommand,
    ) -> Result<
        paper_immersive_reader_gateway::services::ai_client::ExplainResult,
        paper_immersive_reader_gateway::services::ai_client::AiClientError,
    > {
        Ok(
            paper_immersive_reader_gateway::services::ai_client::ExplainResult {
                explanation: "Entropy is the amount of uncertainty.".to_string(),
                background: Some(
                    "Often discussed in thermodynamics and information theory.".to_string(),
                ),
                caution: Some("Interpretation depends on domain context.".to_string()),
            },
        )
    }
}

#[tokio::test]
async fn gateway_explain_route_returns_structured_json_contract() {
    let app = paper_immersive_reader_gateway::routes::build_router_with_client(StubAiClient);

    let response = app
        .oneshot(
            axum::http::Request::post("/api/explain")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({
                        "selected_text": "Entropy",
                        "explanation_type": "term",
                        "explanation_language": "en",
                        "context": "Entropy measures uncertainty."
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("body should be json");

    assert_eq!(json["explanation"], "Entropy is the amount of uncertainty.");
    assert_eq!(
        json["background"],
        "Often discussed in thermodynamics and information theory."
    );
    assert_eq!(json["caution"], "Interpretation depends on domain context.");
}
