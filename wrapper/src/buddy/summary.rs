use serde::Serialize;

use super::events::{BuddyEvent, BuddyEventKind};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RollingSummary {
    pub current_task: Option<String>,
    pub last_status: Option<String>,
    pub notable_files: Vec<String>,
    pub unresolved_issue: Option<String>,
}

const FILE_EXTENSIONS: &[&str] = &[
    ".c", ".cc", ".cpp", ".css", ".go", ".h", ".html", ".java", ".js", ".json", ".jsx", ".md",
    ".py", ".rb", ".rs", ".sh", ".sql", ".toml", ".ts", ".tsx", ".txt", ".yaml", ".yml",
];

impl RollingSummary {
    pub fn apply(&mut self, event: &BuddyEvent) {
        self.extend_notable_files(event.user_excerpt.as_deref());
        self.extend_notable_files(event.assistant_excerpt.as_deref());
        self.extend_notable_files(event.tool_command.as_deref());

        match event.kind {
            BuddyEventKind::UserTurnSubmitted => {
                self.current_task = event.user_excerpt.clone();
            }
            BuddyEventKind::ToolStarted => {
                self.last_status = Some(match &event.tool_name {
                    Some(tool_name) => format!("tool {tool_name} running"),
                    None => "tool running".to_string(),
                });
            }
            BuddyEventKind::ToolFinished => {
                self.last_status = Some(match (&event.tool_name, event.tool_success) {
                    (Some(tool_name), Some(true)) => format!("tool {tool_name} succeeded"),
                    (Some(tool_name), Some(false)) => format!("tool {tool_name} failed"),
                    (Some(tool_name), None) => format!("tool {tool_name} finished"),
                    (None, Some(true)) => "tool succeeded".to_string(),
                    (None, Some(false)) => "tool failed".to_string(),
                    (None, None) => "tool finished".to_string(),
                });

                match event.tool_success {
                    Some(false) => {
                        self.unresolved_issue = event
                            .assistant_excerpt
                            .clone()
                            .or_else(|| event.tool_command.clone());
                    }
                    Some(true)
                        if event
                            .assistant_excerpt
                            .as_deref()
                            .is_some_and(looks_resolved) =>
                    {
                        self.unresolved_issue = None;
                    }
                    _ => {}
                }
            }
            BuddyEventKind::TurnCompleted => {
                if let Some(excerpt) = event.assistant_excerpt.as_deref() {
                    if looks_resolved(excerpt) || event.tool_success == Some(true) {
                        self.unresolved_issue = None;
                    } else if looks_unresolved(excerpt) {
                        self.unresolved_issue = Some(excerpt.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    fn extend_notable_files(&mut self, raw: Option<&str>) {
        let Some(raw) = raw else {
            return;
        };

        for token in raw.split_whitespace() {
            let candidate = token.trim_matches(|ch: char| {
                !(ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
            });

            if looks_like_path(candidate)
                && !self
                    .notable_files
                    .iter()
                    .any(|existing| existing == candidate)
            {
                self.notable_files.push(candidate.to_string());
                if self.notable_files.len() > 8 {
                    self.notable_files.remove(0);
                }
            }
        }
    }
}

fn looks_like_path(candidate: &str) -> bool {
    !candidate.is_empty()
        && FILE_EXTENSIONS
            .iter()
            .any(|extension| candidate.ends_with(extension))
        && (candidate.contains('/') || candidate.contains('.'))
}

fn looks_resolved(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    [
        "fixed",
        "resolved",
        "passed",
        "succeeded",
        "green",
        "working",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn looks_unresolved(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    [
        "fail",
        "failed",
        "error",
        "panic",
        "stuck",
        "blocked",
        "unable",
        "could not",
        "can't",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}
