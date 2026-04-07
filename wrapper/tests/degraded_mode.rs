use buddy_wrapper::app::App;

#[test]
fn quip_failure_clears_the_active_bubble() {
    let mut app = App::new_for_test();
    app.set_active_quip_for_test("hello");
    app.handle_quip_failure();
    assert_eq!(app.active_quip(), None);
}
