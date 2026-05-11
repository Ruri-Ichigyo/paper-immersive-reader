#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfBridgeError {
    ExtractionFailed,
    NoTextFound,
}

pub trait PdfTextBridge {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, PdfBridgeError>;
}

/// Stub extractor for native testing. In browser, this is replaced by pdf.js via wasm-bindgen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StubPdfBridge {
    pub text: Option<&'static str>,
}

impl StubPdfBridge {
    pub fn with_text(text: &'static str) -> Self {
        Self { text: Some(text) }
    }

    pub fn failing() -> Self {
        Self { text: None }
    }
}

impl PdfTextBridge for StubPdfBridge {
    fn extract_text(&self, _bytes: &[u8]) -> Result<String, PdfBridgeError> {
        match self.text {
            Some(t) if !t.trim().is_empty() => Ok(t.to_string()),
            _ => Err(PdfBridgeError::NoTextFound),
        }
    }
}
