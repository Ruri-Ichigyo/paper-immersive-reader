use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub bind_address: String,
    pub ai_api_key: Option<String>,
    pub ai_base_url: String,
    pub ai_model: String,
    pub gateway_token: Option<String>,
    pub use_stub: bool,
}

impl GatewayConfig {
    pub fn from_env() -> Self {
        let use_stub = env::var("GATEWAY_AI_USE_STUB")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        Self {
            bind_address: env::var("GATEWAY_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
            ai_api_key: env::var("GATEWAY_AI_API_KEY")
                .or_else(|_| env::var("AI_API_KEY"))
                .ok(),
            ai_base_url: env::var("GATEWAY_AI_BASE_URL")
                .or_else(|_| env::var("AI_BASE_URL"))
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            ai_model: env::var("GATEWAY_AI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            gateway_token: env::var("GATEWAY_TOKEN").ok(),
            use_stub,
        }
    }

    pub fn require_ai_key(&self) -> Result<&str, String> {
        if self.use_stub {
            return Err("stub mode".to_string());
        }
        self.ai_api_key
            .as_deref()
            .or(self.gateway_token.as_deref())
            .ok_or_else(|| {
                "GATEWAY_AI_API_KEY or GATEWAY_TOKEN is required (or set GATEWAY_AI_USE_STUB=true for dev)".to_string()
            })
    }
}
