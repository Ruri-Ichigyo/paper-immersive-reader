#[path = "support/cli.rs"]
mod cli_support;

#[test]
fn cli_ocr_run_next_returns_non_zero_when_no_queued_job_exists() {
    let temp_root =
        cli_support::create_temp_workspace("paper-immersive-reader-cli-run-next-no-jobs");
    cli_support::write_executable_script(
        &temp_root.join("scripts").join("run_paddle_ocr.sh"),
        "#!/bin/sh\nprintf 'recognized text\\n'\n",
    );

    let output = cli_support::run_cli(&temp_root, &["ocr", "run-next"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("No queued OCR job"));
}
