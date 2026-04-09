use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::Value;

use crate::buddy::events::{BuddyEvent, BuddyEventKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    Buddy(BuddyEvent),
    Commentary(String),
}

#[derive(Debug, Clone)]
struct PendingToolCall {
    tool_name: String,
    tool_command: Option<String>,
}

#[derive(Default)]
pub struct SessionEventNormalizer {
    session_id: Option<String>,
    cwd: Option<String>,
    pending_tools: BTreeMap<String, PendingToolCall>,
}

impl SessionEventNormalizer {
    pub fn push_line(&mut self, line: &str) -> Result<Vec<SessionEvent>> {
        let raw: Value = serde_json::from_str(line)?;
        let payload = raw.get("payload").unwrap_or(&Value::Null);

        match raw.get("type").and_then(Value::as_str) {
            Some("session_meta") => Ok(self.handle_session_meta(payload)),
            Some("event_msg") => Ok(self.handle_event_message(payload)),
            Some("response_item") => Ok(self.handle_response_item(payload)),
            _ => Ok(Vec::new()),
        }
    }

    fn handle_session_meta(&mut self, payload: &Value) -> Vec<SessionEvent> {
        if let Some(session_id) = payload.get("id").and_then(Value::as_str) {
            self.session_id = Some(session_id.to_string());
        }
        if let Some(cwd) = payload.get("cwd").and_then(Value::as_str) {
            self.cwd = Some(cwd.to_string());
        }

        vec![SessionEvent::Buddy(self.buddy_event(
            BuddyEventKind::SessionStarted,
            None,
            None,
            None,
            None,
            None,
        ))]
    }

    fn handle_event_message(&mut self, payload: &Value) -> Vec<SessionEvent> {
        match payload.get("type").and_then(Value::as_str) {
            Some("user_message") => vec![SessionEvent::Buddy(
                self.buddy_event(
                    BuddyEventKind::UserTurnSubmitted,
                    None,
                    None,
                    None,
                    None,
                    payload
                        .get("message")
                        .and_then(Value::as_str)
                        .map(ToString::to_string),
                ),
            )],
            Some("agent_message")
                if payload.get("phase").and_then(Value::as_str) == Some("commentary") =>
            {
                payload
                    .get("message")
                    .and_then(Value::as_str)
                    .map(|message| vec![SessionEvent::Commentary(message.to_string())])
                    .unwrap_or_default()
            }
            Some("task_complete") => vec![SessionEvent::Buddy(
                self.buddy_event(
                    BuddyEventKind::TurnCompleted,
                    payload.get("turn_id").and_then(Value::as_str),
                    None,
                    None,
                    None,
                    payload
                        .get("last_agent_message")
                        .and_then(Value::as_str)
                        .map(ToString::to_string),
                ),
            )],
            Some("exec_command_end") => payload
                .get("call_id")
                .and_then(Value::as_str)
                .and_then(|call_id| self.pending_tools.remove(call_id))
                .map(|pending| {
                    vec![SessionEvent::Buddy(
                        self.buddy_event(
                            BuddyEventKind::ToolFinished,
                            payload.get("turn_id").and_then(Value::as_str),
                            Some(pending.tool_name),
                            pending.tool_command,
                            payload
                                .get("exit_code")
                                .and_then(Value::as_i64)
                                .map(|code| code == 0),
                            None,
                        ),
                    )]
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn handle_response_item(&mut self, payload: &Value) -> Vec<SessionEvent> {
        match payload.get("type").and_then(Value::as_str) {
            Some("function_call") => {
                let tool_name = payload
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let tool_command = payload
                    .get("arguments")
                    .and_then(Value::as_str)
                    .and_then(extract_tool_command);

                if let Some(call_id) = payload.get("call_id").and_then(Value::as_str) {
                    self.pending_tools.insert(
                        call_id.to_string(),
                        PendingToolCall {
                            tool_name: tool_name.clone(),
                            tool_command: tool_command.clone(),
                        },
                    );
                }

                vec![SessionEvent::Buddy(self.buddy_event(
                    BuddyEventKind::ToolStarted,
                    None,
                    Some(tool_name),
                    tool_command,
                    None,
                    None,
                ))]
            }
            Some("function_call_output") => payload
                .get("call_id")
                .and_then(Value::as_str)
                .and_then(|call_id| self.pending_tools.remove(call_id))
                .map(|pending| {
                    vec![SessionEvent::Buddy(self.buddy_event(
                        BuddyEventKind::ToolFinished,
                        None,
                        Some(pending.tool_name),
                        pending.tool_command,
                        None,
                        None,
                    ))]
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn buddy_event(
        &self,
        kind: BuddyEventKind,
        turn_id: Option<&str>,
        tool_name: Option<String>,
        tool_command: Option<String>,
        tool_success: Option<bool>,
        excerpt: Option<String>,
    ) -> BuddyEvent {
        BuddyEvent {
            kind: kind.clone(),
            session_id: self
                .session_id
                .clone()
                .unwrap_or_else(|| "pending-session".to_string()),
            turn_id: turn_id.map(ToString::to_string),
            cwd: self.cwd.clone().unwrap_or_default(),
            tool_name,
            tool_command,
            tool_success,
            assistant_excerpt: if kind == BuddyEventKind::TurnCompleted {
                excerpt.clone()
            } else {
                None
            },
            user_excerpt: if kind == BuddyEventKind::UserTurnSubmitted {
                excerpt
            } else {
                None
            },
        }
    }
}

fn extract_tool_command(arguments: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(arguments).ok()?;
    parsed
        .get("cmd")
        .or_else(|| parsed.get("command"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}
