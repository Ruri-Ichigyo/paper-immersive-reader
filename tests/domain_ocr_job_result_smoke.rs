use paper_immersive_reader_core::domain::{OcrJobResult, PaperId};

#[test]
fn ocr_job_result_can_be_constructed() {
    let result = OcrJobResult {
        paper_id: PaperId("paper-1".to_string()),
        text: "recognized text".to_string(),
    };

    assert_eq!(result.paper_id, PaperId("paper-1".to_string()));
    assert_eq!(result.text, "recognized text".to_string());
}
