use std::time::Duration;

use buddy_wrapper::codex::pty::PtyHost;

#[test]
fn pty_host_captures_screen_output() {
    let mut host = PtyHost::spawn_for_test("/bin/sh", &["-lc", "printf 'hello'"], 80, 24).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    host.pump_output().unwrap();
    assert!(host.screen_text().contains("hello"));
}

#[test]
fn pty_host_retains_scrollback_for_inline_sessions() {
    let script = "for n in $(seq 1 12); do printf 'line%02d\\n' \"$n\"; done";
    let mut host = PtyHost::spawn_for_test("/bin/sh", &["-lc", script], 12, 4).unwrap();

    std::thread::sleep(Duration::from_millis(200));
    host.pump_output().unwrap();

    assert!(host.screen_text().contains("line12"));
    host.scroll_up(3);
    assert!(host.screen_text().contains("line09"));
    assert!(!host.screen_text().contains("line12"));

    host.scroll_to_bottom();
    assert!(host.screen_text().contains("line12"));
}
