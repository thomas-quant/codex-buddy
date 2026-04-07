use buddy_wrapper::app::{App, AppAction, UiFocus};

#[test]
fn tab_toggles_between_pty_and_buddy() {
    let mut app = App::new_for_test();
    app.apply(AppAction::ToggleFocus);
    assert_eq!(app.focus(), UiFocus::BuddyPane);
    app.apply(AppAction::ToggleFocus);
    assert_eq!(app.focus(), UiFocus::Pty);
}

#[test]
fn enter_opens_action_menu_only_when_buddy_is_focused() {
    let mut app = App::new_for_test();
    app.apply(AppAction::OpenBuddyMenu);
    assert!(!app.is_buddy_menu_open());
    app.apply(AppAction::ToggleFocus);
    app.apply(AppAction::OpenBuddyMenu);
    assert!(app.is_buddy_menu_open());
}
