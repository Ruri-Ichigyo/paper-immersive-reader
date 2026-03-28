#[test]
fn core_crate_is_linkable() {
    let version = paper_immersive_reader_core::workspace_marker();
    assert_eq!(version, "core");
}
