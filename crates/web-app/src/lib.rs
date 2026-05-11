pub mod app;
pub mod components;
pub mod import;
pub mod reading;
pub mod routes;
pub mod storage;

pub fn workspace_marker() -> &'static str {
    "web-app"
}
