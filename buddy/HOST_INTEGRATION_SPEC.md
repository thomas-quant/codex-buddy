# Buddy Host Integration Supplement

## Purpose

This document specifies one compatible reconstruction of the Buddy behaviors
that the extracted `buddy/` snapshot references but does not implement.

It complements [CLEAN_ROOM_SPEC.md](./CLEAN_ROOM_SPEC.md):

- `CLEAN_ROOM_SPEC.md` is the source of truth for behavior directly observable
  from the extracted snapshot.
- this document defines the missing host-level contract needed to restore a
  complete Buddy feature in a larger application

The intent is to give a future reimplementation a tighter handoff than "these
parts are missing" without pretending those parts were directly present in the
snapshot.

## Evidence Basis

### Directly Observed in the Snapshot

- deterministic body generation and persisted-soul merge rules
- muted gating across prompt and visual surfaces
- `companionReaction` bubble rendering and lifetime expectations
- `companionPetAt` heart-burst rendering semantics
- `/buddy` teaser notification and trigger detection

### Reconstructed for Host Completion

The sections below are reconstructed requirements derived from:

- the integration seams exposed by the `buddy/` snapshot
- the missing-pieces note in
  `docs/superpowers/plans/2026-04-07-buddy-missing-pieces.md`

They are normative for a compatible "complete Buddy" implementation, but they
should be treated as reconstruction work rather than directly observed source.

## Required Additional Host Interfaces

A host completing the Buddy feature must additionally provide:

- a slash-command registry capable of handling `/buddy`
- a global config write path for:
  - `companion`
  - `companionMuted`
- an app-state write path for:
  - `companionPetAt`
  - `companionReaction`
- a lightweight side-query interface for short, non-streaming model calls
- access to recent post-turn messages so the observer can decide whether to
  emit a reaction

## Slash Command Contract

### Supported Surface

The supported Buddy command surface is:

- `/buddy`
- `/buddy status`
- `/buddy pet`
- `/buddy mute`
- `/buddy unmute`

No other subcommands are required by this supplement.

### Default `/buddy` Semantics

Bare `/buddy` is a hatch-or-status command:

- if no companion exists yet, it hatches one
- if a companion already exists, it shows the current status

`/buddy status` always behaves like the status branch of bare `/buddy`.

### Unknown Subcommands

Unknown subcommands return this exact text:

```text
Unknown /buddy subcommand. Use /buddy, /buddy status, /buddy pet, /buddy mute, or /buddy unmute.
```

## Hatch Flow

### Hatch Trigger

A hatch occurs when:

- the user invokes bare `/buddy`
- no stored companion currently exists

### Hatch Inputs

Hatching uses:

- canonical user identity and deterministic bones from
  `roll(companionUserId())`
- a one-time soul-generation step that produces:
  - `name`
  - `personality`

### Soul Generation Contract

Soul generation must:

- take the deterministic bones as context
- produce a short companion name and one-sentence personality
- preserve the split between deterministic "bones" and persisted "soul"
- avoid widening the persisted config shape beyond `StoredCompanion`

The soul-generation system is intentionally lightweight:

- it should use a short side query rather than the main streaming chat path
- it may use species, rarity, stats, and a stable user-derived seed as prompt
  context
- it must return structured `{ name, personality }` data or a host-generated
  fallback

### Hatch Failure Handling

Hatching must never fail solely because the side query fails.

If soul generation fails or returns unusable output, the host must still hatch
the companion using a deterministic or simple local fallback for:

- `name`
- `personality`

### Hatch Persistence

After a successful hatch, the host persists only:

- `name`
- `personality`
- `hatchedAt`

No deterministic bone fields are persisted.

### Hatch Output Requirements

The hatch response must remain single-screen and text-first. It must include:

- the companion's name
- the companion's species
- the companion's rarity
- the companion's personality

The response may include extra celebratory copy, but it must not require the
sprite renderer to understand the result.

## Status Output

Status output is compact and text-first.

At minimum it must present:

- the companion's name
- the companion's species
- the companion's rarity
- the companion's personality
- a hint that `/buddy pet`, `/buddy mute`, and `/buddy unmute` exist

A compatible compact format is:

```text
{name} the {species}
{rarity} companion
{personality}
Try /buddy pet, /buddy mute, or /buddy unmute.
```

## Pet Contract

### No-Companion Case

If `/buddy pet` is invoked before hatch, return this exact text:

```text
Hatch your companion first with /buddy.
```

### Success Case

If a companion exists, `/buddy pet` must:

- set `companionPetAt` to the current wall-clock timestamp in milliseconds
- return this exact text:

```text
You pet {name}.
```

This timestamp is the existing trigger consumed by the sprite renderer for the
heart-burst animation described in `CLEAN_ROOM_SPEC.md`.

## Mute Contract

### No-Companion Case

If `/buddy mute` or `/buddy unmute` is invoked before hatch, return this exact
text:

```text
Nothing to mute yet. Hatch your companion first with /buddy.
```

### Success Case

If a companion exists:

- `/buddy mute` persists `companionMuted: true` and returns:

```text
{name} is now muted.
```

- `/buddy unmute` persists `companionMuted: false` and returns:

```text
{name} is back.
```

### Muted Effects

Mute state must suppress all Buddy-presented surfaces already defined by the
snapshot:

- prompt intro attachment injection
- sprite rendering
- speech-bubble reactions

The stored companion itself is not deleted by muting.

## Reaction Observer Contract

### Observer Entry Point

The host should expose a narrow post-turn observer of the form:

```ts
fireCompanionObserver(
  messages: Message[],
  onReaction: (reaction: string | undefined) => void,
): Promise<void>
```

The observer does not own app-state writes directly. The caller remains the
single writer of `companionReaction`.

### When the Observer Must Skip

The observer must return no reaction when any of the following is true:

- no companion exists
- the companion is muted
- there is no recent assistant text worth reacting to
- the host's occasional-reaction gate decides this turn should stay silent

The reaction system is intentionally occasional, not per-turn.

### Generation Contract

When it does run, the observer should:

- inspect a compact recent transcript rather than the full session
- use a lightweight side query rather than the main streaming query loop
- ask for a single-line companion reaction only
- favor short, context-aware quips over explanations

The observer's output is a UI bubble, not a new assistant message.

### Sanitization Rules

Before writing any reaction into UI state, the host must sanitize it:

- trim leading and trailing whitespace
- collapse internal whitespace runs to single spaces
- reject empty output
- reject sentinel silence outputs such as:
  - `none`
  - `silent`
  - `skip`
- cap visible length to a short single line

A compatible upper bound is 120 characters.

### Failure Handling

Observer failures must degrade silently:

- side-query failures return no reaction
- malformed output returns no reaction
- the observer must not throw into the main REPL flow

## Integration Boundaries with the Snapshot

This supplement is intentionally narrow.

It does not change the snapshot's existing guarantees about:

- deterministic body generation
- prompt attachment behavior
- teaser notification timing
- sprite rendering thresholds, timing, and fade behavior

Instead, it defines the missing writes and host flows that feed those already
specified read paths:

- `companion` persistence for hatch and status
- `companionMuted` persistence for mute
- `companionPetAt` writes for petting hearts
- `companionReaction` updates for speech bubbles
