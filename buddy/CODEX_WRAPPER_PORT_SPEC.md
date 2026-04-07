# Buddy Codex Wrapper Port Spec

## Scope

This document refines the Buddy port target for a specific host shape:

- a wrapper TUI
- stock Codex running inside a PTY
- Buddy rendered and controlled by the wrapper, not by Codex itself

It does not replace [CLEAN_ROOM_SPEC.md](./CLEAN_ROOM_SPEC.md). Instead:

- `CLEAN_ROOM_SPEC.md` remains the source of truth for Buddy behavior directly
  observed from the extracted snapshot
- `HOST_INTEGRATION_SPEC.md` defines the missing host-level Buddy behaviors
- this document maps those behaviors into a concrete Codex-wrapper runtime

## Goals

- preserve the extracted Buddy behavior where it already exists
- keep stock Codex unmodified
- make the port concrete enough to implement in a wrapper TUI
- stay as close as practical to the "interactive quips with project context"
  feel of Claude Code buddies

## Non-Goals

- intercepting `/buddy` from the Codex composer
- storing Buddy state inside Codex config
- patching Codex's own TUI to render Buddy
- making Buddy a Codex plugin or slash command

## Host Architecture

The wrapper is the Buddy host. Codex is an unmodified child process.

### Runtime Split

The runtime is divided into these responsibilities:

- `CodexPtyHost`
  - launches Codex in a PTY
  - forwards terminal resize events
  - forwards keyboard input when PTY focus is active
  - renders Codex output in the main pane
- `CodexHookAdapter`
  - launches Codex with wrapper-provided hooks for the session
  - receives raw hook payloads from Codex
  - converts raw hook invocations into wrapper-readable event inputs
- `BuddyEventNormalizer`
  - converts raw Codex hook payloads into stable Buddy events
  - isolates Buddy logic from Codex hook payload drift
- `BuddyStore`
  - owns persistent Buddy identity state in wrapper-managed storage
  - owns volatile per-session Buddy state in wrapper memory
- `BuddyPaneController`
  - owns pane focus
  - owns the action menu
  - owns keyboard navigation when the Buddy pane is focused
- `BuddyRenderer`
  - renders the always-visible Buddy side pane
  - renders sprite, name/status line, quips, and action hint text
- `CodexExecQuipBackend`
  - generates contextual Buddy quips by spawning `codex exec`
  - runs in the active session `cwd`
  - consumes Buddy soul plus session-local context

## Layout Contract

Buddy is rendered in an always-visible side pane.

The side pane:

- is persistently visible while the wrapper session is active
- does not require Codex focus to remain visible
- shows Buddy even when idle
- is focusable
- includes a tiny action menu when focused
- includes a small footnote hint showing the hotkey used to open the action
  menu while the Buddy pane is focused

The main Codex surface remains the PTY pane.

## Interaction Model

Buddy actions are wrapper UI actions, not Codex slash commands.

### Action Vocabulary

The preserved Buddy actions are:

- `hatch`
- `status`
- `pet`
- `mute`
- `unmute`

These actions retain the semantics defined in
`HOST_INTEGRATION_SPEC.md`, but they are invoked from the Buddy pane/menu
instead of by typing `/buddy` into Codex.

### Focus Rules

The wrapper must support two independent focus targets:

- `pty`
- `buddy_pane`

When focus is on the PTY:

- keyboard input goes to Codex
- Buddy remains visible but passive

When focus is on the Buddy pane:

- keyboard input is interpreted by the Buddy pane controller
- the action menu can be opened and navigated by keyboard
- Codex does not receive those keystrokes

## Persistence Model

Buddy persistence is wrapper-owned.

### Global Persistent State

The wrapper persists one Buddy identity across all sessions.

Persistent Buddy state includes:

- `hatch_seed`
- `name`
- `personality_paragraph`
- `hatchedAt`
- `lastRebirthAt`
- `muted`
- any future long-lived growth fields

Persistent Buddy state does not include:

- session-local summary
- session-local quip history
- current bubble text
- current pet animation timestamp

### Session-Local Volatile State

Each open Codex session maintains its own volatile Buddy session state.

Session-local state includes:

- wrapper session identifier
- Codex session identifier, if available
- `cwd`
- normalized event history
- rolling session summary
- last few raw turns
- active bubble text
- bubble fade timer state
- pet burst timestamp
- pane focus state
- action menu state
- last processed Codex `turn_id`, if available

Session-local state is ephemeral and must be discarded when that wrapper
session ends.

## Multi-Session Behavior

Buddy is one persistent identity with many concurrent session presences.

If two Codex sessions are open at once:

- both render the same Buddy identity
- both use the same persistent soul and mute state
- each keeps its own local summary
- each keeps its own local recent-turn buffer
- each may show different quips at the same time
- each owns its own bubble lifetime and fade timing

Global writes such as hatch or mute must be concurrency-safe.

Compatible implementations should use:

- file locking, or
- versioned compare-and-swap writes

## Hatch And Rebirth Lifecycle

Buddy identity is wrapper-owned and must not be derived from Codex account
identity.

### Identity Root

