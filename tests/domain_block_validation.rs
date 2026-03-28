use paper_immersive_reader_core::domain::{Block, BlockId, PaperId};

#[test]
fn block_rejects_empty_text() {
    let result = Block::try_new(
        BlockId("b1".to_string()),
        PaperId("p1".to_string()),
        0,
        "   ".to_string(),
        None,
    );

    assert!(result.is_err());
}

#[test]
fn block_accepts_non_empty_text() {
    let block = Block::try_new(
        BlockId("b1".to_string()),
        PaperId("p1".to_string()),
        0,
        "Hello".to_string(),
        None,
    )
    .expect("block should be valid");

    assert_eq!(block.order_index, 0);
}
