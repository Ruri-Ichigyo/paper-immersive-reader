use super::WebImportError;

pub trait PdfTextBridge {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, WebImportError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UnavailablePdfTextBridge;

impl PdfTextBridge for UnavailablePdfTextBridge {
    fn extract_text(&self, _bytes: &[u8]) -> Result<String, WebImportError> {
        Err(WebImportError::PdfBridgeUnavailable)
    }
}