The wrapper should create and persist a local `hatch_seed` when Buddy is first
born.

This seed is the root identity for:

- deterministic bones
- hatch prompt flavoring
- deterministic hatch fallback behavior

This keeps Buddy stable across:

- multiple Codex accounts
- multiple repositories
- multiple sessions in the same wrapper

### Hatch Trigger

In the wrapper port, hatch is triggered from the Buddy pane action menu.

The first successful hatch must:

1. create a new `hatch_seed`
2. derive deterministic bones from that seed
3. generate a one-time soul
4. persist the accepted soul and lifecycle metadata

### Rebirth

`rebirth` is an explicit Buddy-pane action that performs a full identity reset.

Rebirth must:

- generate a new `hatch_seed`
- regenerate deterministic bones from that new seed
- generate a new soul
- reset `hatchedAt`
- set `lastRebirthAt`
- clear any future long-lived growth/affinity fields
- invalidate session-local summaries and bubbles

Rebirth is a full reset, not a soul-only reroll.

### Rebirth Cooldown

Rebirth is allowed only when:

- Buddy already exists, and
- at least 14 days have elapsed since:
  - `lastRebirthAt`, if present, otherwise
  - `hatchedAt`

If rebirth is unavailable, the Buddy pane should disable the action and present
a compact cooldown hint such as:

```text
Rebirth available in 9 days
```

## Event Model

Buddy logic must consume normalized wrapper events rather than raw Codex hook
payloads.

### Normalized Events

The wrapper event model includes:

- `session_started`
- `user_turn_submitted`
- `tool_started`
- `tool_finished`
- `turn_completed`
- `session_ended`

### Event Payload Principles

Normalized events should include only the fields Buddy actually needs, such as:

- timestamp
- session id
- turn id
- tool name
- success/failure
- compact recent assistant text
- compact recent user intent
- whether the event is user-visible

The wrapper must treat Codex hook payloads as adapter input, not as the
long-term Buddy API.

## Codex Hook Integration

The wrapper should launch Codex with wrapper-provided hooks configured for that
session.

The hook layer is used as an event source for Buddy state changes.

Hooks are expected to provide the raw material for:

- detecting turn starts
- observing tool starts and finishes
- capturing compact recent assistant output
- triggering post-turn quip decisions

## Rolling Session Summary

Each open Codex session maintains an ephemeral rolling summary.

This summary exists only for quip generation and should not be persisted after
the session ends.

### Update Triggers

The rolling summary should be updated:

- after `turn_completed`
- after notable `tool_finished` events
- after obvious user intent shifts

### Summary Content

The rolling summary should remain compact and cumulative. It should track:

- the current task
- recent success/failure state
- important files or subsystems in play
- unresolved issue, if one exists

## Soul Generation

The wrapper should preserve the body/soul split:

- body is deterministic from `hatch_seed`
- soul is authored once at hatch or rebirth, then persisted

### Hatch Input Contract

Hatch generation should be seed-and-bones driven only.

The hatch-generation backend should see:

- `hatch_seed`
- deterministic bones derived from that seed
- strict output schema
- style constraints for the soul output

It should not see:

- current repository contents
- current session `cwd`
- current conversation
- Codex account identity

Repository and conversation context should influence later quips, not who Buddy
is at birth.

### Soul Output Shape

The hatch-generation backend should return:

- `name`
- `personality_paragraph`

`name` should be short and characterful.

`personality_paragraph` should be the canonical internal character brief used by
future quip generation.

### Soul Visibility Rules

The full `personality_paragraph` should not be shown in the idle Buddy pane.

UI rules:

- idle pane: do not show the personality paragraph
- focused pane: do not show the personality paragraph by default
- `status` detail view: show the full personality paragraph
- quip generation: always receive the full personality paragraph

### Status View

`status` should open inside the Buddy pane as a temporary detail state.

The status view should show:

- `name`
- visible deterministic traits worth exposing
- hatch date or age
- rebirth availability/cooldown
- full `personality_paragraph`

### Hatch Failure Handling

Hatch must never fail solely because soul generation fails.

If hatch generation via `codex exec` fails or returns unusable output, the
wrapper must silently fall back to deterministic local soul generation.

The fallback must:

- use `hatch_seed` and deterministic bones
- produce a valid `name`
- produce a valid `personality_paragraph`
- avoid retry loops before first birth completes

Fallback use should be silent in the UI.

## Quip Generation

Buddy quips are generated by a wrapper-owned `CodexExecQuipBackend`.

### Why `codex exec`

The quip backend should use `codex exec` because the target experience is
intended to stay close to Claude Code buddies:

- contextual
- project-aware
- aware of recent session history
- flavored by the Buddy soul

### Default Quip Backend Configuration

The initial default quip backend target is:

- model: `gpt-5.4-mini`
- reasoning effort: `medium`

This default reflects the chosen Buddy quip style target: small enough to stay
cheap and responsive, but with enough reasoning budget to stay context-aware.

Because the official OpenAI docs currently surface Codex model selection and
`model_reasoning_effort`, but do not expose a standalone `GPT-5.4 mini` model
page in the same way they expose `GPT-5.4`, the implementation should treat the
literal `gpt-5.4-mini` slug as a startup-validated default rather than as an
assumed permanent contract.

