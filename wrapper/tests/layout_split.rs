use buddy_wrapper::ui::layout::split_main_and_buddy;
use ratatui::layout::Rect;

#[test]
fn buddy_pane_overlays_the_lower_right_without_shrinking_codex() {
    let [main, buddy] = split_main_and_buddy(Rect::new(0, 0, 120, 40));

    assert_eq!(main.x, 0);
    assert_eq!(main.y, 0);
    assert_eq!(main.width, 120);
    assert_eq!(main.height, 40);

    assert_eq!(buddy.x, 94);
    assert_eq!(buddy.y, 29);
    assert_eq!(buddy.width, 26);
    assert_eq!(buddy.height, 11);
}
