use paper_immersive_reader_core::domain::{Paper, PaperId, PaperSourceType, PaperStatus};
use paper_immersive_reader_importing::segment_text_for_existing_paper;

#[test]
fn existing_paper_text_segmentation_reuses_given_paper_id() {
    let paper = Paper {
        id: PaperId("paper-ocr-1".to_string()),
        title: Some("OCR paper".to_string()),
        source_type: PaperSourceType::Pdf,
        source_ref: Some("scan.pdf".to_string()),
        status: PaperStatus::ReadyOcr,
        parse_version: 1,
    };

    let artifacts = segment_text_for_existing_paper(&paper, "first\n\nsecond".to_string(), 123)
        .expect("segmentation should succeed");

    assert_eq!(artifacts.blocks.len(), 2);
    assert_eq!(artifacts.blocks[0].paper_id, paper.id);
    assert_eq!(
        artifacts.reading_progress.paper_id,
        PaperId("paper-ocr-1".to_string())
    );
    assert_eq!(artifacts.reading_progress.updated_at_ms, 123);
}
