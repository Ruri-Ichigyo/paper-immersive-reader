#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    pub bind_address: String,
    pub gateway_token: Option<String>,
    pub upstream_ai_base_url: Option<String>,
    pub upstream_ai_api_key: Option<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        Self {
            bind_address: std::env::var("PAPER_IR_GATEWAY_BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:4000".to_string()),
            gateway_token: std::env::var("PAPER_IR_GATEWAY_TOKEN").ok(),
            upstream_ai_base_url: std::env::var("PAPER_IR_UPSTREAM_AI_BASE_URL").ok(),
            upstream_ai_api_key: std::env::var("PAPER_IR_UPSTREAM_AI_API_KEY").ok(),
        }
    }
}
