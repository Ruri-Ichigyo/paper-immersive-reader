#[path = "support/cli.rs"]
mod cli_support;

#[test]
fn cli_ocr_run_returns_non_zero_when_job_id_is_missing() {
    let temp_root = cli_support::create_temp_workspace("paper-immersive-reader-cli-run-missing");
    cli_support::write_executable_script(
        &temp_root.join("scripts").join("run_paddle_ocr.sh"),
        "#!/bin/sh\nprintf 'recognized text\\n'\n",
    );

    let output = cli_support::run_cli(&temp_root, &["ocr", "run", "job-404"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("OCR job not found"));
}
