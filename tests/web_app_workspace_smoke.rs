#[test]
fn web_app_crate_is_linkable_from_workspace() {
    let shell = paper_immersive_reader_web_app::app::AppShell::default();

    assert_eq!(shell.title, "Paper Immersive Reader");
}
