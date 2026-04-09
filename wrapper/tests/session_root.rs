use std::{env, path::Path};

use buddy_wrapper::util::paths::{StoragePaths, resolve_codex_session_root};

#[test]
fn codex_session_root_uses_state_dir_when_it_is_not_temporary() {
    let storage_paths = StoragePaths::for_test(Path::new("/home/alice/.local/state/buddy-wrapper"));

    let session_root =
        resolve_codex_session_root(&storage_paths, Path::new("/home/alice/.codex")).unwrap();

    assert_eq!(
        session_root,
        Path::new("/home/alice/.local/state/buddy-wrapper/sessions")
    );
}

#[test]
fn codex_session_root_falls_back_when_state_dir_is_temporary() {
    let storage_paths = StoragePaths::for_test(&env::temp_dir().join("buddy-wrapper-state"));

    let session_root =
        resolve_codex_session_root(&storage_paths, Path::new("/home/alice/.codex")).unwrap();

    assert_eq!(
        session_root,
        Path::new("/home/alice/.codex/buddy-wrapper/sessions")
    );
}

#[test]
fn codex_session_root_rejects_temporary_fallbacks() {
    let storage_paths = StoragePaths::for_test(&env::temp_dir().join("buddy-wrapper-state"));
    let base_codex_home = env::temp_dir().join("codex-home");

    let error = resolve_codex_session_root(&storage_paths, &base_codex_home).unwrap_err();

    assert!(error.to_string().contains("temporary"));
}
