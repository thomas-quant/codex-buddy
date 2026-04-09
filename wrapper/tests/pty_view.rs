use buddy_wrapper::ui::pty_view::{PtyRenderFilter, PtyView};
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use vt100::Parser;

#[test]
fn pty_view_renders_without_box_borders() {
    let mut parser = Parser::new(4, 20, 0);
    parser.process(b"hello");

    let widget = PtyView::new().render(parser.screen(), PtyRenderFilter::default());
    let area = Rect::new(0, 0, 20, 4);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let rendered = (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!rendered.contains('┌'));
    assert!(!rendered.contains('┐'));
    assert!(!rendered.contains('│'));
    assert!(!rendered.contains('─'));
}

#[test]
fn pty_view_preserves_terminal_background_colors() {
    let mut parser = Parser::new(4, 20, 0);
    parser.process(b"\x1b[44m> Write tests   \x1b[0m");

    let widget = PtyView::new().render(parser.screen(), PtyRenderFilter::default());
    let area = Rect::new(0, 0, 20, 4);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    assert_eq!(buffer[(0, 0)].symbol(), ">");
    assert_eq!(buffer[(0, 0)].bg, Color::Blue);
    assert_eq!(buffer[(10, 0)].bg, Color::Blue);
}

#[test]
fn pty_view_filters_codex_activity_rows() {
    let mut parser = Parser::new(6, 30, 0);
    parser.process(
        b"\xe2\x80\xa2 Starting MCP servers\r\n\xe2\x80\xa2 Explored\r\n\xe2\x94\x94 Read SKILL.md\r\n\xe2\x80\xa2 hi\r\n\xe2\x80\xba prompt",
    );

    let widget = PtyView::new().render(parser.screen(), PtyRenderFilter::default());
    let area = Rect::new(0, 0, 30, 6);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let rendered = (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!rendered.contains("Starting MCP servers"));
    assert!(!rendered.contains("Explored"));
    assert!(!rendered.contains("Read SKILL.md"));
    assert!(rendered.contains("hi"));
    assert!(rendered.contains("prompt"));
}

#[test]
fn pty_view_filters_recent_commentary_rows() {
    let mut parser = Parser::new(4, 60, 0);
    parser.process(
        b"\xe2\x80\xa2 I\xe2\x80\x99m checking the workspace guidance first\r\n\xe2\x80\xba prompt",
    );

    let filter = PtyRenderFilter::new(["I’m checking the workspace guidance first"]);
    let widget = PtyView::new().render(parser.screen(), filter);
    let area = Rect::new(0, 0, 60, 4);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let rendered = (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!rendered.contains("checking the workspace guidance"));
    assert!(rendered.contains("prompt"));
}
