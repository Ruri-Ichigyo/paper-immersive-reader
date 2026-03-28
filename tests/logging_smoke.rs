use std::path::PathBuf;

#[test]
fn log_path_is_under_logs_dir_and_has_log_extension() {
    let path: PathBuf = paper_immersive_reader_core::logging::log_file_path();

    let extension = path.extension().and_then(std::ffi::OsStr::to_str);
    assert_eq!(extension, Some("log"));

    let parent_dir_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(std::ffi::OsStr::to_str);

    assert_eq!(parent_dir_name, Some("logs"));
}
