use buddy_wrapper::buddy::events::{BuddyEventKind, normalize_hook_event};
use serde_json::json;

#[test]
fn user_prompt_submit_normalizes_to_user_turn_submitted() {
    let raw = json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": "s1",
        "turn_id": "t1",
        "cwd": "/tmp/project",
        "prompt": "fix the failing test"
    });
    let normalized = normalize_hook_event(&raw).unwrap();
    assert_eq!(normalized.kind, BuddyEventKind::UserTurnSubmitted);
}

#[test]
fn post_tool_use_normalizes_tool_name_and_result() {
    let raw = json!({
        "hook_event_name": "PostToolUse",
        "session_id": "s1",
        "turn_id": "t1",
        "cwd": "/tmp/project",
        "tool_name": "Bash",
        "tool_input": { "command": "cargo test" },
        "tool_response": "{\"exit_code\":1}"
    });
    let normalized = normalize_hook_event(&raw).unwrap();
    assert_eq!(normalized.kind, BuddyEventKind::ToolFinished);
    assert_eq!(normalized.tool_name.as_deref(), Some("Bash"));
}
