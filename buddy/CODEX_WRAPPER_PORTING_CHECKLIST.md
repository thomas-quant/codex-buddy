# Buddy Codex Wrapper Porting Checklist

## Purpose

This checklist turns the layered Buddy docs into a concrete port sequence for a
wrapper TUI that hosts stock Codex in a PTY.

Use it with:

1. [CLEAN_ROOM_SPEC.md](./CLEAN_ROOM_SPEC.md)
2. [HOST_INTEGRATION_SPEC.md](./HOST_INTEGRATION_SPEC.md)
3. [CODEX_WRAPPER_PORT_SPEC.md](./CODEX_WRAPPER_PORT_SPEC.md)

This checklist is ordered to minimize rework and keep Buddy additive rather
than invasive.

## Read Order

Before porting, confirm these boundaries:

- `CLEAN_ROOM_SPEC.md`
  Snapshot-derived Buddy behavior: deterministic body generation, rendering,
  teaser behavior, prompt semantics, and timing.
- `HOST_INTEGRATION_SPEC.md`
  Missing Buddy semantics: hatch, status, pet, mute, unmute, and reaction
  observer behavior.
- `CODEX_WRAPPER_PORT_SPEC.md`
  Wrapper-specific host mapping: PTY host, side pane, normalized events,
  wrapper-owned persistence, `codex exec` quips, and multi-session behavior.

## Port Sequence

### 1. Build the wrapper shell

Deliver:

- a wrapper TUI with:
  - a main PTY pane for Codex
  - an always-visible Buddy side pane
- focus switching between:
  - `pty`
  - `buddy_pane`

Verify:

- Codex remains usable when Buddy is inert
- the Buddy pane remains visible without stealing PTY input

### 2. Implement wrapper-owned Buddy storage

Deliver:

- persistent global Buddy state for:
  - `hatch_seed`
  - `name`
  - `personality_paragraph`
  - `hatchedAt`
  - `lastRebirthAt`
  - `muted`
- volatile session-local Buddy state for:
  - rolling session summary
  - recent turns
  - active bubble
  - pet burst timing
  - focus/menu state

Verify:

- Buddy identity survives wrapper restarts
- session-local state disappears when a session ends

### 3. Recreate deterministic body generation from `hatch_seed`

Deliver:

- deterministic regeneration of:
  - rarity
  - species
  - eye
  - hat
  - shiny
  - stats

Verify against `CLEAN_ROOM_SPEC.md`:

- rarity weights
- stat floors and roll rules
- merge semantics between deterministic body and persisted soul

### 4. Render the Buddy pane baseline

Deliver:

- idle sprite rendering in the side pane
- compact name display
- focused and unfocused visual states
- pet-heart burst support
- speech-bubble support

Verify against `CLEAN_ROOM_SPEC.md`:

- narrow/full visual rules adapted into the side-pane host layout
- bubble lifetime and fade timing
- pet burst timing
- muted gating

### 5. Implement the Buddy pane controller

Deliver:

- Buddy-pane focus
- a keyboard-navigable action menu
- a visible hint footnote for opening the menu while the pane is focused

Required actions:

- `hatch`
- `status`
- `pet`
- `mute`
- `unmute`
- `rebirth`

Verify:

- PTY keystrokes do not leak into the Buddy pane when focused
- Buddy-pane keystrokes do not leak into Codex

### 6. Implement hatch and rebirth

Deliver:

- first-birth flow using:
  - new `hatch_seed`
  - deterministic bones
  - one-shot soul generation
- rebirth flow using:
  - full reset
  - new `hatch_seed`
  - 14-day cooldown

Rules to preserve:

- hatch is seed-and-bones driven only
- no repo/cwd/conversation/account context at birth
- full personality is persisted as `personality_paragraph`
- fallback is deterministic and silent

Verify:

- first hatch persists global Buddy state
- rebirth stays disabled during cooldown
- rebirth replaces both body and soul identity

### 7. Implement the in-pane status view

Deliver:

- a temporary detail state inside the Buddy pane

Status must show:

- `name`
- visible deterministic traits worth exposing
- hatch date or age
- rebirth availability/cooldown
- full `personality_paragraph`

Verify:

- personality text appears only in `status`
- dismissing status returns to the normal Buddy pane

### 8. Launch Codex with wrapper-owned hooks

Deliver:

- per-session Codex launch with wrapper-provided hooks enabled
- a hook adapter that turns hook invocations into wrapper-readable inputs

Verify:

- Codex still runs if Buddy-specific consumers are disabled
- hook failures do not crash the PTY host

### 9. Normalize Codex hook data into Buddy events

Deliver normalized events:

- `session_started`
- `user_turn_submitted`
- `tool_started`
- `tool_finished`
- `turn_completed`
- `session_ended`

Verify:

- Buddy logic depends on normalized events, not raw hook payload shape
- `session_id` and `turn_id` are tracked when available

### 10. Implement the rolling session summary

Deliver:

- per-session ephemeral summary
- update points after:
  - `turn_completed`
  - notable `tool_finished`
  - obvious user-intent shifts

Summary content should track:

- current task
- recent success/failure state
- important files/subsystems in play
- unresolved issue, if one exists

Verify:

- summary is updated before quip generation on `turn_completed`
- summary is discarded when the session ends

### 11. Implement the `codex exec` quip backend

Deliver:

- a wrapper-owned `CodexExecQuipBackend`
- default backend config using:
  - `model = "gpt-5.4-mini"`
  - `model_reasoning_effort = "medium"`
- machine-readable quip output parsing
- per-session quip rendering

Quip context packet must include:

- `name`
- `personality_paragraph`
- normalized event
- rolling session summary
- structured digest of recent turns
- optional short raw excerpts
- session `cwd`

Verify:

- quip generation runs in the active session `cwd`
- malformed output is dropped cleanly
- the configured model slug is validated at startup or first use
- output schema accepts only:
  - `emit`
  - `text`
  - `tone`

### 12. Implement quip policy and safety gates

Deliver:

- primary quip opportunities on:
  - `tool_finished`
  - `turn_completed`
- long-run quip gate:
  - one quip max per active phase
  - only after 20 minutes
- hard per-session cooldown:
  - 10 minutes
- quip blacklist for:
  - secrets/auth/token material
  - raw crash/stack-trace-only contexts
  - explicit user frustration/distress
  - mocking/intrusive/unsafe moments

Verify:

- eligible events may still produce `emit: false`
- no retry storm occurs on repeated failures
- different sessions can quip independently

### 13. Harden multi-session behavior

Deliver:

- one global Buddy identity
- many concurrent session-local presences
- concurrency-safe writes for hatch, rebirth, mute, and future long-lived state

Verify:

- two open sessions can show different quips simultaneously
- hatch/rebirth/mute writes do not corrupt storage under contention

### 14. Confirm degraded-mode behavior

Verify:

- if hooks fail, Codex PTY still works and Buddy falls back to passive mode
- if `codex exec` quip generation fails, no bubble is shown and no crash occurs
- if storage fails, Codex still launches and Buddy degrades cleanly

## Acceptance Checklist

The port is ready when all of the following are true:

- Buddy appears as an always-visible, focusable side pane
- Codex remains unmodified and runs inside the PTY
- Buddy identity is wrapper-owned and survives across sessions/accounts
- hatch and rebirth follow the `hatch_seed` contract
- personality text is visible only in the in-pane `status` view
- quips are contextual, session-local, and generated via `codex exec`
- rolling summaries are ephemeral per session
- multi-session behavior is concurrency-safe
- Codex remains usable when any Buddy subsystem degrades
