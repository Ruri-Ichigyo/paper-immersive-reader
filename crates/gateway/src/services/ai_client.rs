use std::future::Future;
use std::pin::Pin;

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
        let result = format!("[{}] {}", target_language, text);
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
            explanation: format!("[{} explanation of '{}' in {}]", explanation_type, text, language),
            background: Some("Additional context would appear here.".to_string()),
            caution: if explanation_type == "sentence" {
                Some("This interpretation may vary by discipline.".to_string())
            } else {
                None
            },
        };
        Box::pin(async move { result })
    }
}
