use serde_json::Value;

pub fn parse_hook_payload(bytes: &[u8]) -> anyhow::Result<Value> {
    Ok(serde_json::from_slice(bytes)?)
}

pub fn render_hooks_json(wrapper_exe: &str, socket_path: &str) -> String {
    format!(
        r#"{{
  "hooks": {{
    "SessionStart": [{{ "matcher": "startup|resume", "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "UserPromptSubmit": [{{ "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "PreToolUse": [{{ "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "PostToolUse": [{{ "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "Stop": [{{ "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}]
  }}
}}"#
    )
}
