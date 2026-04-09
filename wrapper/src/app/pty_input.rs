use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use vt100::{MouseProtocolEncoding, MouseProtocolMode, Screen};

pub fn encode_key_for_pty(key: KeyEvent) -> Option<Vec<u8>> {
    match key.code {
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let lower = ch.to_ascii_lowercase() as u8;
            Some(vec![lower.saturating_sub(b'a') + 1])
        }
        KeyCode::Char(ch) => Some(ch.to_string().into_bytes()),
        _ => None,
    }
}

pub fn encode_mouse_for_pty(event: MouseEvent, area: Rect, screen: &Screen) -> Option<Vec<u8>> {
    let MouseProtocolMode::None = screen.mouse_protocol_mode() else {
        let column = relative_coordinate(event.column, area.x, area.width)?;
        let row = relative_coordinate(event.row, area.y, area.height)?;
        return encode_mouse_report(
            event,
            column + 1,
            row + 1,
            screen.mouse_protocol_mode(),
            screen.mouse_protocol_encoding(),
        );
    };

    None
}

fn relative_coordinate(value: u16, origin: u16, length: u16) -> Option<u16> {
    let relative = value.checked_sub(origin)?;
    (relative < length).then_some(relative)
}

fn encode_mouse_report(
    event: MouseEvent,
    column: u16,
    row: u16,
    mode: MouseProtocolMode,
    encoding: MouseProtocolEncoding,
) -> Option<Vec<u8>> {
    if !mouse_event_supported(event.kind, mode) {
        return None;
    }

    let button = mouse_button_code(event.kind)?;
    let modifiers = modifier_code(event.modifiers);

    match encoding {
        MouseProtocolEncoding::Default => encode_default_mouse(button + modifiers, column, row),
        MouseProtocolEncoding::Utf8 => encode_utf8_mouse(button + modifiers, column, row),
        MouseProtocolEncoding::Sgr => Some(encode_sgr_mouse(
            event.kind,
            button + modifiers,
            column,
            row,
        )),
    }
}

fn mouse_event_supported(kind: MouseEventKind, mode: MouseProtocolMode) -> bool {
    match kind {
        MouseEventKind::Down(_) => mode != MouseProtocolMode::None,
        MouseEventKind::Up(_) => matches!(
            mode,
            MouseProtocolMode::PressRelease
                | MouseProtocolMode::ButtonMotion
                | MouseProtocolMode::AnyMotion
        ),
        MouseEventKind::Drag(_) => {
            matches!(
                mode,
                MouseProtocolMode::ButtonMotion | MouseProtocolMode::AnyMotion
            )
        }
        MouseEventKind::Moved => mode == MouseProtocolMode::AnyMotion,
        MouseEventKind::ScrollUp
        | MouseEventKind::ScrollDown
        | MouseEventKind::ScrollLeft
        | MouseEventKind::ScrollRight => mode != MouseProtocolMode::None,
    }
}

fn mouse_button_code(kind: MouseEventKind) -> Option<u8> {
    Some(match kind {
        MouseEventKind::Down(button) => button_code(button),
        MouseEventKind::Up(_) => 3,
        MouseEventKind::Drag(button) => 32 + button_code(button),
        MouseEventKind::Moved => 35,
        MouseEventKind::ScrollUp => 64,
        MouseEventKind::ScrollDown => 65,
        MouseEventKind::ScrollLeft => 66,
        MouseEventKind::ScrollRight => 67,
    })
}

fn button_code(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}

fn modifier_code(modifiers: KeyModifiers) -> u8 {
    let mut value = 0;
    if modifiers.contains(KeyModifiers::SHIFT) {
        value += 4;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        value += 8;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        value += 16;
    }
    value
}

fn encode_default_mouse(button: u8, column: u16, row: u16) -> Option<Vec<u8>> {
    let button = button.checked_add(32)?;
    let column = u8::try_from(column.checked_add(32)?).ok()?;
    let row = u8::try_from(row.checked_add(32)?).ok()?;
    Some(vec![0x1b, b'[', b'M', button, column, row])
}

fn encode_utf8_mouse(button: u8, column: u16, row: u16) -> Option<Vec<u8>> {
    let button = button.checked_add(32)?;
    let mut bytes = vec![0x1b, b'[', b'M', button];
    push_utf8_codepoint(&mut bytes, column.checked_add(32)?);
    push_utf8_codepoint(&mut bytes, row.checked_add(32)?);
    Some(bytes)
}

fn push_utf8_codepoint(bytes: &mut Vec<u8>, value: u16) {
    let ch = char::from_u32(value.into()).expect("mouse coordinates are valid Unicode");
    let mut buffer = [0_u8; 4];
    let encoded = ch.encode_utf8(&mut buffer);
    bytes.extend_from_slice(encoded.as_bytes());
}

fn encode_sgr_mouse(kind: MouseEventKind, button: u8, column: u16, row: u16) -> Vec<u8> {
    let suffix = if matches!(kind, MouseEventKind::Up(_)) {
        'm'
    } else {
        'M'
    };
    format!("\x1b[<{button};{column};{row}{suffix}").into_bytes()
}
