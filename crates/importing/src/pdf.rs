use crate::{ImportPipelineError, PdfTextExtractor};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RealPdfTextExtractor;

impl PdfTextExtractor for RealPdfTextExtractor {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, ImportPipelineError> {
        pdf_extract::extract_text_from_mem(bytes).map_err(|_| ImportPipelineError::OcrRequired)
    }
}
