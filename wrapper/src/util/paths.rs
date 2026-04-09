use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePaths {
    pub state_dir: PathBuf,
    pub global_buddy_file: PathBuf,
}

impl StoragePaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("dev", "openai", "buddy-wrapper")
            .context("unable to resolve project directories")?;
        let state_dir = dirs.state_dir().unwrap_or(dirs.data_dir()).to_path_buf();
        Ok(Self {
            global_buddy_file: state_dir.join("buddy-state.json"),
            state_dir,
        })
    }

    pub fn for_test(root: &Path) -> Self {
        Self {
            state_dir: root.to_path_buf(),
            global_buddy_file: root.join("buddy-state.json"),
        }
    }
}

pub fn resolve_codex_session_root(
    storage_paths: &StoragePaths,
    base_codex_home: &Path,
) -> Result<PathBuf> {
    let primary_root = storage_paths.state_dir.join("sessions");
    if !is_temporary_path(&primary_root) {
        return Ok(primary_root);
    }

    let fallback_root = base_codex_home.join("buddy-wrapper").join("sessions");
    if !is_temporary_path(&fallback_root) {
        return Ok(fallback_root);
    }

    bail!(
        "refusing to create Codex session roots under temporary directories (state_dir: {}, CODEX_HOME: {})",
        storage_paths.state_dir.display(),
        base_codex_home.display()
    );
}

fn is_temporary_path(path: &Path) -> bool {
    path.starts_with(env::temp_dir())
}
