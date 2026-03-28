use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};

#[test]
fn paper_can_be_constructed() {
    let paper = Paper {
        id: PaperId("p1".to_string()),
        title: None,
        source_type: PaperSourceType::Pdf,
        source_ref: None,
        status: PaperStatus::Created,
        parse_version: 1,
    };

    assert_eq!(paper.parse_version, 1);
}
