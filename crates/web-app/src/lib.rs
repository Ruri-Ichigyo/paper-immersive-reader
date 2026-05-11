pub mod app;
pub mod import;
pub mod routes;
pub mod storage;

pub fn workspace_marker() -> &'static str {
    "web-app"
}
