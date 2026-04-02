use paper_immersive_reader_core::domain::OcrEngine;

#[test]
fn ocr_engine_is_exported() {
    let engine = OcrEngine::PaddleOcr;

    assert_eq!(engine, OcrEngine::PaddleOcr);
}
