use buddy_wrapper::buddy::lifecycle::{apply_pet, hatch_fallback};

#[test]
fn hatch_fallback_produces_name_and_personality() {
    let soul = hatch_fallback("seed-123", "rare", "duck");
    assert!(!soul.name.is_empty());
    assert!(soul.personality_paragraph.len() > 20);
}

#[test]
fn pet_action_sets_timestamp() {
    let ts = apply_pet(1_700_000_000_000);
    assert_eq!(ts, 1_700_000_000_000);
}
