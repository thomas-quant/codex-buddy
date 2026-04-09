use buddy_wrapper::buddy::store::PersistedBuddy;
use buddy_wrapper::buddy::types::CompanionBones;
use buddy_wrapper::ui::buddy_pane::{
    BuddyMenuEntry, render_action_menu_lines, render_idle_lines, render_status_lines,
};
use chrono::Utc;

#[test]
fn idle_view_renders_the_requested_sprite_frame() {
    let buddy = PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin.");
    let bones = CompanionBones::test_fixture();

    let frame_zero = render_idle_lines(&buddy, &bones, 0, None, false);
    let frame_one = render_idle_lines(&buddy, &bones, 1, None, false);

    assert_eq!(frame_zero[4], "    `--´    ");
    assert_eq!(frame_one[4], "    `--´~   ");
}

#[test]
fn status_view_renders_the_requested_sprite_frame() {
    let buddy = PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin.");
    let bones = CompanionBones::test_fixture();

    let frame_zero = render_status_lines(&buddy, &bones, 0, Utc::now());
    let frame_one = render_status_lines(&buddy, &bones, 1, Utc::now());

    assert_eq!(frame_zero[4], "    `--´    ");
    assert_eq!(frame_one[4], "    `--´~   ");
}

#[test]
fn idle_view_hides_personality_text() {
    let lines = render_idle_lines(
        &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
        &CompanionBones::test_fixture(),
        0,
        None,
        false,
    );
    assert!(
        lines
            .iter()
            .all(|line| !line.contains("observant little goblin"))
    );
}

#[test]
fn status_view_shows_personality_text() {
    let lines = render_status_lines(
        &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
        &CompanionBones::test_fixture(),
        0,
        Utc::now(),
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("observant little goblin"))
    );
}

#[test]
fn action_menu_view_shows_status_as_a_selectable_item() {
    let lines = render_action_menu_lines(
        &[
            BuddyMenuEntry::new("Status", true, true),
            BuddyMenuEntry::new("Pet", false, true),
            BuddyMenuEntry::new("Mute", false, true),
        ],
        Some("Buddy is listening again."),
    );

    assert!(lines.iter().any(|line| line == "Actions"));
    assert!(lines.iter().any(|line| line == "> Status"));
    assert!(lines.iter().any(|line| line == "  Pet"));
    assert!(lines.iter().any(|line| line.contains("Enter: choose")));
}
