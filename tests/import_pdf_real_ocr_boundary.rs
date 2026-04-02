use std::fs;

use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn pdf_import_with_no_usable_text_requires_ocr() {
    let pdf_bytes =
        fs::read("tests/fixtures/pdf/no-usable-text.pdf").expect("fixture should be readable");

    let pipeline = DefaultImportPipeline::new();

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: pdf_bytes,
            title: Some("No usable text fixture".to_string()),
            source_ref: Some("tests/fixtures/pdf/no-usable-text.pdf".to_string()),
        })
        .expect_err("pdf import should require OCR when no usable text is extracted");

    assert_eq!(err, ImportPipelineError::OcrRequired);
}
