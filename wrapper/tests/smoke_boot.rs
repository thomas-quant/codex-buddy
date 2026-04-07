use buddy_wrapper::app::App;

#[test]
fn app_constructs_without_starting_codex() {
    let app = App::new_for_test();
    assert_eq!(app.focus_label(), "pty");
    assert!(!app.has_buddy());
}
