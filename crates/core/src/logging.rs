use std::path::PathBuf;

pub fn log_file_path() -> PathBuf {
    PathBuf::from("logs").join("placeholder.log")
}
