use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::Local;
use tracing_appender::non_blocking::WorkerGuard;

pub fn init_gateway_logging() -> io::Result<(WorkerGuard, PathBuf)> {
    fs::create_dir_all("logs")?;

    let file_name = format!("{}.log", Local::now().format("%Y-%m-%d_%H%M%S"));
    let file_appender = tracing_appender::rolling::never("logs", &file_name);
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    let _ = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_level(true)
        .with_target(true)
        .with_writer(writer)
        .try_init();

    Ok((guard, PathBuf::from("logs").join(file_name)))
}
