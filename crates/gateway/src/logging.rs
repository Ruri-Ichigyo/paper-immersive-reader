use std::fs;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

pub fn init_logging() -> WorkerGuard {
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir).ok();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Format: logs/YYYY-MM-DD_HHMMSS.log
    let dt = chrono_lite(&now);
    let filename = format!("{}.log", dt);
    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::NEVER)
        .filename_prefix("")
        .filename_suffix("")
        .max_log_files(1)
        .build(log_dir.join(filename))
        .expect("failed to create log file appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(non_blocking)
        .with_target(false)
        .init();

    guard
}

fn chrono_lite(ts_secs: &u64) -> String {
    // Manual UTC date formatting without chrono dependency
    let secs = *ts_secs;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let secs = time_of_day % 60;

    // Calculate date from days since Unix epoch using civil calendar algorithm
    let (year, month, day) = civil_from_days(days_since_epoch as i64);

    format!(
        "{:04}-{:02}-{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, secs
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Algorithm from Howard Hinnant
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
