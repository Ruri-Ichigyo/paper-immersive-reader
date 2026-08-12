use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ExplainResult {
    pub explanation: String,
    pub background: Option<String>,
    pub caution: Option<String>,
}

pub trait AiClient: Send + Sync {
    fn translate(
        &self,
        text: &str,
        target_language: &str,
        context: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = String> + Send + '_>>;

    fn explain(
        &self,
        text: &str,
        explanation_type: &str,
        language: &str,
        context: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = ExplainResult> + Send + '_>>;
}

// --- OpenAI-compatible real client ---

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Debug, Clone)]
pub struct OpenAiClient {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    async fn chat(&self, system: &str, user: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user.to_string(),
                },
            ],
            temperature: Some(0.3),
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI API request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("AI API error {}: {}", status, text));
        }

        let data: ChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse AI response: {}", e))?;

        data.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "AI returned empty response".to_string())
    }
}

impl AiClient for OpenAiClient {
    fn translate(
        &self,
        text: &str,
        target_language: &str,
        context: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = String> + Send + '_>> {
        let text = text.to_string();
        let target_language = target_language.to_string();
        let context = context.map(|s| s.to_string());
        Box::pin(async move {
            let system = format!(
                "You are a translation assistant. Translate the given text to {}. \
                 Return only the translation, no explanations, no additional text.",
                target_language
            );
            let user = match &context {
                Some(ctx) => format!(
                    "Context: {}\n\nTranslate to {}: {}",
                    ctx, target_language, text
                ),
                None => format!("Translate to {}: {}", target_language, text),
            };
            match self.chat(&system, &user).await {
                Ok(t) => t.trim().to_string(),
                Err(e) => {
                    tracing::warn!("Translate API error, falling back: {}", e);
                    format!("[translation unavailable: {}]", e)
                }
            }
        })
    }

    fn explain(
        &self,
        text: &str,
        explanation_type: &str,
        language: &str,
        context: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = ExplainResult> + Send + '_>> {
        let text = text.to_string();
        let explanation_type = explanation_type.to_string();
        let language = language.to_string();
        let context = context.map(|s| s.to_string());
        Box::pin(async move {
            let system = if explanation_type == "term" {
                format!(
                    "You are an academic tutor. Explain the given term in {}. \
                     Return ONLY a JSON object with fields: \
                     \"explanation\" (concise definition), \
                     \"background\" (optional, brief disciplinary context), \
                     \"caution\" (optional, any nuance or common misunderstanding). \
                     Do not include any text outside the JSON object.",
                    language
                )
            } else {
                format!(
                    "You are an academic reading assistant. Explain the given sentence in {}.\
                     Return ONLY a JSON object with fields: \
                     \"explanation\" (clear explanation of the sentence meaning), \
                     \"background\" (optional, wider context or implications), \
                     \"caution\" (optional, any interpretative uncertainty). \
                     Do not include any text outside the JSON object.",
                    language
                )
            };

            let user = match &context {
                Some(ctx) => format!(
                    "Context: {}\n\nExplain this {}: {}",
                    ctx, explanation_type, text
                ),
                None => format!("Explain this {}: {}", explanation_type, text),
            };

            match self.chat(&system, &user).await {
                Ok(content) => parse_explain_json(&content),
                Err(e) => {
                    tracing::warn!("Explain API error, falling back: {}", e);
                    ExplainResult {
                        explanation: format!("[explanation unavailable: {}]", e),
                        background: None,
                        caution: None,
                    }
                }
            }
        })
    }
}

#[derive(Debug, Deserialize)]
struct ExplainJson {
    explanation: String,
    background: Option<String>,
    caution: Option<String>,
}

fn parse_explain_json(raw: &str) -> ExplainResult {
    let trimmed = raw.trim();
    // Strip markdown code fences if present
    let json_str = if trimmed.starts_with("```") {
        trimmed
            .strip_prefix("```json")
            .or_else(|| trimmed.strip_prefix("```"))
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim())
            .unwrap_or(trimmed)
    } else {
        trimmed
    };

    match serde_json::from_str::<ExplainJson>(json_str) {
        Ok(parsed) => ExplainResult {
            explanation: parsed.explanation,
            background: parsed.background,
            caution: parsed.caution,
        },
        Err(_) => {
            // If JSON parsing fails, treat whole response as explanation
            ExplainResult {
                explanation: trimmed.to_string(),
                background: None,
                caution: Some("Response format was unexpected; raw output shown.".to_string()),
            }
        }
    }
}

// --- Stub client for testing / local development ---

/// Stub AI client for testing and local development.
/// Returns canned responses without calling an external AI service.
pub struct StubAiClient;

impl AiClient for StubAiClient {
    fn translate(
        &self,
        text: &str,
        target_language: &str,
        _context: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = String> + Send + '_>> {
        let result = format!(
            "⚠️ [Stub 模式] 未配置 AI API Key。\n\
             原文 ({target_language}): {text}\n\n\
             请在 Settings → Gateway 中设置 GATEWAY_AI_API_KEY 以启用真实翻译，\
             或设置 GATEWAY_AI_USE_STUB=false 并配置 API Key。"
        );
        Box::pin(async move { result })
    }

    fn explain(
        &self,
        text: &str,
        explanation_type: &str,
        language: &str,
        _context: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = ExplainResult> + Send + '_>> {
        let result = ExplainResult {
            explanation: format!(
                "⚠️ Stub 模式 — 未配置真实 AI 服务。\n\
                 此处显示的是 `{text}` 的 {explanation_type} 解释占位符（{language}）。\n\n\
                 要获得真实的 AI 解释，请在 .env 中设置 GATEWAY_AI_API_KEY 并重启 Gateway。"
            ),
            background: Some("当前运行在 Stub 模式下，所有 AI 输出均为占位符。".to_string()),
            caution: if explanation_type == "sentence" {
                Some("Stub 模式无法提供真实的语义解释。".to_string())
            } else {
                None
            },
        };
        Box::pin(async move { result })
    }
}
