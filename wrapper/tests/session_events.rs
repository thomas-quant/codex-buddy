use buddy_wrapper::{
    buddy::events::BuddyEventKind,
    codex::session::{SessionEvent, SessionEventNormalizer},
};

#[test]
fn session_normalizer_maps_user_messages_and_turn_completion() {
    let mut normalizer = SessionEventNormalizer::default();

    let started = normalizer
        .push_line(
            r#"{"type":"session_meta","payload":{"id":"session-1","cwd":"/root/codex-buddy"}}"#,
        )
        .unwrap();
    assert!(matches!(
        &started[0],
        SessionEvent::Buddy(event) if event.kind == BuddyEventKind::SessionStarted
    ));

    let submitted = normalizer
        .push_line(r#"{"type":"event_msg","payload":{"type":"user_message","message":"hi there"}}"#)
        .unwrap();
    assert!(matches!(
        &submitted[0],
        SessionEvent::Buddy(event)
            if event.kind == BuddyEventKind::UserTurnSubmitted
                && event.user_excerpt.as_deref() == Some("hi there")
    ));

    let completed = normalizer
        .push_line(
            r#"{"type":"event_msg","payload":{"type":"task_complete","turn_id":"turn-1","last_agent_message":"done"}}"#,
        )
        .unwrap();
    assert!(matches!(
        &completed[0],
        SessionEvent::Buddy(event)
            if event.kind == BuddyEventKind::TurnCompleted
                && event.turn_id.as_deref() == Some("turn-1")
                && event.assistant_excerpt.as_deref() == Some("done")
    ));
}

#[test]
fn session_normalizer_tracks_tool_calls_without_hooks() {
    let mut normalizer = SessionEventNormalizer::default();
    normalizer
        .push_line(
            r#"{"type":"session_meta","payload":{"id":"session-1","cwd":"/root/codex-buddy"}}"#,
        )
        .unwrap();

    let started = normalizer
        .push_line(
            r#"{"type":"response_item","payload":{"type":"function_call","name":"exec_command","call_id":"call-1","arguments":"{\"cmd\":\"cargo test\"}"}}"#,
        )
        .unwrap();
    assert!(matches!(
        &started[0],
        SessionEvent::Buddy(event)
            if event.kind == BuddyEventKind::ToolStarted
                && event.tool_name.as_deref() == Some("exec_command")
                && event.tool_command.as_deref() == Some("cargo test")
    ));

    let finished = normalizer
        .push_line(
            r#"{"type":"event_msg","payload":{"type":"exec_command_end","call_id":"call-1","turn_id":"turn-1","exit_code":0}}"#,
        )
        .unwrap();
    assert!(matches!(
        &finished[0],
        SessionEvent::Buddy(event)
            if event.kind == BuddyEventKind::ToolFinished
                && event.turn_id.as_deref() == Some("turn-1")
                && event.tool_success == Some(true)
    ));
}

#[test]
fn session_normalizer_emits_commentary_for_ui_filtering() {
    let mut normalizer = SessionEventNormalizer::default();
    normalizer
        .push_line(
            r#"{"type":"session_meta","payload":{"id":"session-1","cwd":"/root/codex-buddy"}}"#,
        )
        .unwrap();

    let events = normalizer
        .push_line(
            r#"{"type":"event_msg","payload":{"type":"agent_message","phase":"commentary","message":"I’m checking the workspace guidance first"}}"#,
        )
        .unwrap();

    assert_eq!(
        events,
        vec![SessionEvent::Commentary(
            "I’m checking the workspace guidance first".to_string()
        )]
    );
}
