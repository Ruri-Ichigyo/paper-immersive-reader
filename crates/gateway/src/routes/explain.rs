use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::routes::{GatewayAppState, GatewayHttpError};
use crate::services::ai_client::ExplainCommand;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainRequest {
    pub selected_text: String,
    pub explanation_type: String,
    pub explanation_language: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainResponse {
    pub explanation: String,
    pub background: Option<String>,
    pub caution: Option<String>,
}

pub async fn explain(
    State(state): State<GatewayAppState>,
    Json(request): Json<ExplainRequest>,
) -> Result<Json<ExplainResponse>, GatewayHttpError> {
    let result = state
        .ai_client
        .explain(ExplainCommand {
            selected_text: request.selected_text,
            explanation_type: request.explanation_type,
            explanation_language: request.explanation_language,
            context: request.context,
        })
        .await?;

    Ok(Json(ExplainResponse {
        explanation: result.explanation,
        background: result.background,
        caution: result.caution,
    }))
}
