/// Current time in milliseconds since the Unix epoch.
///
/// On `wasm32` we use `js_sys::Date::now()` because `std::time::SystemTime`
/// is not supported on `wasm32-unknown-unknown` and returns `Err` (yielding 0
/// via `unwrap_or_default`).
#[cfg(target_arch = "wasm32")]
pub fn current_time_ms() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Generate a short unique ID using the first segment of a UUID v4.
pub fn short_id(prefix: &str) -> String {
    format!(
        "{}-{}",
        prefix,
        uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("0")
    )
}

/// Format a millisecond timestamp as a human-readable relative time.
pub fn format_relative_time(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    if days > 0 {
        format!("{}d ago", days)
    } else if hours > 0 {
        format!("{}h ago", hours)
    } else if mins > 0 {
        format!("{}m ago", mins)
    } else {
        "just now".to_string()
    }
}
