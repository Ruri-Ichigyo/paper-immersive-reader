use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::services::ai_client::AiClient;

#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub source_language: Option<String>,
    pub target_language: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TranslateResponse {
    pub translation: String,
    pub source_text: String,
    pub target_language: String,
    pub normalized_term: Option<String>,
}

pub fn routes<A: AiClient + 'static>(client: Arc<A>) -> Router {
    Router::new()
        .route("/api/translate", post(translate_handler))
        .with_state(client)
}

async fn translate_handler<A: AiClient>(
    State(client): State<Arc<A>>,
    Json(req): Json<TranslateRequest>,
) -> Json<TranslateResponse> {
    let translation = client
        .translate(&req.text, &req.target_language, req.context.as_deref())
        .await;

    let normalized_term = extract_single_word(&req.text).or_else(|| {
        if translation.split_whitespace().count() == 1 {
            Some(translation.clone())
        } else {
            None
        }
    });

    Json(TranslateResponse {
        translation,
        source_text: req.text,
        target_language: req.target_language,
        normalized_term,
    })
}

fn extract_single_word(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.split_whitespace().count() == 1
        && trimmed.chars().all(|c| c.is_alphabetic() || c == '-')
    {
        Some(trimmed.to_lowercase())
    } else {
        None
    }
}
