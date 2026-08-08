pub mod explain;
pub mod translate;

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::{from_fn_with_state, Next},
    response::Response,
    Router,
};

use crate::services::ai_client::AiClient;

/// Shared application state for the gateway.
#[derive(Clone)]
pub struct GatewayState {
    /// If set, every `/api/*` request must carry `Authorization: Bearer <token>`.
    pub expected_token: Option<String>,
}

pub fn build_router<A: AiClient + 'static>(ai_client: A, gateway_state: GatewayState) -> Router {
    let shared = std::sync::Arc::new(ai_client);

    let api_routes = Router::new()
        .merge(translate::routes(shared.clone()))
        .merge(explain::routes(shared))
        .layer(from_fn_with_state(gateway_state.clone(), require_token));

    Router::new()
        .route("/health", axum::routing::get(health_handler))
        .nest("/api", api_routes)
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}

/// Middleware: if `expected_token` is `Some`, require a matching
/// `Authorization: Bearer <token>` header.  If `None`, pass through.
pub async fn require_token(
    State(state): State<GatewayState>,
    req: Request,
    next: Next,
) -> Response {
    match &state.expected_token {
        None => next.run(req).await,
        Some(token) => {
            let provided = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "));

            match provided {
                Some(t) if t == token => next.run(req).await,
                _ => Response::builder()
                    .status(axum::http::StatusCode::UNAUTHORIZED)
                    .body(axum::body::Body::from("unauthorized"))
                    .unwrap(),
            }
        }
    }
}
