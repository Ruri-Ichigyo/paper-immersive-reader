use paper_immersive_reader_core::domain::{PaperId, ReadingProgress};

#[test]
fn reading_progress_can_be_constructed() {
    let progress = ReadingProgress {
        paper_id: PaperId("p1".to_string()),
        current_block_index: 3,
        current_block_id: None,
        updated_at_ms: 123,
    };

    assert_eq!(progress.current_block_index, 3);
}
