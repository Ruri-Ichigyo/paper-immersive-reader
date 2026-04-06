pub mod import;
pub mod library;
pub mod read;
pub mod settings;
pub mod vocabulary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDescriptor {
    pub path: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

pub fn all() -> Vec<RouteDescriptor> {
    vec![
        library::descriptor(),
        import::descriptor(),
        read::descriptor(),
        vocabulary::descriptor(),
        settings::descriptor(),
    ]
}
