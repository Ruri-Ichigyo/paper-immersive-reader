use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::services::ai_client::AiClient;

#[derive(Debug, Deserialize)]
pub struct ExplainRequest {
    pub text: String,
    pub explanation_type: String, // "term" or "sentence"
    pub language: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExplainResponse {
    pub explanation: String,
    pub background: Option<String>,
    pub caution: Option<String>,
}

pub fn routes<A: AiClient + 'static>(client: Arc<A>) -> Router {
    Router::new()
        .route("/api/explain", post(explain_handler))
        .with_state(client)
}

async fn explain_handler<A: AiClient>(
    State(client): State<Arc<A>>,
    Json(req): Json<ExplainRequest>,
) -> Json<ExplainResponse> {
    let result = client
        .explain(
            &req.text,
            &req.explanation_type,
            &req.language,
            req.context.as_deref(),
        )
        .await;

    Json(ExplainResponse {
        explanation: result.explanation,
        background: result.background,
        caution: result.caution,
    })
}
