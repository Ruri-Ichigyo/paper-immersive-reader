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
