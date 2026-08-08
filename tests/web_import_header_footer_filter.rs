/// Verify that filter_repeated_lines strips repeated header/footer lines
/// from multi-page PDF text.  Since filter_repeated_lines is private,
/// we test via the public segment_text path through import_paste.
///
/// For a more direct test, we verify that pasting text with form-feed
/// separated pages and repeated headers produces blocks without the
/// repeated header content.
#[test]
fn import_paste_with_repeated_lines_filters_them() {
    use paper_immersive_reader_web_app::import::WebImportService;
    use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
    use paper_immersive_reader_web_app::storage::AppStorage;

    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    // Simulate 4 pages with a repeated header "Journal of Testing"
    // and a repeated footer "Page X"
    let text = "Journal of Testing\n\nFirst page body text here.\n\nPage 1\x0cJournal of Testing\n\nSecond page body content.\n\nPage 2\x0cJournal of Testing\n\nThird page body paragraph.\n\nPage 3\x0cJournal of Testing\n\nFourth page body data.\n\nPage 4";

    let paper = service.import_paste(text.to_string(), None).unwrap();
    let store = service.into_store();
    let blocks = store.load_blocks(&paper.id).unwrap();

    // The repeated header "Journal of Testing" should not appear in any block
    for block in &blocks {
        assert!(
            !block.text.contains("Journal of Testing"),
            "Header leaked into block: {}",
            block.text
        );
    }

    // Body text should be present
    let all_text: String = blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(all_text.contains("First page body text"));
    assert!(all_text.contains("Fourth page body data"));
}

/// Verify that single-newline fallback segmentation works when
/// text has no double-newline paragraph breaks.
#[test]
fn import_paste_single_newline_segmentation() {
    use paper_immersive_reader_web_app::import::WebImportService;
    use paper_immersive_reader_web_app::storage::memory_store::InMemoryStore;
    use paper_immersive_reader_web_app::storage::AppStorage;

    let store = InMemoryStore::new();
    let mut service = WebImportService::new(store);

    // Text with only single newlines between paragraphs (common in PDF extraction)
    let text = "This is the first paragraph and it is long enough to pass the threshold.\nThis is the second paragraph and it also exceeds twenty characters.\nThis is the third paragraph with enough length to be included.";

    let paper = service.import_paste(text.to_string(), None).unwrap();
    let store = service.into_store();
    let blocks = store.load_blocks(&paper.id).unwrap();

    assert!(
        blocks.len() >= 3,
        "Expected at least 3 blocks from single-newline text, got {}",
        blocks.len()
    );
}
