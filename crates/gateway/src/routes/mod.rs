pub mod explain;
pub mod translate;

use std::sync::Arc;

use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

use crate::services::ai_client::{AiClient, AiClientError};

#[derive(Clone)]
pub struct GatewayAppState {
    pub ai_client: Arc<dyn AiClient>,
}

pub fn build_router_with_client<C>(client: C) -> Router
where
    C: AiClient,
{
    Router::new()
        .route("/api/translate", post(translate::translate))
        .route("/api/explain", post(explain::explain))
        .with_state(GatewayAppState {
            ai_client: Arc::new(client),
        })
}

#[derive(Debug)]
pub struct GatewayHttpError {
    status: axum::http::StatusCode,
    message: String,
}

impl From<AiClientError> for GatewayHttpError {
    fn from(value: AiClientError) -> Self {
        let status = match value {
            AiClientError::NotConfigured => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            AiClientError::UpstreamUnavailable => axum::http::StatusCode::BAD_GATEWAY,
        };

        Self {
            status,
            message: value.to_string(),
        }
    }
}

impl IntoResponse for GatewayHttpError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.message
            })),
        )
            .into_response()
    }
}
