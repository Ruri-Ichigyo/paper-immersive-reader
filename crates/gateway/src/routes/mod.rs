pub mod explain;
pub mod translate;

use axum::Router;

use crate::services::ai_client::AiClient;

pub fn build_router<A: AiClient + 'static>(ai_client: A) -> Router {
    let shared = std::sync::Arc::new(ai_client);

    Router::new()
        .merge(translate::routes(shared.clone()))
        .merge(explain::routes(shared))
}
