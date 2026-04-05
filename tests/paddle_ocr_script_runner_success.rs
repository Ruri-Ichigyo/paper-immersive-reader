use paper_immersive_reader_application::orchestration::{PaddleOcrRunner, ScriptPaddleOcrRunner};
use paper_immersive_reader_core::domain::{OcrEngine, OcrJob, OcrJobId, OcrJobStatus, PaperId};
use std::os::unix::fs::PermissionsExt;

#[test]
fn script_runner_reads_trimmed_ocr_text_from_stdout() {
    let temp_root = std::env::temp_dir().join("paper-immersive-reader-script-runner-success");
    let _ = std::fs::remove_dir_all(&temp_root);
    std::fs::create_dir_all(&temp_root).expect("temp root should exist");

    let script_path = temp_root.join("run.sh");
    std::fs::write(&script_path, "#!/bin/sh\nprintf 'recognized text\\n'\n")
        .expect("script should write");
    let mut permissions = std::fs::metadata(&script_path)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&script_path, permissions).expect("script should be executable");

    let pdf_path = temp_root.join("paper-1.pdf");
    std::fs::write(&pdf_path, b"%PDF fake asset").expect("pdf asset should write");

    let runner = ScriptPaddleOcrRunner::new(script_path);
    let output = runner
        .run(&sample_job(), &pdf_path)
        .expect("runner should succeed");

    assert_eq!(output.text, "recognized text");
}

fn sample_job() -> OcrJob {
    OcrJob {
        id: OcrJobId("job-1".to_string()),
        paper_id: PaperId("paper-1".to_string()),
        engine: OcrEngine::PaddleOcr,
        status: OcrJobStatus::Queued,
        requested_at_ms: 1,
        started_at_ms: None,
        finished_at_ms: None,
        error_message: None,
    }
}
