use std::path::Path;

use buddy_wrapper::codex::launch::build_codex_launch;

#[test]
fn interactive_launch_uses_inline_mode_and_reasoning_suppression() {
    let launch = build_codex_launch(Path::new("/root/codex-buddy"), Path::new("/tmp/codex-home"));

    assert!(launch.args.iter().any(|arg| arg == "--no-alt-screen"));
    assert!(
        launch
            .args
            .windows(2)
            .any(|pair| pair[0] == "-c" && pair[1] == "hide_agent_reasoning=true")
    );
    assert!(
        launch
            .args
            .windows(2)
            .any(|pair| pair[0] == "-c" && pair[1] == "model_reasoning_summary=\"none\"")
    );
    assert!(!launch.args.iter().any(|arg| arg == "codex_hooks"));
}
