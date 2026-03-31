use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest, ImportResult,
};

#[test]
fn importing_types_exist() {
    let _req = ImportRequest::PasteText {
        text: "hello".to_string(),
        title: None,
        source_ref: None,
    };

    let _pdf = ImportRequest::PdfDocument {
        bytes: vec![1, 2, 3],
        title: Some("paper".to_string()),
        source_ref: None,
    };

    let _ = ImportPipelineError::PdfExtractionFailed;
    let _ = ImportPipelineError::OcrRequired;

    fn _assert_pipeline<T: ImportPipeline>(_t: &T) {}

    let _maybe: Option<ImportResult> = None;
    assert!(_maybe.is_none());
}

#[test]
fn pdf_document_placeholder_returns_extraction_failed_instead_of_panicking() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let err = pipeline
        .import(ImportRequest::PdfDocument {
            bytes: vec![1, 2, 3],
            title: Some("paper".to_string()),
            source_ref: None,
        })
        .expect_err("PDF placeholder should return a typed error");

    assert_eq!(err, ImportPipelineError::PdfExtractionFailed);
}
