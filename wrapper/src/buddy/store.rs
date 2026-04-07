use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};

use crate::util::paths::StoragePaths;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersistedBuddy {
    pub hatch_seed: String,
    pub name: String,
    pub personality_paragraph: String,
    pub hatched_at: DateTime<Utc>,
    pub last_rebirth_at: Option<DateTime<Utc>>,
    pub muted: bool,
}

impl PersistedBuddy {
    pub fn new_for_test(seed: &str, name: &str, personality: &str) -> Self {
        Self {
            hatch_seed: seed.to_string(),
            name: name.to_string(),
            personality_paragraph: personality.to_string(),
            hatched_at: Utc::now(),
            last_rebirth_at: None,
            muted: false,
        }
    }
}

pub struct BuddyStore {
    paths: StoragePaths,
}

impl BuddyStore {
    pub fn new(paths: StoragePaths) -> Result<Self> {
        fs::create_dir_all(&paths.state_dir)?;
        Ok(Self { paths })
    }

    pub fn load_global(&self) -> Result<Option<PersistedBuddy>> {
        if !self.paths.global_buddy_file.exists() {
            return Ok(None);
        }

        let file = File::open(&self.paths.global_buddy_file)?;
        file.lock_shared()?;
        let parsed = serde_json::from_reader(BufReader::new(&file))?;
        file.unlock()?;
        Ok(Some(parsed))
    }

    pub fn save_global(&self, buddy: &PersistedBuddy) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.paths.global_buddy_file)?;
        file.lock_exclusive()?;

        {
            let mut writer = BufWriter::new(&file);
            serde_json::to_writer_pretty(&mut writer, buddy)?;
            writer.flush()?;
        }

        file.sync_all()?;
        file.unlock()?;
        Ok(())
    }
}
