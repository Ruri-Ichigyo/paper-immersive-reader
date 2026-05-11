#[test]
fn web_app_crate_is_linkable_from_workspace() {
    let crate_name = env!("CARGO_PKG_NAME");
    assert!(!crate_name.is_empty());
}
