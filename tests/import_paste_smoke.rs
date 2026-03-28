use paper_immersive_reader_core::domain::{PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::{DefaultImportPipeline, ImportPipeline, ImportRequest};

#[test]
fn paste_text_import_produces_paper_blocks_and_progress() {
    let pipeline = DefaultImportPipeline::new_for_tests();

    let result = pipeline
        .import(ImportRequest::PasteText {
            text: "p1\n\n p2".to_string(),
            title: Some("t".to_string()),
            source_ref: None,
        })
        .expect("import should succeed");

    assert_eq!(result.paper.source_type, PaperSourceType::Paste);
    assert_eq!(result.paper.status, PaperStatus::ReadyText);
    assert!(result.blocks.len() >= 2);
    assert_eq!(result.blocks[0].order_index, 0);
    assert_eq!(result.reading_progress.current_block_index, 0);
}
