use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use vt100::Parser;

use buddy_wrapper::app::pty_input::{encode_key_for_pty, encode_mouse_for_pty};

#[test]
fn page_keys_are_forwarded_to_the_pty() {
    assert_eq!(
        encode_key_for_pty(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
        Some(b"\x1b[5~".to_vec())
    );
    assert_eq!(
        encode_key_for_pty(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
        Some(b"\x1b[6~".to_vec())
    );
}

#[test]
fn scroll_wheel_is_encoded_for_sgr_mouse_mode() {
    let mut parser = Parser::new(24, 80, 0);
    parser.process(b"\x1b[?1000h\x1b[?1006h");

    let event = MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 3,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };

    assert_eq!(
        encode_mouse_for_pty(event, Rect::new(0, 0, 80, 24), parser.screen()),
        Some(b"\x1b[<64;4;3M".to_vec())
    );
}

#[test]
fn scroll_wheel_is_ignored_when_mouse_mode_is_disabled() {
    let parser = Parser::new(24, 80, 0);
    let event = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 3,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };

    assert_eq!(
        encode_mouse_for_pty(event, Rect::new(0, 0, 80, 24), parser.screen()),
        None
    );
}
