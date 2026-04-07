use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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
