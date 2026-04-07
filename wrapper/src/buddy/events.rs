use anyhow::{Result, anyhow};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum BuddyEventKind {
    SessionStarted,
    UserTurnSubmitted,
    ToolStarted,
    ToolFinished,
    TurnCompleted,
    SessionEnded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuddyEvent {
    pub kind: BuddyEventKind,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub cwd: String,
    pub tool_name: Option<String>,
    pub tool_command: Option<String>,
    pub tool_success: Option<bool>,
    pub assistant_excerpt: Option<String>,
    pub user_excerpt: Option<String>,
}

fn string_field(raw: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| raw.get(key).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn required_string_field(raw: &Value, key: &str) -> Result<String> {
    raw.get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("missing {key}"))
}

fn nested_string_field(raw: &Value, path: &[&str]) -> Option<String> {
    let mut current = raw;
    for segment in path {
        current = current.get(segment)?;
    }
    current.as_str().map(ToString::to_string)
}

fn extract_tool_success(raw: &Value) -> Option<bool> {
    fn from_value(value: &Value) -> Option<bool> {
        value
            .get("exit_code")
            .and_then(Value::as_i64)
            .map(|exit_code| exit_code == 0)
            .or_else(|| value.get("success").and_then(Value::as_bool))
    }

    raw.get("tool_response").and_then(|value| {
        from_value(value).or_else(|| {
            value
                .as_str()
                .and_then(|raw_json| serde_json::from_str::<Value>(raw_json).ok())
                .and_then(|parsed| from_value(&parsed))
        })
    })
}

pub fn normalize_hook_event(raw: &Value) -> Result<BuddyEvent> {
    let event_name = required_string_field(raw, "hook_event_name")?;
    let kind = match event_name.as_str() {
        "SessionStart" => BuddyEventKind::SessionStarted,
        "UserPromptSubmit" => BuddyEventKind::UserTurnSubmitted,
        "PreToolUse" => BuddyEventKind::ToolStarted,
        "PostToolUse" => BuddyEventKind::ToolFinished,
        "Stop" => BuddyEventKind::TurnCompleted,
        "SessionEnd" | "SessionEnded" => BuddyEventKind::SessionEnded,
        other => return Err(anyhow!("unsupported hook event: {other}")),
    };

    Ok(BuddyEvent {
        kind,
        session_id: required_string_field(raw, "session_id")?,
        turn_id: string_field(raw, &["turn_id"]),
        cwd: required_string_field(raw, "cwd")?,
        tool_name: string_field(raw, &["tool_name", "tool"]),
        tool_command: nested_string_field(raw, &["tool_input", "command"])
            .or_else(|| string_field(raw, &["command"])),
        tool_success: extract_tool_success(raw),
        assistant_excerpt: string_field(
            raw,
            &[
                "last_assistant_message",
                "assistant_message",
                "assistant_excerpt",
            ],
        ),
        user_excerpt: string_field(raw, &["prompt", "user_message", "user_excerpt"]),
    })
}
