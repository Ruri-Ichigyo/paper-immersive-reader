use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn paste_text_splits_on_double_newline_and_drops_empty_segments() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let result = pipeline
        .import(ImportRequest::PasteText {
            text: "a\n\n\n\n b\n\n  \n\n c".to_string(),
            title: None,
            source_ref: None,
        })
        .expect("import should succeed");

    assert_eq!(result.blocks.len(), 3);
}
