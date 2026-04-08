# Buddy Animation Design

## Goal

Add lightweight Buddy sprite animation to the wrapper without changing persisted state, sprite art assets, or the existing Buddy interaction model.

The animation should feel alive but restrained:

- the Buddy rests on the default pose most of the time
- idle motion appears occasionally as a short non-looping burst
- petting triggers a faster short animation burst immediately
- pet animation takes priority over idle animation

## Non-Goals

- no new sprite artwork
- no per-species custom animation scripts
- no persisted animation state
- no continuous looping idle animation
- no changes to Buddy hatch, quip, or store formats

## Existing Constraints

- `wrapper/src/buddy/sprites.rs` already defines 3 frames per species
- `render_sprite_frame(bones, frame)` already renders any frame index
- current UI always renders frame `0`
- runtime already has a periodic `tick()` loop and existing pet feedback timing

## Recommended Approach

Use timer-driven micro-animations owned by the runtime.

This keeps the implementation local to the wrapper runtime, avoids touching persistence, and fits the existing redraw loop. The runtime will maintain transient animation state and choose the current sprite frame. The Buddy pane will render whichever frame the runtime provides.

## Behavior

### Rest Pose

- frame `0` is the default resting pose
- when no animation is active, the Buddy remains on frame `0`

### Idle Animation

- idle animation is burst-based, not looping
- after a quiet period, the runtime starts a short burst
- the burst sequence is `1 -> 2 -> 1 -> 0`
- each idle step uses a moderate frame duration so the motion reads clearly
- after the burst ends, the Buddy returns to frame `0`
- the next idle burst is scheduled for a later time instead of repeating immediately

### Pet Animation

- petting starts a burst immediately
- pet burst uses the same frame family but faster timing than idle
- the burst sequence is `1 -> 2 -> 1 -> 0`
- pet burst overrides an in-progress idle burst
- existing `<3` feedback remains unchanged

## Timing

Use fixed timing constants so behavior is predictable and easy to test.

- idle quiet delay: several seconds between bursts
- idle frame step: slower than pet
- pet frame step: fast and snappy

Exact values should be chosen in code as named constants near the runtime animation logic.

## State Model

Add transient runtime-only animation state. Do not store it in persisted Buddy data.

Suggested fields:

- current frame index
- current animation mode: `Idle`, `Pet`, or `None`
- current step index within the active sequence
- timestamp for when the current animation step started
- timestamp for when the next idle burst may begin

Suggested sequence:

- steps: `[1, 2, 1, 0]`

## Data Flow

1. Runtime tick evaluates whether an animation should advance.
2. If a pet burst is active, it advances first.
3. Otherwise, if idle is inactive and the quiet delay has elapsed, runtime starts an idle burst.
4. Runtime exposes the currently selected frame index to Buddy rendering.
5. Buddy pane rendering uses that frame instead of hardcoding frame `0`.

## Integration Points

### Runtime

Update `wrapper/src/app/runtime.rs` to:

- store animation state in `Runtime`
- initialize the next idle animation deadline
- start a pet burst from the existing pet action path
- advance animation state from `tick()`
- pass the current frame into Buddy rendering

### Buddy Pane

Update `wrapper/src/ui/buddy_pane.rs` to:

- accept a frame index for idle and status rendering helpers
- stop hardcoding frame `0`

### Sprites

Leave `wrapper/src/buddy/sprites.rs` unchanged except for any helper additions that make frame selection cleaner. Existing species frames are sufficient.

## Testing Strategy

Add focused regression tests in `wrapper/tests/` for observable behavior:

- idle rendering uses rest pose when no animation is active
- idle burst advances through non-zero frames and returns to rest
- pet burst starts immediately and overrides idle
- Buddy pane renders the requested frame index instead of always frame `0`

Implementation should keep animation progression deterministic enough for unit tests by using explicit timestamps or injectable time inputs inside animation helpers.

## Risks

- if animation state is derived directly from wall clock at render time, tests become brittle
- if idle scheduling is too aggressive, the Buddy will look noisy instead of occasional
- if pet and idle share state poorly, petting can leave the Buddy stuck on a non-rest frame

The implementation should therefore centralize frame progression in one small helper and make returning to frame `0` explicit at the end of every burst.
