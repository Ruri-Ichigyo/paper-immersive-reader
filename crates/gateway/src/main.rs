use paper_immersive_reader_gateway::config::GatewayConfig;
use paper_immersive_reader_gateway::logging;
use paper_immersive_reader_gateway::routes::{self, GatewayState};
use paper_immersive_reader_gateway::services::ai_client::{OpenAiClient, StubAiClient};

#[tokio::main]
async fn main() {
    let _guard = logging::init_logging();
    let config = GatewayConfig::from_env();

    tracing::info!("Starting gateway on {}", config.bind_address);

    let gateway_state = GatewayState {
        expected_token: config.gateway_token.clone(),
    };

    if config.use_stub {
        tracing::info!("Using StubAiClient (GATEWAY_AI_USE_STUB=true)");
        let app = routes::build_router(StubAiClient, gateway_state);
        let listener = tokio::net::TcpListener::bind(&config.bind_address)
            .await
            .expect("failed to bind");
        tracing::info!("Gateway listening on {}", config.bind_address);
        axum::serve(listener, app).await.expect("server error");
    } else {
        let api_key = config.require_ai_key().unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });
        tracing::info!(
            "Using OpenAiClient (model: {}, base_url: {})",
            config.ai_model,
            config.ai_base_url
        );
        let ai_client = OpenAiClient::new(
            config.ai_base_url.clone(),
            api_key.to_string(),
            config.ai_model.clone(),
        );
        let app = routes::build_router(ai_client, gateway_state);
        let listener = tokio::net::TcpListener::bind(&config.bind_address)
            .await
            .expect("failed to bind");
        tracing::info!("Gateway listening on {}", config.bind_address);
        axum::serve(listener, app).await.expect("server error");
    }
}
