use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn pdf_import_maps_extractor_failures_to_pdf_extraction_failed() {
    let pipeline = DefaultImportPipeline::new_for_tests_with_pdf_failure();

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: b"%PDF-1.4 fake".to_vec(),
            title: None,
            source_ref: None,
        })
        .expect_err("pdf import should return a typed extraction failure");

    assert_eq!(err, ImportPipelineError::PdfExtractionFailed);
}
