# Buddy Clean Room Spec

## Scope

This document specifies the observable behavior of the `buddy/` module in this repository snapshot. It is intended to support a from-scratch reimplementation without reusing the original source.

The covered surface area is:

- deterministic companion generation from a user identity
- persisted companion merge rules
- prompt attachment behavior for introducing the companion
- startup teaser notification behavior
- `/buddy` trigger detection
- narrow and full terminal rendering behavior for the companion sprite and speech bubble

This snapshot does not contain the full hatch flow, the `/buddy` command handler, or the system that generates the companion's soul text. Those pieces are therefore specified only where they are referenced by the `buddy/` module.

## Design Goals

- Each user gets a stable companion body plan derived from identity, not from stored config edits.
- The companion's "soul" is persistent and authored once at hatch time.
- The companion is lightweight enough to render on a 500 ms tick in hot UI paths.
- The feature degrades cleanly on narrow terminals and when muted or disabled.

## Required Host Interfaces

A compatible host must provide the following capabilities:

- a `BUDDY` feature flag
- a global config object with:
  - `oauthAccount?.accountUuid`
  - `userID`
  - `companion`
  - `companionMuted`
- app state fields for:
  - `companionReaction?: string`
  - `companionPetAt?: number`
  - `footerSelection`
- a way to enqueue and remove notifications
- a fullscreen-mode query
- terminal column measurement
- a display-width function for strings
- a theme with the named colors `inactive`, `success`, `permission`, `autoAccept`, and `warning`

## Data Model

### Enumerations

`Rarity`

- `common`
- `uncommon`
- `rare`
- `epic`
- `legendary`

`Species`

- `duck`
- `goose`
- `blob`
- `cat`
- `dragon`
- `octopus`
- `owl`
- `penguin`
- `turtle`
- `snail`
- `ghost`
- `axolotl`
- `capybara`
- `cactus`
- `robot`
- `rabbit`
- `mushroom`
- `chonk`

`Eye`

- `·`
- `✦`
- `×`
- `◉`
- `@`
- `°`

`Hat`

- `none`
- `crown`
- `tophat`
- `propeller`
- `halo`
- `wizard`
- `beanie`
- `tinyduck`

`StatName`

- `DEBUGGING`
- `PATIENCE`
- `CHAOS`
- `WISDOM`
- `SNARK`

### Core Types

`CompanionBones`

- `rarity: Rarity`
- `species: Species`
- `eye: Eye`
- `hat: Hat`
- `shiny: boolean`
- `stats: Record<StatName, number>`

`CompanionSoul`

- `name: string`
- `personality: string`

`Companion`

- all `CompanionBones`
- all `CompanionSoul`
- `hatchedAt: number`

`StoredCompanion`

- all `CompanionSoul`
- `hatchedAt: number`

Stored companions do not normatively persist bones. On read, a live companion is reconstructed by merging the stored soul with freshly regenerated bones.

## Deterministic Generation

### Canonical User Identity

The canonical user identity is chosen in this order:

1. `config.oauthAccount.accountUuid`
2. `config.userID`
3. the literal fallback `anon`

### Seed Derivation

The deterministic roll for a real user is keyed by:

- `userId + "friend-2026-401"`

The implementation uses a 32-bit hash and a Mulberry32 PRNG. Exact internal memoization is not normative, but the output must be stable for a given canonical user identity.

`rollWithSeed(seed)` is the same generation pipeline without the fixed salt; it hashes the provided seed string directly.

### Rarity Weights

Weighted random selection must use these weights:

| Rarity | Weight |
| --- | ---: |
| `common` | 60 |
| `uncommon` | 25 |
| `rare` | 10 |
| `epic` | 4 |
| `legendary` | 1 |

Selection is standard roulette-wheel sampling over the weights above.

### Bone Generation Rules

After rarity is chosen:

- `species` is chosen uniformly from all species.
- `eye` is chosen uniformly from all eye glyphs.
- `hat` is:
  - always `none` for `common`
  - otherwise chosen uniformly from the full hat list, including `none`
- `shiny` is true with probability `0.01`

### Stat Generation Rules

Each rarity has a minimum floor:

| Rarity | Floor |
| --- | ---: |
| `common` | 5 |
| `uncommon` | 15 |
| `rare` | 25 |
| `epic` | 35 |
| `legendary` | 50 |

Stat rolling algorithm:

1. Pick one stat uniformly as the `peak`.
2. Pick a different stat uniformly as the `dump`.
3. For each stat:
   - peak stat: `min(100, floor + 50 + randomInt(0..29))`
   - dump stat: `max(1, floor - 10 + randomInt(0..14))`
   - every other stat: `floor + randomInt(0..39)`

The roll also emits `inspirationSeed = floor(rng() * 1_000_000_000)`.

## Persistence and Read Semantics

`getCompanion()` behaves as follows:

- if `config.companion` is absent, return no companion
- otherwise regenerate bones from the canonical user identity
- merge stored soul fields with regenerated bones
- regenerated bones win over any stale bone fields that may exist in old config data

