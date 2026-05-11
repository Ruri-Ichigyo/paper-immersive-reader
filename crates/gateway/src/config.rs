use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub bind_address: String,
    pub ai_api_key: Option<String>,
    pub ai_base_url: Option<String>,
    pub gateway_token: Option<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        Self {
            bind_address: env::var("GATEWAY_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
            ai_api_key: env::var("AI_API_KEY").ok(),
            ai_base_url: env::var("AI_BASE_URL").ok(),
            gateway_token: env::var("GATEWAY_TOKEN").ok(),
        }
    }
}
