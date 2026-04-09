use std::{fs, os::unix::fs::PermissionsExt};

use buddy_wrapper::codex::exec::build_hatch_command;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn hatch_exec_command_suppresses_child_terminal_output() {
    let dir = tempdir().unwrap();
    let codex_path = dir.path().join("codex");
    fs::write(
        &codex_path,
        r#"#!/bin/sh
last=""
for arg in "$@"; do
  last="$arg"
done
printf 'STDOUT:%s\n' "$last"
printf 'STDERR:%s\n' "$last" >&2
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&codex_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&codex_path, perms).unwrap();

    let schema_path = dir.path().join("schema.json");
    let output_path = dir.path().join("out.json");
    fs::write(&schema_path, "{}").unwrap();

    let mut command = build_hatch_command(
        "secret hatch prompt",
        &schema_path,
        &output_path,
        dir.path(),
    );
    command.env("PATH", dir.path());

    let output = command.output().unwrap();
    assert!(output.status.success());
    assert!(
        output.stdout.is_empty(),
        "expected no stdout leak, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "expected no stderr leak, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn hatch_exec_command_disables_repo_skills_for_sidecar_runs() {
    let dir = tempdir().unwrap();
    let skill_dir = dir.path().join(".agents/skills/release-helper");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "name: release\ndescription: test\n",
    )
    .unwrap();

    let schema_path = dir.path().join("schema.json");
    let output_path = dir.path().join("out.json");
    fs::write(&schema_path, "{}").unwrap();

    let command = build_hatch_command(
        "secret hatch prompt",
        &schema_path,
        &output_path,
        dir.path(),
    );
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert!(
        args.iter().any(|arg| arg.starts_with("skills.config=")),
        "expected a per-skill disable override, got args: {args:?}"
    );
    assert!(
        args.iter().any(|arg| arg.contains("release-helper")),
        "expected repo skill path in args: {args:?}"
    );
}

#[test]
fn quip_schema_requires_all_defined_properties_for_codex_exec() {
    let schema: Value = serde_json::from_str(include_str!("../schemas/quip.schema.json")).unwrap();
    let properties = schema.get("properties").and_then(Value::as_object).unwrap();
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();

    for key in properties.keys() {
        assert!(
            required.contains(key.as_str()),
            "missing required entry for property {key}"
        );
    }

    assert_eq!(
        schema["properties"]["text"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert_eq!(
        schema["properties"]["tone"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert!(
        schema["properties"]["tone"]["enum"]
            .as_array()
            .unwrap()
            .contains(&Value::Null)
    );
}
