use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::routes::{GatewayAppState, GatewayHttpError};
use crate::services::ai_client::TranslateCommand;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub selected_text: String,
    pub selection_type: String,
    pub source_language: String,
    pub target_language: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub translation: String,
    pub normalized_term: Option<String>,
}

pub async fn translate(
    State(state): State<GatewayAppState>,
    Json(request): Json<TranslateRequest>,
) -> Result<Json<TranslateResponse>, GatewayHttpError> {
    let result = state
        .ai_client
        .translate(TranslateCommand {
            selected_text: request.selected_text,
            selection_type: request.selection_type,
            source_language: request.source_language,
            target_language: request.target_language,
            context: request.context,
        })
        .await?;

    Ok(Json(TranslateResponse {
        translation: result.translation,
        normalized_term: result.normalized_term,
    }))
}
