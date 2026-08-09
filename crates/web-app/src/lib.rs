pub mod ai;
pub mod annotations;
pub mod app;
pub mod components;
pub mod export;
pub mod import;
pub mod reading;
pub mod routes;
pub mod selection;
pub mod state;
pub mod storage;
pub mod util;

pub fn workspace_marker() -> &'static str {
    "web-app"
}
