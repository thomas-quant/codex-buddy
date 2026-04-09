use serde_json::Value;

pub fn parse_hook_payload(bytes: &[u8]) -> anyhow::Result<Value> {
    Ok(serde_json::from_slice(bytes)?)
}

pub fn render_hooks_json(wrapper_exe: &str, socket_path: &str) -> String {
    let _ = (wrapper_exe, socket_path);
    r#"{
  "hooks": {}
}"#
    .to_string()
}
