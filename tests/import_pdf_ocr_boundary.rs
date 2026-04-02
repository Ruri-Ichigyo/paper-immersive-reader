use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn pdf_import_requires_ocr_when_extracted_text_is_empty_after_trimming() {
    let pipeline = DefaultImportPipeline::new_for_tests_with_pdf_text("  \n\n  ");

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF-1.4 fake".to_vec(),
            title: None,
            source_ref: None,
        })
        .expect_err("pdf import should require OCR when no usable text is extracted");

    assert_eq!(err, ImportPipelineError::OcrRequired);
}
