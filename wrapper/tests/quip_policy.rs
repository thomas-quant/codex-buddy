use chrono::{Duration, Utc};

use buddy_wrapper::buddy::policy::{QuipPolicyConfig, can_attempt_long_run_quip};
use buddy_wrapper::buddy::quips::sanitize_quip;

#[test]
fn quip_text_is_single_line_and_capped() {
    let text = sanitize_quip("hello\nworld ".repeat(20).as_str()).unwrap();
    assert!(!text.contains('\n'));
    assert!(text.chars().count() <= 80);
}

#[test]
fn long_run_quip_requires_twenty_minutes_and_only_fires_once() {
    let cfg = QuipPolicyConfig::default();
    let started = Utc::now();
    assert!(!can_attempt_long_run_quip(
        started,
        started + Duration::minutes(19),
        false,
        &cfg,
    ));
    assert!(can_attempt_long_run_quip(
        started,
        started + Duration::minutes(20),
        false,
        &cfg,
    ));
    assert!(!can_attempt_long_run_quip(
        started,
        started + Duration::minutes(21),
        true,
        &cfg,
    ));
}
