use std::{collections::BTreeMap, path::Path};

pub struct CodexLaunch {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

pub fn build_codex_launch(cwd: &Path, codex_home: &Path) -> CodexLaunch {
    let mut env = BTreeMap::new();
    env.insert("CODEX_HOME".into(), codex_home.display().to_string());

    CodexLaunch {
        command: "codex".into(),
        args: vec![
            "--enable".into(),
            "codex_hooks".into(),
            "--cd".into(),
            cwd.display().to_string(),
        ],
        env,
    }
}
