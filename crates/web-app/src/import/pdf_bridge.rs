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

// --- WASM pdf.js bridge ---

/// Real PDF text extraction via vendored pdf.js.
///
/// In the browser, this calls a JavaScript pdf.js bridge function loaded
/// from `assets/pdfjs/`. The function must be registered on the global scope
/// as `pdfjs_extract_text(data: Uint8Array): string | null`.
#[cfg(target_arch = "wasm32")]
pub struct WasmPdfBridge;

#[cfg(target_arch = "wasm32")]
impl WasmPdfBridge {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = globalThis, js_name = pdfjs_extract_text)]
        fn call_pdfjs_extract(data: &[u8]) -> JsValue;
    }

    pub fn extract(bytes: &[u8]) -> Result<String, super::PdfBridgeError> {
        let result = call_pdfjs_extract(bytes);
        if result.is_null() || result.is_undefined() {
            return Err(super::PdfBridgeError::ExtractionFailed);
        }
        let text = result.as_string().unwrap_or_default();
        if text.trim().is_empty() {
            Err(super::PdfBridgeError::NoTextFound)
        } else {
            Ok(text)
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl PdfTextBridge for WasmPdfBridge {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, PdfBridgeError> {
        wasm_impl::extract(bytes)
    }
}
