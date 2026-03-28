use paper_immersive_reader_importing::{
    DefaultImportPipeline, ImportPipeline, ImportPipelineError, ImportRequest,
};

#[test]
fn paste_text_empty_input_is_rejected() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let err = pipeline
        .import(ImportRequest::PasteText {
            text: "   ".to_string(),
            title: None,
            source_ref: None,
        })
        .expect_err("should fail");

    assert_eq!(err, ImportPipelineError::EmptyInput);
}
