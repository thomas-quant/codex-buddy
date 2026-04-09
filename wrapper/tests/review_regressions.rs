use std::{fs, io::Read};

use buddy_wrapper::{
    buddy::{
        events::{BuddyEvent, BuddyEventKind},
        store::PersistedBuddy,
        summary::RollingSummary,
        types::CompanionBones,
    },
    codex::{hooks::render_hooks_json, relay::relay_hook_payload},
    ui::buddy_pane::render_status_lines,
};
use chrono::{Duration, TimeZone, Utc};
use serde_json::Value;

#[test]
fn hooks_json_is_inert_when_wrapper_uses_session_logs_instead() {
    let raw = render_hooks_json("/tmp/buddy-wrapper", "/tmp/buddy.sock");
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed, serde_json::json!({ "hooks": {} }));
}

#[test]
fn relay_hook_payload_writes_the_full_message() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("buddy.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
    let payload = "x".repeat(256 * 1024);

    let client_payload = payload.clone();
    let client_socket = socket_path.clone();
    let sender = std::thread::spawn(move || relay_hook_payload(&client_socket, client_payload));

    let (mut stream, _) = listener.accept().unwrap();
    let mut received = String::new();
    stream.read_to_string(&mut received).unwrap();
    sender.join().unwrap().unwrap();

    assert_eq!(received, payload);
}

#[test]
fn rolling_summary_tracks_status_files_and_clears_resolved_issues() {
    let mut summary = RollingSummary::default();

    summary.apply(&BuddyEvent {
        kind: BuddyEventKind::UserTurnSubmitted,
        session_id: "s1".into(),
        turn_id: Some("t1".into()),
        cwd: "/tmp/project".into(),
        tool_name: None,
        tool_command: None,
        tool_success: None,
        assistant_excerpt: None,
        user_excerpt: Some("Fix src/main.rs and wrapper/src/app/mod.rs".into()),
    });

    summary.apply(&BuddyEvent {
        kind: BuddyEventKind::ToolFinished,
        session_id: "s1".into(),
        turn_id: Some("t1".into()),
        cwd: "/tmp/project".into(),
        tool_name: Some("Bash".into()),
        tool_command: Some("cargo test wrapper/src/app/mod.rs".into()),
        tool_success: Some(false),
        assistant_excerpt: Some("Tests failed in wrapper/src/app/mod.rs".into()),
        user_excerpt: None,
    });

    assert_eq!(
        summary.current_task.as_deref(),
        Some("Fix src/main.rs and wrapper/src/app/mod.rs")
    );
    assert_eq!(summary.last_status.as_deref(), Some("tool Bash failed"));
    assert!(
        summary
            .notable_files
            .contains(&"wrapper/src/app/mod.rs".to_string())
    );
    assert_eq!(
        summary.unresolved_issue.as_deref(),
        Some("Tests failed in wrapper/src/app/mod.rs")
    );

    summary.apply(&BuddyEvent {
        kind: BuddyEventKind::TurnCompleted,
        session_id: "s1".into(),
        turn_id: Some("t1".into()),
        cwd: "/tmp/project".into(),
        tool_name: None,
        tool_command: None,
        tool_success: Some(true),
        assistant_excerpt: Some("Fixed wrapper/src/app/mod.rs and all tests passed.".into()),
        user_excerpt: None,
    });

    assert_eq!(summary.unresolved_issue, None);
    assert!(summary.notable_files.contains(&"src/main.rs".to_string()));
}

#[test]
fn status_view_shows_hatch_age_and_rebirth_availability() {
    let buddy = PersistedBuddy {
        hatch_seed: "seed".into(),
        name: "Mochi".into(),
        personality_paragraph: "An observant little goblin.".into(),
        hatched_at: Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap(),
        last_rebirth_at: Some(Utc.with_ymd_and_hms(2026, 3, 24, 12, 0, 0).unwrap()),
        muted: false,
    };

    let now = Utc.with_ymd_and_hms(2026, 4, 7, 12, 0, 0).unwrap();
    let lines = render_status_lines(&buddy, &CompanionBones::test_fixture(), 0, now);

    assert!(
        lines
            .iter()
            .any(|line| line.contains("Hatched: 2026-03-01"))
    );
    assert!(lines.iter().any(|line| line.contains("Age: 37 days")));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Rebirth: available now"))
    );
}

#[test]
fn status_view_shows_rebirth_cooldown_when_not_ready() {
    let buddy = PersistedBuddy {
        hatch_seed: "seed".into(),
        name: "Mochi".into(),
        personality_paragraph: "An observant little goblin.".into(),
        hatched_at: Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap(),
        last_rebirth_at: None,
        muted: false,
    };

    let now = buddy.hatched_at + Duration::days(5);
    let lines = render_status_lines(&buddy, &CompanionBones::test_fixture(), 0, now);

    assert!(
        lines
            .iter()
            .any(|line| line.contains("Rebirth: available in 9 days"))
    );
}

#[test]
fn pty_can_receive_input_bytes_after_spawn() {
    let dir = tempfile::tempdir().unwrap();
    let capture = dir.path().join("capture.txt");
    let script = format!("read line; printf '%s' \"$line\" > {}", capture.display());

    let mut host =
        buddy_wrapper::codex::pty::PtyHost::spawn_for_test("/bin/sh", &["-lc", &script], 80, 24)
            .unwrap();

    host.write_all(b"hello from wrapper\r").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));

    assert_eq!(fs::read_to_string(capture).unwrap(), "hello from wrapper");
}
