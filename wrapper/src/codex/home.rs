use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

pub struct CodexHomeOverlay {
    pub root: PathBuf,
    pub config_toml: PathBuf,
    pub hooks_json: PathBuf,
}

pub fn build_codex_home_overlay(
    root: &Path,
    wrapper_exe: &str,
    socket_path: &str,
) -> Result<CodexHomeOverlay> {
    fs::create_dir_all(root)?;

    let config_toml = root.join("config.toml");
    let hooks_json = root.join("hooks.json");

    fs::write(
        &config_toml,
        r#"[features]
codex_hooks = true

[history]
persistence = "none"
"#,
    )?;
    fs::write(
        &hooks_json,
        crate::codex::hooks::render_hooks_json(wrapper_exe, socket_path),
    )?;

    Ok(CodexHomeOverlay {
        root: root.to_path_buf(),
        config_toml,
        hooks_json,
    })
}
