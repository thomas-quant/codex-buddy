use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::BaseDirs;
use toml::{Value, map::Map};

#[cfg(unix)]
use std::os::unix::fs::symlink;

pub struct CodexHomeOverlay {
    pub root: PathBuf,
    pub config_toml: PathBuf,
    pub hooks_json: PathBuf,
}

pub fn resolve_base_codex_home() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }

    let base_dirs = BaseDirs::new().context("unable to resolve user home directory")?;
    Ok(base_dirs.home_dir().join(".codex"))
}

pub fn build_codex_home_overlay(
    base_home: &Path,
    root: &Path,
    wrapper_exe: &str,
    socket_path: &str,
) -> Result<CodexHomeOverlay> {
    fs::create_dir_all(root)?;

    let config_toml = root.join("config.toml");
    let hooks_json = root.join("hooks.json");

    mirror_optional_entry(base_home, root, "auth.json")?;
    mirror_optional_entry(base_home, root, "skills")?;
    mirror_optional_entry(base_home, root, "memories")?;

    let base_config = base_home.join("config.toml");
    let mut config = if base_config.exists() {
        fs::read_to_string(&base_config)?
            .parse::<Value>()
            .context("failed to parse Codex config")?
    } else {
        Value::Table(Map::new())
    };
    config_root(&mut config)?.insert("hide_agent_reasoning".into(), Value::Boolean(true));
    fs::write(&config_toml, toml::to_string(&config)?)?;
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

#[cfg(unix)]
fn mirror_optional_entry(base_home: &Path, root: &Path, entry_name: &str) -> Result<()> {
    let source = base_home.join(entry_name);
    if !source.exists() {
        return Ok(());
    }

    let destination = root.join(entry_name);
    symlink(&source, &destination)?;
    Ok(())
}

fn config_root(config: &mut Value) -> Result<&mut Map<String, Value>> {
    config
        .as_table_mut()
        .context("Codex config root must be a TOML table")
}
