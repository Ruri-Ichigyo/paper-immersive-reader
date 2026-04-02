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

    let pipeline = DefaultImportPipeline::new_for_tests();
    _assert_pipeline(&pipeline);

    let _maybe: Option<ImportResult> = None;
    assert!(_maybe.is_none());
}

#[test]
fn default_and_test_pdf_pipeline_constructors_return_typed_errors() {
    let _default_pipeline = DefaultImportPipeline::new();

    let test_pipeline = DefaultImportPipeline::new_for_tests();

    let err = test_pipeline
        .import(ImportRequest::PdfDocument {
            bytes: vec![1, 2, 3],
            title: Some("paper".to_string()),
            source_ref: None,
        })
        .expect_err("test PDF constructor should return a typed seam error");

    assert_eq!(err, ImportPipelineError::PdfExtractionFailed);
}
