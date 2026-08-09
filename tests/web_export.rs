use paper_immersive_reader_core::domain::{
    Block, BlockId, Paper, PaperId, PaperSourceType, PaperStatus, VocabularyId, VocabularyItem,
};
use paper_immersive_reader_web_app::export::ExportBundle;
use paper_immersive_reader_web_app::storage::schema::{ReadingStats, StoredPaper};

fn make_paper(id: &str, title: Option<&str>) -> StoredPaper {
    StoredPaper {
        paper: Paper {
            id: PaperId(id.to_string()),
            title: title.map(|s| s.to_string()),
            source_type: PaperSourceType::Paste,
            source_ref: None,
            status: PaperStatus::ReadyText,
            parse_version: 1,
        },
        created_at_ms: 1000,
        updated_at_ms: 2000,
    }
}

fn make_block(paper_id: &str, idx: u32, text: &str) -> Block {
    Block::try_new(
        BlockId(format!("block-{}", idx)),
        PaperId(paper_id.to_string()),
        idx,
        text.to_string(),
        None,
    )
    .unwrap()
}

#[test]
fn export_bundle_serializes_to_json() {
    let bundle = ExportBundle {
        paper: make_paper("p1", Some("Test Paper")),
        blocks: vec![
            make_block("p1", 0, "First block."),
            make_block("p1", 1, "Second block."),
        ],
        annotations: vec![],
        vocabulary: vec![],
        reading_stats: None,
    };

    let json = bundle.to_json().unwrap();
    assert!(json.contains("Test Paper"));
    assert!(json.contains("First block."));
    assert!(json.contains("Second block."));
}

#[test]
fn export_bundle_excludes_ai_records() {
    // The ExportBundle struct does not have an ai_records field.
    // This test verifies the struct layout at compile time - if someone
    // adds ai_records to ExportBundle, this test will fail to compile.
    let bundle = ExportBundle {
        paper: make_paper("p1", None),
        blocks: vec![],
        annotations: vec![],
        vocabulary: vec![],
        reading_stats: None,
    };

    let json = bundle.to_json().unwrap();
    // Ensure no AI record fields leak into the export
    assert!(!json.contains("ai_record"));
    assert!(!json.contains("translation"));
    assert!(!json.contains("explain_term"));
}

#[test]
fn export_filename_sanitizes_title() {
    let bundle = ExportBundle {
        paper: make_paper("p1", Some("Attention Is All You Need: A Study")),
        blocks: vec![],
        annotations: vec![],
        vocabulary: vec![],
        reading_stats: None,
    };

    let filename = bundle.filename();
    assert!(filename.ends_with("_export.json"));
    assert!(!filename.contains(":"));
    assert!(!filename.contains(" "));
}

#[test]
fn export_filename_defaults_to_untitled() {
    let bundle = ExportBundle {
        paper: make_paper("p1", None),
        blocks: vec![],
        annotations: vec![],
        vocabulary: vec![],
        reading_stats: None,
    };

    assert_eq!(bundle.filename(), "untitled_export.json");
}

#[test]
fn export_via_app_state_includes_all_data() {
    use paper_immersive_reader_web_app::state::AppState;

    let app = AppState::new();
    let paper = make_paper("p1", Some("My Paper"));
    app.save_paper(&paper);

    let blocks = vec![
        make_block("p1", 0, "Block one."),
        make_block("p1", 1, "Block two."),
    ];
    app.save_blocks(&blocks);

    let vocab = VocabularyItem {
        id: VocabularyId("v1".to_string()),
        term: "epistemology".to_string(),
        term_lower: "epistemology".to_string(),
        paper_id_first_seen: PaperId("p1".to_string()),
        block_id_first_seen: BlockId("block-0".to_string()),
        lookup_count: 3,
        first_seen_at_ms: 5000,
        last_seen_at_ms: 6000,
        note: Some("knowledge theory".to_string()),
        learned: false,
    };
    app.upsert_vocabulary(&vocab);

    let stats = ReadingStats {
        paper_id: PaperId("p1".to_string()),
        total_reading_ms: 120_000,
        session_started_ms: None,
        session_count: 2,
    };
    app.save_reading_stats(&stats);

    let bundle = app.export_paper(&PaperId("p1".to_string()));
    assert!(bundle.is_some());

    let bundle = bundle.unwrap();
    assert_eq!(bundle.paper.paper.title, Some("My Paper".to_string()));
    assert_eq!(bundle.blocks.len(), 2);
    assert_eq!(bundle.vocabulary.len(), 1);
    assert_eq!(bundle.vocabulary[0].term, "epistemology");
    assert!(bundle.reading_stats.is_some());
    assert_eq!(bundle.reading_stats.unwrap().total_reading_ms, 120_000);
}
