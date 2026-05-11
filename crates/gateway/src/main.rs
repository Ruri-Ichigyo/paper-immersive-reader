use paper_immersive_reader_gateway::config::GatewayConfig;
use paper_immersive_reader_gateway::logging;
use paper_immersive_reader_gateway::routes;
use paper_immersive_reader_gateway::services::ai_client::StubAiClient;

#[tokio::main]
async fn main() {
    let _guard = logging::init_logging();
    let config = GatewayConfig::from_env();

    tracing::info!("Starting gateway on {}", config.bind_address);

    let ai_client = StubAiClient;
    let app = routes::build_router(ai_client);

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .expect("failed to bind");

    tracing::info!("Gateway listening on {}", config.bind_address);

    axum::serve(listener, app).await.expect("server error");
}
