use std::fs;

use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn pdf_import_with_embedded_text_produces_ready_text_paper_blocks_and_progress() {
    let pdf_bytes = fs::read("tests/fixtures/pdf/embedded-text-minimal.pdf")
        .expect("fixture should be readable");

    let pipeline = DefaultImportPipeline::new();

    let result = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: pdf_bytes,
            title: Some("Embedded text fixture".to_string()),
            source_ref: Some("tests/fixtures/pdf/embedded-text-minimal.pdf".to_string()),
        })
        .expect("pdf import should succeed");

    assert_eq!(result.paper.source_type, PaperSourceType::Pdf);
    assert_eq!(result.paper.status, PaperStatus::ReadyText);
    assert!(!result.blocks.is_empty());
    assert_eq!(result.blocks[0].order_index, 0);
    assert_eq!(result.reading_progress.current_block_index, 0);
}
