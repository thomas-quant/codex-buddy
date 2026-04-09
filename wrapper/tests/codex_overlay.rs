use std::fs;

use serde_json::Value as JsonValue;
use tempfile::tempdir;
use toml::Value;

use buddy_wrapper::codex::home::build_codex_home_overlay;

#[test]
fn overlay_inherits_user_config_and_auth() {
    let base_home = tempdir().unwrap();
    fs::write(
        base_home.path().join("config.toml"),
        "model = \"gpt-5.4\"\n[projects.\"/root/codex-buddy\"]\ntrust_level = \"trusted\"\n[notice]\nhide_rate_limit_model_nudge = true\n",
    )
    .unwrap();
    fs::write(
        base_home.path().join("auth.json"),
        "{\"access_token\":\"token\"}",
    )
    .unwrap();
    fs::create_dir(base_home.path().join("skills")).unwrap();

    let dir = tempdir().unwrap();
    let overlay = build_codex_home_overlay(
        base_home.path(),
        dir.path(),
        "/tmp/buddy-wrapper",
        "/tmp/buddy.sock",
    )
    .unwrap();

    assert!(overlay.config_toml.exists());
    assert!(overlay.hooks_json.exists());
    assert!(dir.path().join("auth.json").exists());
    assert!(dir.path().join("skills").exists());

    let config: Value = fs::read_to_string(overlay.config_toml)
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(config.get("model").and_then(Value::as_str), Some("gpt-5.4"));
    assert_eq!(
        config
            .get("projects")
            .and_then(|projects| projects.get("/root/codex-buddy"))
            .and_then(|project| project.get("trust_level"))
            .and_then(Value::as_str),
        Some("trusted")
    );
    assert_eq!(
        config.get("hide_agent_reasoning").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        config
            .get("notice")
            .and_then(|notice| notice.get("hide_rate_limit_model_nudge"))
            .and_then(Value::as_bool),
        Some(true)
    );

    let hooks: JsonValue =
        serde_json::from_str(&fs::read_to_string(overlay.hooks_json).unwrap()).unwrap();
    assert_eq!(hooks, serde_json::json!({ "hooks": {} }));
}