This means:

- users cannot promote themselves to a rarer companion by editing config
- species list changes do not invalidate existing stored companions
- names, personality text, and hatch timestamp persist

## Prompt Attachment Behavior

### Intro Attachment Injection

An attachment introducing the companion is added only when all conditions hold:

- the `BUDDY` feature flag is enabled
- a companion exists
- the global config is not muted
- the current message history does not already contain a `companion_intro` attachment for the same companion name

The emitted attachment payload is:

- `type: "companion_intro"`
- `name: companion.name`
- `species: companion.species`

### Assistant Guidance Text

The companion intro text instructs the primary assistant that:

- a small companion sits next to the input box and can comment in a speech bubble
- the companion is a separate entity, not the main assistant
- when the user addresses the companion by name, the main assistant should stay out of the way
- in that case, the main assistant should respond in one line or less, or only answer the portion that is meant for the assistant
- the main assistant must not explain that it is not the companion and must not narrate the companion's answer

## Release Gating and Teaser Behavior

### Live Date

Buddy is considered live on local time once the local date is in April 2026 or later.

Equivalent rule:

- `year > 2026`, or
- `year == 2026` and `month >= April`

### Teaser Window

The teaser window is open only during local dates `2026-04-01` through `2026-04-07`, inclusive.

### Startup Teaser Notification

On startup, the host may show a teaser notification only when all conditions hold:

- the `BUDDY` feature flag is enabled
- no companion has been hatched yet
- the current local date is inside the teaser window

The teaser notification:

- uses key `buddy-teaser`
- renders the literal text `/buddy`
- colors each character independently with a rainbow sequence
- uses immediate priority
- auto-dismisses after 15 seconds
- unregisters itself by removing `buddy-teaser` on cleanup

## `/buddy` Trigger Detection

Given arbitrary text input, trigger detection returns all spans matching the regex:

```text
/\/buddy\b/g
```

Each match yields:

- `start`: the match start index
- `end`: the first index after the match

No match is returned when the `BUDDY` feature flag is disabled.

## Visual System

### Color Mapping

Rarity maps to theme color names as follows:

| Rarity | Theme color |
| --- | --- |
| `common` | `inactive` |
| `uncommon` | `success` |
| `rare` | `permission` |
| `epic` | `autoAccept` |
| `legendary` | `warning` |

### Compact Face Strings

The narrow presentation uses one-line face strings:

