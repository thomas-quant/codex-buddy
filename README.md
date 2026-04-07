# codex-buddy

Clean-room Buddy research plus a working Rust wrapper TUI for Codex.

## What Is Here

- [`buddy/`](/root/codex-buddy/buddy)
  Clean-room Buddy behavior notes, extracted snapshot files, and wrapper-port specs.
- [`docs/superpowers/plans/`](/root/codex-buddy/docs/superpowers/plans)
  Planning notes for the missing host behavior and the wrapper implementation.
- [`wrapper/`](/root/codex-buddy/wrapper)
  A Rust `ratatui` wrapper that hosts stock `codex` in a PTY and renders an always-visible Buddy side pane.

## Wrapper Highlights

- PTY-hosted Codex main pane
- Always-visible Buddy side pane
- Focus switching between Codex and Buddy
- Buddy action menu with `hatch`, `status`, `pet`, `mute`, `unmute`, and `rebirth`
- Wrapper-owned Buddy persistence
- Codex hook relay and normalized Buddy events
- `codex exec` hatch and quip backends

## Run

Prerequisites:

- Rust toolchain
- `codex` on `PATH`

Launch the wrapper:

```bash
cargo run --manifest-path wrapper/Cargo.toml
```

Controls:

- `Tab`: switch focus between Codex and Buddy
- `Enter`: open or activate the Buddy action menu when Buddy is focused
- `Up` / `Down` or `k` / `j`: move in the Buddy action menu
- `Esc`: close Buddy menu or status view
- `Ctrl+Q`: quit the wrapper

## Verify

```bash
cargo test --manifest-path wrapper/Cargo.toml
cargo fmt --manifest-path wrapper/Cargo.toml --check
cargo clippy --manifest-path wrapper/Cargo.toml --all-targets -- -D warnings
```

## Notes

- Codex hooks are currently Unix-first; Windows is not the target for this wrapper.
- `session_ended` is synthesized by the wrapper when the PTY exits because Codex does not expose a documented `SessionEnd` hook.
