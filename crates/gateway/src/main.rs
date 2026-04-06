use paper_immersive_reader_gateway::config::GatewayConfig;
use paper_immersive_reader_gateway::logging::init_gateway_logging;
use paper_immersive_reader_gateway::routes::build_router_with_client;
use paper_immersive_reader_gateway::services::ai_client::ConfiguredAiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GatewayConfig::from_env();
    let (_guard, log_path) = init_gateway_logging()?;

    tracing::info!(
        log_path = %log_path.display(),
        bind_address = %config.bind_address,
        "starting gateway"
    );

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    let app = build_router_with_client(ConfiguredAiClient::from_config(&config));

    axum::serve(listener, app).await?;
    Ok(())
}
