use crate::routes::{self, RouteDescriptor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppShell {
    pub title: &'static str,
    pub routes: Vec<RouteDescriptor>,
}

impl Default for AppShell {
    fn default() -> Self {
        Self {
            title: "Paper Immersive Reader",
            routes: routes::all(),
        }
    }
}
