use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use paper_immersive_reader_application::orchestration::{LocalOcrExecutor, ScriptPaddleOcrRunner};
use paper_immersive_reader_application::persistence::{
    SqliteAppStateStore, SqliteAppStateStoreError, DEFAULT_APP_DB_PATH,
};
use paper_immersive_reader_core::domain::{OcrJobId, OcrJobStatus};
use paper_immersive_reader_core::logging;

fn main() -> ExitCode {
    let _ = logging::log_file_path();

    match run(std::env::args().skip(1).collect()) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    match args.as_slice() {
        [ocr, run_next] if ocr == "ocr" && run_next == "run-next" => run_next_job(),
        [ocr, run, job_id] if ocr == "ocr" && run == "run" => run_job(job_id),
        _ => Err("Usage: paper-immersive-reader-cli ocr run-next | ocr run <job-id>".to_string()),
    }
}

fn run_next_job() -> Result<String, String> {
    let store = SqliteAppStateStore::open_or_migrate(DEFAULT_APP_DB_PATH)
        .map_err(format_load_error)?;
    let queued_job = store
        .first_queued_job()
        .ok_or_else(|| "No queued OCR job found.".to_string())?;
    let job_id = queued_job.id.clone();
    let mut executor = LocalOcrExecutor::new(
        store,
        ScriptPaddleOcrRunner::new(PathBuf::from("scripts/run_paddle_ocr.sh")),
        current_time_ms as fn() -> u64,
    );

    match executor.run_job(&job_id) {
        Some(completed_job) => Ok(format!(
            "Completed OCR job {} for paper {}.",
            completed_job.id.0, completed_job.paper_id.0
        )),
        None => Err(format_job_failure(&executor, &job_id)),
    }
}

fn run_job(job_id: &str) -> Result<String, String> {
    let job_id = OcrJobId(job_id.to_string());
    let store = SqliteAppStateStore::open_or_migrate(DEFAULT_APP_DB_PATH)
        .map_err(format_load_error)?;
    let queued_job = store
        .get_job(&job_id)
        .ok_or_else(|| format!("OCR job not found: {}", job_id.0))?;

    if queued_job.status != OcrJobStatus::Queued {
        return Err(format!("OCR job is not queued: {}", job_id.0));
    }

    let mut executor = LocalOcrExecutor::new(
        store,
        ScriptPaddleOcrRunner::new(PathBuf::from("scripts/run_paddle_ocr.sh")),
        current_time_ms as fn() -> u64,
    );

    match executor.run_job(&job_id) {
        Some(completed_job) => Ok(format!(
            "Completed OCR job {} for paper {}.",
            completed_job.id.0, completed_job.paper_id.0
        )),
        None => Err(format_job_failure(&executor, &job_id)),
    }
}

fn format_job_failure(
    executor: &LocalOcrExecutor<SqliteAppStateStore, ScriptPaddleOcrRunner, fn() -> u64>,
    job_id: &OcrJobId,
) -> String {
    match executor.store().get_job(job_id) {
        Some(job) if job.status == OcrJobStatus::Failed => match job.error_message {
            Some(message) => format!("OCR job {} failed: {message}", job_id.0),
            None => format!("OCR job {} failed.", job_id.0),
        },
        Some(_) => format!("OCR job {} did not complete.", job_id.0),
        None => format!("OCR job {} could not be loaded after execution.", job_id.0),
    }
}

fn format_load_error(error: SqliteAppStateStoreError) -> String {
    format!("Failed to load app state from {DEFAULT_APP_DB_PATH}: {error}")
}

fn current_time_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}
