use std::path::PathBuf;

pub mod logging;

pub fn workspace_marker() -> &'static str {
    "core"
}

pub fn workspace_root_marker() -> PathBuf {
    PathBuf::from(".")
}
