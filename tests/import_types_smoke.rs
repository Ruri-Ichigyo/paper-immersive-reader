use paper_immersive_reader_importing::{ImportPipeline, ImportRequest, ImportResult};

#[test]
fn importing_types_exist() {
    let _req = ImportRequest::PasteText {
        text: "hello".to_string(),
        title: None,
        source_ref: None,
    };

    fn _assert_pipeline<T: ImportPipeline>(_t: &T) {}

    let _maybe: Option<ImportResult> = None;
    assert!(_maybe.is_none());
}
