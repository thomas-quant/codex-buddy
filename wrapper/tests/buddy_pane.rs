use buddy_wrapper::buddy::store::PersistedBuddy;
use buddy_wrapper::buddy::types::CompanionBones;
use buddy_wrapper::ui::buddy_pane::{render_idle_lines, render_status_lines};
use chrono::Utc;

#[test]
fn idle_view_hides_personality_text() {
    let lines = render_idle_lines(
        &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
        &CompanionBones::test_fixture(),
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
        Utc::now(),
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("observant little goblin"))
    );
}
