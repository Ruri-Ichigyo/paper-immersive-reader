#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfImportRequest {
    pub bytes: Vec<u8>,
    pub title: Option<String>,
    pub source_ref: Option<String>,
    pub imported_at_ms: u64,
}