If the configured slug is unavailable in the local Codex model catalog or
runtime, the wrapper should fail over in this order:

1. user override
2. wrapper-configured fallback
3. `gpt-5.4`

### Quip Context Packet

Each quip-generation invocation should include:

- Buddy soul:
  - `name`
  - `personality_paragraph`
  - optional future fields such as mood/affinity
- current normalized event
- current session `cwd`
- rolling session summary
- last few raw user/assistant turns
- recent notable tool outcomes

The backend should run in the active session `cwd`.

### Quip Timing Policy

Quips are milestone-driven by default.

Primary quip opportunities:

- `tool_finished`
- `turn_completed`

Secondary quip opportunities:

- rare long-running phases during extended work

The wrapper should not attempt quips for every:

- `tool_started`
- user turn
- minor event

### Long-Run Quip Gate

A long-running phase may produce at most one mid-run quip when all conditions
hold:

- the phase has been active for at least 20 minutes
- no quip has already fired for that phase
- Buddy is not muted
- session cooldown allows it

Long-run quips should read as ambient presence rather than as milestone verdicts.

### Session Cooldown

The wrapper must enforce a hard per-session cooldown of 10 minutes between
emitted quips.

This cooldown applies to:

- milestone quips
- long-run quips

Other open sessions are unaffected by a session's cooldown.

### Two-Stage Quip Decision

Quip emission is a two-stage process:

1. the wrapper policy gate decides whether this event is eligible for a quip
   attempt
2. the quip backend may still return `emit: false`

An eligible event is not guaranteed to display a quip.

### Quip Output Contract

The quip-generation process must return machine-readable output that the wrapper
can validate before rendering.

A compatible output shape is:

```json
{
  "emit": true,
  "text": "Nice steady fix.",
  "tone": "pleased"
}
```

Where:

- `emit` is required
- `text` is required only when `emit` is `true`
- `tone` is optional

The wrapper must reject malformed output and treat it as `emit: false`.

### Quip Tone Enum

If provided, `tone` must be one of:

- `pleased`
- `amused`
- `concerned`
- `sleepy`
- `impressed`
- `neutral`

### Quip Sanitization

Before rendering a quip, the wrapper must:

- trim surrounding whitespace
- collapse internal whitespace to single spaces
- require a single visible line
- apply a maximum visible length cap of 80 characters
- drop malformed or empty output

### Quip Policy

The wrapper should not generate quips for every event.

It should apply a lightweight policy gate first, based on:

- event type
- time since last quip in that session
- whether the event was notable enough
- whether Buddy is muted

### Quip Blacklist

The wrapper must suppress quip attempts entirely when the immediate context
contains any of the following:

- obvious secrets, authentication material, or tokens
- raw crash/log/stack-trace-only output with no meaningful interpretation yet
- explicit user frustration or distress signals
- moments where a Buddy quip would likely read as mocking, intrusive, or unsafe

This blacklist applies before `codex exec` is invoked.

### Quip Context Packet

Each `codex exec` quip attempt should receive:

- Buddy soul:
  - `name`
  - `personality_paragraph`
- normalized event:
  - `type`
  - `timestamp`
  - `tool_name`, if relevant
  - success/failure, if relevant
- rolling session summary
- compact structured digest of recent turns
- optionally 1-2 short raw turn excerpts
- current session `cwd`

The goal is to provide strong project and conversation context without passing
the full raw transcript every time.

### Summary Update Order

On `turn_completed`, the wrapper should:

1. update the rolling session summary
2. evaluate the quip policy gate
3. if eligible, run `codex exec`
4. sanitize and render the result, or drop it

For long-run quips, the wrapper should use the last settled summary plus current
phase metadata because the turn is not yet complete.

## Failure And Degradation Behavior

Codex is the primary product surface. Buddy must degrade cleanly.

### Hook Failure

If hooks fail:

- Codex PTY must still function
- Buddy falls back to passive pane behavior
- no crash should propagate into the PTY host

### Quip Failure

If `codex exec` quip generation fails:

- render no quip
- do not crash
- do not retry in a tight loop

### Storage Failure

If Buddy storage is unavailable:

- Codex PTY must still function
- Buddy may degrade to an unhatched/passive state
- the wrapper must not block Codex startup solely because Buddy storage failed

## Porting Implications

A Codex wrapper implementation should treat the Buddy docs as layered:

1. `CLEAN_ROOM_SPEC.md`
   Snapshot-derived behavior for bones, rendering, prompt semantics, teaser
   behavior, trigger detection, and UI timing.
2. `HOST_INTEGRATION_SPEC.md`
   Missing Buddy host behaviors: hatch, pet, mute, unmute, and reaction
   observer semantics.
3. `CODEX_WRAPPER_PORT_SPEC.md`
   Concrete host mapping into a PTY wrapper around stock Codex.

For an ordered implementation sequence, see
[`CODEX_WRAPPER_PORTING_CHECKLIST.md`](./CODEX_WRAPPER_PORTING_CHECKLIST.md).
