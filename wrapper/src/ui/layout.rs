use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub const BUDDY_HINT_FOOTER: &str = "Tab: focus  Enter: actions";

pub fn split_main_and_buddy(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(34)])
        .split(area);

    [chunks[0], chunks[1]]
}
