use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::buddy::{lifecycle::HatchSoul, types::CompanionBones};

const DEFAULT_MODEL: &str = "gpt-5.4-mini";
const DEFAULT_REASONING_EFFORT: &str = "medium";
const HATCH_PROMPT: &str = include_str!("../../prompts/hatch.md");
const QUIP_PROMPT: &str = include_str!("../../prompts/quip.md");
const HATCH_SCHEMA: &str = include_str!("../../schemas/hatch.schema.json");
const QUIP_SCHEMA: &str = include_str!("../../schemas/quip.schema.json");

#[derive(Debug, Clone, Serialize)]
pub struct QuipRequest {
    pub buddy_name: String,
    pub personality_paragraph: String,
    pub event_type: String,
    pub cwd: String,
    pub rolling_summary: serde_json::Value,
    pub recent_turn_digest: serde_json::Value,
    pub raw_excerpts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuipResponse {
    pub emit: bool,
    pub text: Option<String>,
    pub tone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct HatchResponse {
    name: String,
    personality_paragraph: String,
}

pub fn build_hatch_command(
    prompt: &str,
    schema_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    cwd: &Path,
) -> Command {
    build_exec_command(prompt, schema_path.as_ref(), output_path.as_ref(), cwd)
}

pub fn generate_hatch_soul(cwd: &Path, seed: &str, bones: &CompanionBones) -> Result<HatchSoul> {
    let prompt = format!(
        "{HATCH_PROMPT}\n\nContext JSON:\n{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "hatch_seed": seed,
            "bones": bones,
        }))?
    );

    let response: HatchResponse = run_structured_exec(&prompt, HATCH_SCHEMA, cwd)
        .context("codex exec hatch generation failed")?;

    Ok(HatchSoul {
        name: response.name,
        personality_paragraph: response.personality_paragraph,
    })
}

pub fn generate_quip(cwd: &Path, request: &QuipRequest) -> Result<QuipResponse> {
    let prompt = format!(
        "{QUIP_PROMPT}\n\nContext JSON:\n{}",
        serde_json::to_string_pretty(request)?
    );

    run_structured_exec(&prompt, QUIP_SCHEMA, cwd).context("codex exec quip generation failed")
}

fn run_structured_exec<T>(prompt: &str, schema: &str, cwd: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let schema_file = write_temp_file(schema, "schema.json")?;
    let output_file = NamedTempFile::new()?;
    let mut command = build_exec_command(prompt, schema_file.path(), output_file.path(), cwd);
    let status = command.status()?;

    if !status.success() {
        return Err(anyhow!("codex exec exited with status {status}"));
    }

    let raw = std::fs::read_to_string(output_file.path())?;
    serde_json::from_str(&raw).context("failed to parse structured codex exec output")
}

fn write_temp_file(contents: &str, suffix: &str) -> Result<NamedTempFile> {
    let file = NamedTempFile::with_suffix(suffix)?;
    std::fs::write(file.path(), contents)?;
    Ok(file)
}

fn build_exec_command(prompt: &str, schema_path: &Path, output_path: &Path, cwd: &Path) -> Command {
    let mut command = Command::new("codex");
    command
        .current_dir(cwd)
        .arg("exec")
        .arg("--ephemeral")
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(cwd)
        .arg("--output-schema")
        .arg(schema_path)
        .arg("-o")
        .arg(output_path)
        .arg("-m")
        .arg(DEFAULT_MODEL)
        .arg("-c")
        .arg(format!(
            "model_reasoning_effort=\"{DEFAULT_REASONING_EFFORT}\""
        ))
        .arg(prompt);
    command
}

#[allow(dead_code)]
fn _workspace_relative(path: &Path) -> PathBuf {
    path.to_path_buf()
}
