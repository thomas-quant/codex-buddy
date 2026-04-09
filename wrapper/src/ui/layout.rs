use ratatui::layout::Rect;

pub const BUDDY_HINT_FOOTER: &str = "Tab: focus  Enter: actions";

pub fn split_main_and_buddy(area: Rect) -> [Rect; 2] {
    let buddy_width = area.width.min(26);
    let buddy_height = area.height.min(11);
    let buddy_x = area
        .x
        .saturating_add(area.width.saturating_sub(buddy_width));
    let buddy_y = area
        .y
        .saturating_add(area.height.saturating_sub(buddy_height));
    let buddy = Rect::new(buddy_x, buddy_y, buddy_width, buddy_height);

    [area, buddy]
}
