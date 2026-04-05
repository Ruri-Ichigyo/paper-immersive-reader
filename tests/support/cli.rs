use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

pub fn create_temp_workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("database")).expect("database dir should exist");
    std::fs::create_dir_all(root.join("scripts")).expect("scripts dir should exist");
    root
}

pub fn write_executable_script(path: &Path, contents: &str) {
    std::fs::write(path, contents).expect("script should write");
    let mut permissions = std::fs::metadata(path)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("script should be executable");
}

pub fn run_cli(workspace_root: &Path, args: &[&str]) -> Output {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

    let mut command = std::process::Command::new(cargo);
    command
        .current_dir(workspace_root)
        .arg("run")
        .arg("-q")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("-p")
        .arg("paper-immersive-reader-cli")
        .arg("--");

    for arg in args {
        command.arg(arg);
    }

    command.output().expect("CLI command should run")
}
