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
            "--no-alt-screen".into(),
            "-c".into(),
            "history.persistence=\"none\"".into(),
            "-c".into(),
            "hide_agent_reasoning=true".into(),
            "-c".into(),
            "model_reasoning_summary=\"none\"".into(),
            "--cd".into(),
            cwd.display().to_string(),
        ],
        env,
    }
}
