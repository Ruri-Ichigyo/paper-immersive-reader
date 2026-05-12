use std::collections::HashMap;

use super::{AiContext, AiError, AiRequestType, AiResult};
use crate::storage::schema::AiRecord;
use crate::storage::AppStorage;

/// Client that handles AI requests and persists results to storage.
/// In a real browser context, this would call the gateway via HTTP.
/// For native testing, it uses an in-process stub.
pub struct WebAiClient<S: AppStorage> {
    store: S,
    #[allow(dead_code)]
    gateway_url: String,
    is_online: bool,
    // In-memory cache of responses for testing
    canned_responses: HashMap<String, AiResult>,
}

impl<S: AppStorage> WebAiClient<S> {
    pub fn new(store: S, gateway_url: String) -> Self {
        Self {
            store,
            gateway_url,
            is_online: true,
            canned_responses: HashMap::new(),
        }
    }

    pub fn new_offline(store: S) -> Self {
        Self {
            store,
            gateway_url: String::new(),
            is_online: false,
            canned_responses: HashMap::new(),
        }
    }

    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
    }

    pub fn is_online(&self) -> bool {
        self.is_online
    }

    #[allow(dead_code)]
    pub fn with_canned_response(mut self, key: &str, result: AiResult) -> Self {
        self.canned_responses.insert(key.to_string(), result);
        self
    }

    pub fn request(
        &mut self,
        req_type: AiRequestType,
        ctx: &AiContext,
    ) -> Result<AiResult, AiError> {
        if !self.is_online {
            return Err(AiError::Offline);
        }

        let (record_type, language, key) = match &req_type {
            AiRequestType::Translate {
                target_language, ..
            } => ("translate", target_language.clone(), "translate"),
            AiRequestType::ExplainTerm { language } => {
                ("explain_term", language.clone(), "explain_term")
            }
            AiRequestType::ExplainSentence { language } => {
                ("explain_sentence", language.clone(), "explain_sentence")
            }
        };

        // Use canned response for testing, or generate a stub response
        let result = if let Some(canned) = self.canned_responses.get(key) {
            canned.clone()
        } else {
            let output = match &req_type {
                AiRequestType::Translate {
                    target_language, ..
                } => format!("[{}] {}", target_language, ctx.selected_text),
                AiRequestType::ExplainTerm { language } => {
                    format!("[{} term explanation] {}", language, ctx.selected_text)
                }
                AiRequestType::ExplainSentence { language } => {
                    format!("[{} sentence explanation] {}", language, ctx.selected_text)
                }
            };

            let normalized_term = match &req_type {
                AiRequestType::Translate { .. } | AiRequestType::ExplainTerm { .. } => {
                    if ctx.selected_text.split_whitespace().count() == 1 {
                        Some(ctx.selected_text.to_lowercase())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            AiResult {
                record_type: record_type.to_string(),
                input: ctx.selected_text.clone(),
                output,
                language,
                normalized_term,
            }
        };

        Ok(result)
    }

    pub fn persist_result(
        &mut self,
        ctx: &AiContext,
        result: &AiResult,
    ) -> Result<(), crate::storage::StorageError> {
        let record = AiRecord {
            id: format!(
                "ai-{}",
                uuid::Uuid::new_v4()
                    .to_string()
                    .split('-')
                    .next()
                    .unwrap_or("0")
            ),
            paper_id: ctx.paper_id.clone(),
            block_id: ctx.block_id.clone(),
            record_type: result.record_type.clone(),
            input: result.input.clone(),
            output: result.output.clone(),
            language: result.language.clone(),
            created_at_ms: current_time_ms(),
        };
        self.store.save_ai_record(&record)?;
        Ok(())
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }
}

fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// --- Gateway HTTP client (browser / WASM) ---

/// Real AI client that calls the gateway HTTP API via gloo-net.
/// Only available on wasm32 target. For native testing, use `WebAiClient`.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct GatewayAiClient {
    gateway_url: String,
    is_online: bool,
}

#[cfg(target_arch = "wasm32")]
impl GatewayAiClient {
    pub fn new(gateway_url: String) -> Self {
        Self {
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            is_online: true,
        }
    }

    pub fn new_offline() -> Self {
        Self {
            gateway_url: String::new(),
            is_online: false,
        }
    }

    pub fn is_online(&self) -> bool {
        self.is_online
    }

    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
    }

    /// Sync check stub (WASM health check needs async).
    pub fn check_online(&mut self) -> bool {
        true
    }

    /// Sync stub that guides users to the async path.
    pub fn translate(
        &self,
        _text: &str,
        _target: &str,
        _ctx: Option<&str>,
    ) -> Result<String, String> {
        Err("GatewayAiClient: use translate_async in WASM".into())
    }

    pub fn explain(
        &self,
        _text: &str,
        _type: &str,
        _lang: &str,
        _ctx: Option<&str>,
    ) -> Result<super::AiResult, String> {
        Err("GatewayAiClient: use explain_async in WASM".into())
    }

    /// Async translate — calls POST /api/translate on the gateway.
    pub async fn translate_async(
        &self,
        text: &str,
        target_language: &str,
        context: Option<&str>,
    ) -> Result<String, String> {
        let body = serde_json::json!({
            "text": text,
            "target_language": target_language,
            "context": context,
        });
        let mut opts = gloo_net::http::RequestOptions::default();
        opts.method = gloo_net::http::Method::POST;
        opts.headers = std::collections::HashMap::from([(
            "Content-Type".to_string(),
            "application/json".to_string(),
        )]);
        opts.body = Some(serde_json::to_string(&body).unwrap_or_default().into());

        let resp =
            gloo_net::http::Request::new(&format!("{}/api/translate", self.gateway_url), opts)
                .send()
                .await
                .map_err(|e| e.to_string())?;

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        json["translation"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "no translation in response".to_string())
    }

    /// Async explain — calls POST /api/explain on the gateway.
    pub async fn explain_async(
        &self,
        text: &str,
        explanation_type: &str,
        language: &str,
        context: Option<&str>,
    ) -> Result<super::AiResult, String> {
        let body = serde_json::json!({
            "text": text,
            "explanation_type": explanation_type,
            "language": language,
            "context": context,
        });
        let mut opts = gloo_net::http::RequestOptions::default();
        opts.method = gloo_net::http::Method::POST;
        opts.headers = std::collections::HashMap::from([(
            "Content-Type".to_string(),
            "application/json".to_string(),
        )]);
        opts.body = Some(serde_json::to_string(&body).unwrap_or_default().into());

        let resp = gloo_net::http::Request::new(&format!("{}/api/explain", self.gateway_url), opts)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(super::AiResult {
            record_type: format!("explain_{}", explanation_type),
            input: text.to_string(),
            output: json["explanation"]
                .as_str()
                .unwrap_or("[no explanation]")
                .to_string(),
            language: language.to_string(),
            normalized_term: None,
        })
    }
}