| Species | Face pattern |
| --- | --- |
| `duck` | `({eye}>` |
| `goose` | `({eye}>` |
| `blob` | `({eye}{eye})` |
| `cat` | `={eye}ω{eye}=` |
| `dragon` | `<{eye}~{eye}>` |
| `octopus` | `~({eye}{eye})~` |
| `owl` | `({eye})({eye})` |
| `penguin` | `({eye}>)` |
| `turtle` | `[{eye}_{eye}]` |
| `snail` | `{eye}(@)` |
| `ghost` | `/{eye}{eye}\` |
| `axolotl` | `}{eye}.{eye}{` |
| `capybara` | `({eye}oo{eye})` |
| `cactus` | `|{eye}  {eye}|` |
| `robot` | `[{eye}{eye}]` |
| `rabbit` | `({eye}..{eye})` |
| `mushroom` | `|{eye}  {eye}|` |
| `chonk` | `({eye}.{eye})` |

### Sprite Asset Contract

The full sprite renderer depends on a species-to-frames asset library with these constraints:

- each species has exactly 3 animation frames
- each frame is conceptually 5 text rows tall
- nominal body width is 12 columns after eye substitution
- the eye placeholder is replaced with the selected eye glyph
- row 0 is the hat row
- frames 0 and 1 should leave row 0 blank
- frame 2 may use row 0 for species-specific effects such as smoke or props

Hat row glyphs:

| Hat | Top-row glyphs |
| --- | --- |
| `none` | empty |
| `crown` | `\^^^/` centered in the 12-column row |
| `tophat` | `[___]` centered |
| `propeller` | `-+-` centered |
| `halo` | `(   )` centered |
| `wizard` | `/^\` centered |
| `beanie` | `(___)` centered |
| `tinyduck` | `,>` near the middle-left |

Hat overlay rules:

- apply the hat only if the current frame's row 0 is blank
- if the current frame's row 0 is blank and every frame for that species also has a blank row 0, drop that row from the rendered output entirely
- never drop row 0 when any frame for the species uses it, or sprite height will oscillate between frames

### Runtime Rendering Rules

All visual renderers in this module return no UI when any of the following is true:

- the `BUDDY` feature flag is disabled
- no companion exists
- the companion is muted

#### Global Timing Constants

- tick interval: 500 ms
- speech bubble lifetime: 20 ticks, about 10 seconds
- speech bubble fade window: last 6 ticks, about 3 seconds
- pet-heart burst lifetime: 2500 ms

#### Idle Animation

The idle sequence is a fixed repeating series of frame intents:

```text
[0, 0, 0, 0, 1, 0, 0, 0, blink, 0, 0, 2, 0, 0, 0]
```

Interpretation:

- `0`, `1`, `2` mean render that sprite frame
- `blink` means render frame 0 but replace all eye glyphs in the output with `-`

#### Excited Animation

When the companion is speaking or in the petting burst:

- stop using the idle sequence
- cycle through every available frame continuously based on the global tick

#### Petting Overlay

Petting is driven by a host state timestamp, `companionPetAt`.

When a new pet timestamp arrives:

- capture the current tick as the pet start tick before the next render commits
- for the next 2500 ms, prepend a floating heart row above the sprite
- use a 5-frame heart animation, advancing one heart frame per 500 ms tick

In compact mode, petting shows a single leading heart before the face instead of the multi-line overlay.

#### Speech Bubble Formatting

Bubble text behavior:

- wrap on spaces only
- target wrap width is 30 characters
- rendered text lives in a rounded border box of width 34
- horizontal padding inside the box is 1

Bubble color behavior:

- normal state uses the companion rarity color
- faded state uses `inactive` for the border and text

Bubble text styling:

- bubble text is italic
- normal-state bubble text uses dim coloring
- faded-state bubble text is no longer dimmed and instead relies on the `inactive` color

Bubble tail behavior:

- inline mode uses a one-character tail extending rightward from the bubble
- fullscreen overlay mode uses a two-line downward tail aligned to the bubble's right edge

#### Layout Thresholds and Width Reservation

Constants:

- full-sprite threshold: 100 terminal columns
- sprite body width: 12
- name-row padding budget: 2
- sprite horizontal padding: 2
- inline bubble width budget: 36
- narrow-mode quip cap: 24 characters

`spriteColWidth(nameWidth)` is:

- `max(12, nameWidth + 2)`

`companionReservedColumns(terminalColumns, speaking)` is:

- `0` if the feature is disabled
- `0` if no companion exists
- `0` if the companion is muted
- `0` if `terminalColumns < 100`
- otherwise `spriteColWidth(displayWidth(companion.name)) + 2 + bubbleWidth`

Where:

- `bubbleWidth = 36` only when the companion is speaking and the app is not fullscreen
- `bubbleWidth = 0` in fullscreen, because the bubble is rendered in an overlay slot

#### Narrow Mode

Narrow mode is active when terminal width is below 100 columns.

Rendering rules:

- render a single-line face string plus a label
- if petting is active, prepend one heart glyph in `autoAccept`
- the face glyphs are bold and use the rarity color
- the label is:
  - `"reaction text"` when speaking
  - otherwise ` companionName ` when focused
  - otherwise `companionName`
- if a reaction exceeds 24 characters, truncate to 23 characters and append an ellipsis
- the label text is always italic
- focused, non-speaking labels are inverse-highlighted and bold
- unfocused, non-speaking labels are dim
- speaking labels use the rarity color until the fade window starts, then `inactive`

#### Full Sprite Mode

Full sprite mode is active when terminal width is at least 100 columns.

When not speaking:

- render only the sprite column

When speaking and not fullscreen:

- render the speech bubble inline to the left of the sprite column
- align the row to the sprite baseline
- keep the sprite column from shrinking

When speaking and fullscreen:

- render only the sprite column in the main footer area
- render the bubble separately in a floating overlay region

Sprite column rules:

- center-align the sprite art and the name row
- sprite width is `spriteColWidth(displayWidth(companion.name))`
- the name row always exists
- name rows are always italic
- focused name rows are bold, inverse-highlighted, and use the rarity color
- unfocused name rows are dim
- when pet hearts are present, the prepended heart row uses `autoAccept`
- all other sprite rows use the rarity color

#### Bubble Lifetime

When `companionReaction` becomes defined:

- record the current global tick as `lastSpokeTick`
- schedule a clear for approximately 10 seconds later
- do not clear if the reaction was already removed before the timeout fires

Fade begins once the bubble age reaches 14 ticks.

The fullscreen floating bubble maintains its own local tick counter for fade timing, resetting whenever the reaction string changes.

## Non-Goals and Explicit Unknowns

This repository snapshot does not define:

- how a companion is first hatched
- how `name` and `personality` are generated
- what text causes `companionReaction` to be set
- how `/buddy pet` is parsed and dispatched
- where the fullscreen floating bubble is mounted, beyond the requirement that it live outside any clipping region that would cut it off

Any compatible reimplementation must provide those missing pieces separately.

This repository now includes one reconstructed completion contract in
[`HOST_INTEGRATION_SPEC.md`](./HOST_INTEGRATION_SPEC.md); that supplement
should be read as host-level reconstruction, not as directly observed snapshot
behavior.

For a Codex wrapper TUI port target, see
[`CODEX_WRAPPER_PORT_SPEC.md`](./CODEX_WRAPPER_PORT_SPEC.md).
