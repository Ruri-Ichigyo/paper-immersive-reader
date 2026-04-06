use async_trait::async_trait;

use crate::config::GatewayConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslateCommand {
    pub selected_text: String,
    pub selection_type: String,
    pub source_language: String,
    pub target_language: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslateResult {
    pub translation: String,
    pub normalized_term: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainCommand {
    pub selected_text: String,
    pub explanation_type: String,
    pub explanation_language: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainResult {
    pub explanation: String,
    pub background: Option<String>,
    pub caution: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiClientError {
    NotConfigured,
    UpstreamUnavailable,
}

impl std::fmt::Display for AiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => {
                write!(
                    f,
                    "gateway AI client is not configured with an upstream API key"
                )
            }
            Self::UpstreamUnavailable => write!(f, "gateway AI upstream is unavailable"),
        }
    }
}

impl std::error::Error for AiClientError {}

#[async_trait]
pub trait AiClient: Send + Sync + 'static {
    async fn translate(&self, request: TranslateCommand) -> Result<TranslateResult, AiClientError>;

    async fn explain(&self, request: ExplainCommand) -> Result<ExplainResult, AiClientError>;
}

#[derive(Debug, Clone)]
pub struct ConfiguredAiClient {
    upstream_ai_base_url: Option<String>,
    upstream_ai_api_key: Option<String>,
}

impl ConfiguredAiClient {
    pub fn from_config(config: &GatewayConfig) -> Self {
        Self {
            upstream_ai_base_url: config.upstream_ai_base_url.clone(),
            upstream_ai_api_key: config.upstream_ai_api_key.clone(),
        }
    }
}

#[async_trait]
impl AiClient for ConfiguredAiClient {
    async fn translate(&self, request: TranslateCommand) -> Result<TranslateResult, AiClientError> {
        if self.upstream_ai_api_key.is_none() || self.upstream_ai_base_url.is_none() {
            return Err(AiClientError::NotConfigured);
        }

        Ok(TranslateResult {
            translation: format!("stub translation: {}", request.selected_text),
            normalized_term: match request.selection_type.as_str() {
                "term" => Some(request.selected_text.trim().to_lowercase()),
                _ => None,
            },
        })
    }

    async fn explain(&self, request: ExplainCommand) -> Result<ExplainResult, AiClientError> {
        if self.upstream_ai_api_key.is_none() || self.upstream_ai_base_url.is_none() {
            return Err(AiClientError::NotConfigured);
        }

        Ok(ExplainResult {
            explanation: format!(
                "stub explanation ({}): {}",
                request.explanation_language, request.selected_text
            ),
            background: request.context,
            caution: Some("Stub client in use until upstream AI wiring lands.".to_string()),
        })
    }
}
