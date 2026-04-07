use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuipPolicyConfig {
    pub cooldown: Duration,
    pub long_run_threshold: Duration,
}

impl Default for QuipPolicyConfig {
    fn default() -> Self {
        Self {
            cooldown: Duration::minutes(10),
            long_run_threshold: Duration::minutes(20),
        }
    }
}

pub fn can_attempt_long_run_quip(
    started_at: DateTime<Utc>,
    now: DateTime<Utc>,
    already_fired: bool,
    cfg: &QuipPolicyConfig,
) -> bool {
    !already_fired && now >= started_at + cfg.long_run_threshold
}
