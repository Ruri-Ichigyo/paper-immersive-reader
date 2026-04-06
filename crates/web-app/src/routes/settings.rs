use super::RouteDescriptor;

pub fn descriptor() -> RouteDescriptor {
    RouteDescriptor {
        path: "/settings",
        label: "Settings",
        description: "Configure MVP reader and AI preferences.",
    }
}
