use tempfile::tempdir;

use buddy_wrapper::codex::home::build_codex_home_overlay;

#[test]
fn overlay_writes_config_and_hooks_json() {
    let dir = tempdir().unwrap();
    let overlay =
        build_codex_home_overlay(dir.path(), "/tmp/buddy-wrapper", "/tmp/buddy.sock").unwrap();
    assert!(overlay.config_toml.exists());
    assert!(overlay.hooks_json.exists());
}
