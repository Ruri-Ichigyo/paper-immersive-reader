#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteImportRequest {
    pub text: String,
    pub title: Option<String>,
    pub source_ref: Option<String>,
    pub imported_at_ms: u64,
}
