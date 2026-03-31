use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest, PdfTextExtractor,
};

struct ByteCountingExtractor;

impl PdfTextExtractor for ByteCountingExtractor {
    fn extract_text(&self, bytes: &[u8]) -> Result<String, ImportPipelineError> {
        Ok(format!("{} bytes", bytes.len()))
    }
}

#[test]
fn pdf_import_with_injected_extractor_uses_custom_extraction_seam() {
    let pipeline = DefaultImportPipeline::new_with_pdf_extractor(ByteCountingExtractor);

    let result = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF-1.4 fake".to_vec(),
            title: None,
            source_ref: None,
        })
        .expect("pdf import should succeed with injected extractor");

    assert_eq!(result.paper.source_type, PaperSourceType::Pdf);
    assert_eq!(result.paper.status, PaperStatus::ReadyText);
    assert_eq!(result.blocks.len(), 1);
    assert_eq!(result.blocks[0].text, "13 bytes");
}
