use std::time::Duration;

use buddy_wrapper::codex::pty::PtyHost;

#[test]
fn pty_host_captures_screen_output() {
    let mut host = PtyHost::spawn_for_test("/bin/sh", &["-lc", "printf 'hello'"], 80, 24).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    host.pump_output().unwrap();
    assert!(host.screen_text().contains("hello"));
}
