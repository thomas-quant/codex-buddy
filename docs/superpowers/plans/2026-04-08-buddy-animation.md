# Buddy Animation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add occasional non-looping idle animation bursts and a faster pet animation burst to the Buddy pane without changing persisted state or sprite assets.

**Architecture:** Keep animation state transient and runtime-owned by introducing a small `buddy::animation` helper that is driven by `Instant` timestamps. Reuse the existing three-frame species art, thread the active frame index into Buddy rendering, and let the runtime tick advance animation while preserving the existing quip and pet-heart behavior.

**Tech Stack:** Rust 2024, `std::time::Instant`, existing `ratatui` UI, existing wrapper integration tests under `wrapper/tests/`.

---

## File Structure

### Create

- `wrapper/src/buddy/animation.rs`
  Pure runtime-only animation state machine for occasional idle bursts and pet bursts.
- `wrapper/tests/buddy_animation.rs`
  Focused regressions for animation scheduling, burst progression, and pet override behavior.

### Modify

- `wrapper/src/buddy/mod.rs`
  Export the new animation module.
- `wrapper/src/ui/buddy_pane.rs`
  Accept a caller-supplied frame index instead of hardcoding frame `0`.
- `wrapper/src/app/runtime.rs`
  Own the animation state, tick it, reset it when Buddy state changes, and trigger pet animation on the pet action path.
- `wrapper/tests/buddy_pane.rs`
  Add a regression proving Buddy pane rendering respects the requested frame index.

### Leave As-Is

- `wrapper/src/buddy/sprites.rs`
  Existing three-frame species art is reused directly.
- `wrapper/src/app/mod.rs`
  Existing Buddy UI focus and pet timestamp state remain sufficient.

### Task 1: Add A Runtime-Only Buddy Animation State Machine

**Files:**
- Create: `wrapper/src/buddy/animation.rs`
- Modify: `wrapper/src/buddy/mod.rs`
- Test: `wrapper/tests/buddy_animation.rs`

- [ ] **Step 1: Write the failing animation-state tests**

Create `wrapper/tests/buddy_animation.rs`:

```rust
use std::time::{Duration, Instant};

use buddy_wrapper::buddy::animation::{
    BuddyAnimation, BuddyAnimationMode, IDLE_QUIET_DELAY, IDLE_STEP_DURATION, PET_STEP_DURATION,
};

#[test]
fn idle_animation_stays_resting_until_quiet_delay_expires() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    animation.tick(now + IDLE_QUIET_DELAY - Duration::from_millis(1));

    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);
}

#[test]
fn idle_animation_advances_through_burst_and_returns_to_rest() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);
    let idle_start = now + IDLE_QUIET_DELAY;

    animation.tick(idle_start);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(idle_start + IDLE_STEP_DURATION);
    assert_eq!(animation.current_frame(), 2);

    animation.tick(idle_start + IDLE_STEP_DURATION + IDLE_STEP_DURATION);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(
        idle_start + IDLE_STEP_DURATION + IDLE_STEP_DURATION + IDLE_STEP_DURATION,
    );
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);
}

#[test]
fn pet_animation_starts_immediately_and_overrides_idle() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);
    let idle_start = now + IDLE_QUIET_DELAY;

    animation.tick(idle_start);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    let pet_start = idle_start + Duration::from_millis(10);
    animation.start_pet(pet_start);
    assert_eq!(animation.mode(), BuddyAnimationMode::Pet);
    assert_eq!(animation.current_frame(), 1);

    animation.tick(pet_start + PET_STEP_DURATION);
    assert_eq!(animation.current_frame(), 2);
}
```

- [ ] **Step 2: Run the new test file and verify it fails for the right reason**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_animation
```

Expected:

- FAIL with unresolved import/module errors for `buddy::animation`

- [ ] **Step 3: Implement the minimal animation helper and export it**

Create `wrapper/src/buddy/animation.rs`:

```rust
use std::time::{Duration, Instant};

pub const IDLE_QUIET_DELAY: Duration = Duration::from_secs(8);
pub const IDLE_STEP_DURATION: Duration = Duration::from_millis(180);
pub const PET_STEP_DURATION: Duration = Duration::from_millis(90);

const BURST_SEQUENCE: [usize; 4] = [1, 2, 1, 0];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuddyAnimationMode {
    None,
    Idle,
    Pet,
}

#[derive(Clone, Debug)]
pub struct BuddyAnimation {
    current_frame: usize,
    mode: BuddyAnimationMode,
    step_index: usize,
    step_started_at: Instant,
    next_idle_at: Instant,
}

impl BuddyAnimation {
    pub fn new(now: Instant) -> Self {
        Self {
            current_frame: 0,
            mode: BuddyAnimationMode::None,
            step_index: 0,
            step_started_at: now,
            next_idle_at: now + IDLE_QUIET_DELAY,
        }
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn mode(&self) -> BuddyAnimationMode {
        self.mode
    }

    pub fn start_pet(&mut self, now: Instant) {
        self.mode = BuddyAnimationMode::Pet;
        self.step_index = 0;
        self.step_started_at = now;
        self.current_frame = BURST_SEQUENCE[0];
    }

    pub fn tick(&mut self, now: Instant) {
        match self.mode {
            BuddyAnimationMode::None => {
                if now >= self.next_idle_at {
                    self.mode = BuddyAnimationMode::Idle;
                    self.step_index = 0;
                    self.step_started_at = now;
                    self.current_frame = BURST_SEQUENCE[0];
                }
            }
            BuddyAnimationMode::Idle | BuddyAnimationMode::Pet => {
                let step = match self.mode {
                    BuddyAnimationMode::Idle => IDLE_STEP_DURATION,
                    BuddyAnimationMode::Pet => PET_STEP_DURATION,
                    BuddyAnimationMode::None => unreachable!(),
                };

                while now.duration_since(self.step_started_at) >= step {
                    self.step_started_at += step;
                    self.step_index += 1;

                    if self.step_index >= BURST_SEQUENCE.len() {
                        self.mode = BuddyAnimationMode::None;
                        self.step_index = 0;
                        self.current_frame = 0;
                        self.next_idle_at = now + IDLE_QUIET_DELAY;
                        return;
                    }

                    self.current_frame = BURST_SEQUENCE[self.step_index];
                }
            }
        }
    }
}
```

Update `wrapper/src/buddy/mod.rs`:

```rust
pub mod animation;
pub mod events;
pub mod lifecycle;
pub mod policy;
pub mod quips;
pub mod roll;
pub mod sprites;
pub mod store;
pub mod summary;
pub mod types;
```

- [ ] **Step 4: Run the animation tests and verify they pass**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_animation
```

Expected:

- PASS with `3 passed; 0 failed`

- [ ] **Step 5: Commit the animation helper**

```bash
git add wrapper/src/buddy/animation.rs wrapper/src/buddy/mod.rs wrapper/tests/buddy_animation.rs
git commit -m "Add buddy animation state machine"
```

### Task 2: Make Buddy Pane Rendering Use The Requested Frame Index

**Files:**
- Modify: `wrapper/src/ui/buddy_pane.rs`
- Modify: `wrapper/tests/buddy_pane.rs`

- [ ] **Step 1: Add a failing frame-sensitive Buddy pane regression**

Append to `wrapper/tests/buddy_pane.rs`:

```rust
#[test]
fn idle_view_renders_the_requested_sprite_frame() {
    let buddy = PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin.");
    let bones = CompanionBones::test_fixture();

    let frame_zero = render_idle_lines(&buddy, &bones, 0, None, false);
    let frame_one = render_idle_lines(&buddy, &bones, 1, None, false);

    assert_eq!(frame_zero[4], "    `--´    ");
    assert_eq!(frame_one[4], "    `--´~   ");
}
```

- [ ] **Step 2: Run the Buddy pane test file and verify it fails**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_pane
```

Expected:

- FAIL because `render_idle_lines` does not accept a frame parameter yet

- [ ] **Step 3: Thread the frame argument into idle and status rendering**

Update `wrapper/src/ui/buddy_pane.rs`:

```rust
pub fn render_idle_lines(
    buddy: &PersistedBuddy,
    bones: &CompanionBones,
    frame: usize,
    quip: Option<&str>,
    focused: bool,
) -> Vec<String> {
    let mut lines = render_sprite_frame(bones, frame);
    lines.push(if focused {
        format!(" {} ", buddy.name)
    } else {
        buddy.name.clone()
    });

    if let Some(quip) = quip {
        lines.push(format!("\"{quip}\""));
    }

    lines
}

pub fn render_status_lines(
    buddy: &PersistedBuddy,
    bones: &CompanionBones,
    frame: usize,
    now: DateTime<Utc>,
) -> Vec<String> {
    let mut lines = render_sprite_frame(bones, frame);
    lines.push(format!("{} the {}", buddy.name, bones.species));
    lines.push(format!("{} companion", bones.rarity));
    lines.push(format!("Eyes: {}  Hat: {}", bones.eye, bones.hat));
    lines.push(format!("Hatched: {}", buddy.hatched_at.format("%Y-%m-%d")));
    lines.push(format!(
        "Age: {} days",
        now.signed_duration_since(buddy.hatched_at).num_days()
    ));
    lines.push(
        if can_rebirth_at(buddy.hatched_at, buddy.last_rebirth_at, now) {
            "Rebirth: available now".to_string()
        } else {
            let gate =
                buddy.last_rebirth_at.unwrap_or(buddy.hatched_at) + chrono::Duration::days(14);
            let days_remaining = (gate - now).num_days().max(0);
            format!("Rebirth: available in {days_remaining} days")
        },
    );
    lines.push(buddy.personality_paragraph.clone());
    lines
}
```

Update the existing test call sites in `wrapper/tests/buddy_pane.rs` to pass frame `0`:

```rust
let lines = render_idle_lines(
    &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
    &CompanionBones::test_fixture(),
    0,
    None,
    false,
);
```

```rust
let lines = render_status_lines(
    &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
    &CompanionBones::test_fixture(),
    0,
    Utc::now(),
);
```

Update the existing runtime call sites in `wrapper/src/app/runtime.rs` to keep using frame `0` until Task 3 wires live animation:

```rust
        let mut lines = match (&self.buddy, &self.bones) {
            (Some(buddy), Some(bones)) if self.app.is_buddy_status_open() => {
                render_status_lines(buddy, bones, 0, Utc::now())
            }
            (Some(buddy), Some(bones)) => render_idle_lines(
                buddy,
                bones,
                0,
                self.app.active_quip(),
                self.app.focus() == UiFocus::BuddyPane,
            ),
            _ => vec![
                "  .---.".to_string(),
                " (  ?  )".to_string(),
                "  `---'".to_string(),
                "Unhatched Buddy".to_string(),
            ],
        };
```

- [ ] **Step 4: Run the Buddy pane tests and verify they pass**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_pane
```

Expected:

- PASS with `4 passed; 0 failed`

- [ ] **Step 5: Commit the frame-aware Buddy pane change**

```bash
git add wrapper/src/ui/buddy_pane.rs wrapper/tests/buddy_pane.rs
git commit -m "Render buddy pane with animation frames"
```

### Task 3: Wire Animation State Into The Runtime Loop

**Files:**
- Modify: `wrapper/src/app/runtime.rs`
- Test: `wrapper/tests/buddy_animation.rs`
- Test: `wrapper/tests/buddy_pane.rs`

- [ ] **Step 1: Extend the animation tests with a reset regression for hatch/runtime wiring**

Append to `wrapper/tests/buddy_animation.rs`:

```rust
#[test]
fn reset_returns_to_rest_and_reschedules_idle_animation() {
    let now = Instant::now();
    let mut animation = BuddyAnimation::new(now);

    animation.tick(now + IDLE_QUIET_DELAY);
    assert_eq!(animation.mode(), BuddyAnimationMode::Idle);
    assert_eq!(animation.current_frame(), 1);

    let reset_at = now + IDLE_QUIET_DELAY + Duration::from_millis(25);
    animation.reset(reset_at);
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);

    animation.tick(reset_at + IDLE_QUIET_DELAY - Duration::from_millis(1));
    assert_eq!(animation.mode(), BuddyAnimationMode::None);
    assert_eq!(animation.current_frame(), 0);
}
```

- [ ] **Step 2: Run the animation test file and verify the new regression fails**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_animation
```

Expected:

- FAIL because the helper does not expose `reset` yet

- [ ] **Step 3: Integrate animation into the runtime**

Update the Buddy imports in `wrapper/src/app/runtime.rs`:

```rust
    buddy::{
        animation::BuddyAnimation,
        events::{BuddyEvent, BuddyEventKind},
        lifecycle::{apply_pet, can_rebirth_at, hatch_fallback},
        policy::{QuipPolicyConfig, can_attempt_long_run_quip},
        quips::sanitize_quip,
        roll::roll_with_seed,
        store::{BuddyStore, PersistedBuddy},
        summary::RollingSummary,
        types::CompanionBones,
    },
```

Add runtime state:

```rust
    animation: BuddyAnimation,
```

Initialize it in `Runtime::new`:

```rust
            animation: BuddyAnimation::new(Instant::now()),
```

Add `reset` to `wrapper/src/buddy/animation.rs` so hatch/rebirth can restart the quiet period cleanly:

```rust
    pub fn reset(&mut self, now: Instant) {
        *self = Self::new(now);
    }
```

Use the current animation frame in `render_buddy_text`:

```rust
        let frame = self.animation.current_frame();
        let mut lines = match (&self.buddy, &self.bones) {
            (Some(buddy), Some(bones)) if self.app.is_buddy_status_open() => {
                render_status_lines(buddy, bones, frame, Utc::now())
            }
            (Some(buddy), Some(bones)) => render_idle_lines(
                buddy,
                bones,
                frame,
                self.app.active_quip(),
                self.app.focus() == UiFocus::BuddyPane,
            ),
            _ => vec![
                "  .---.".to_string(),
                " (  ?  )".to_string(),
                "  `---'".to_string(),
                "Unhatched Buddy".to_string(),
            ],
        };
```

Start pet animation in the pet action branch:

```rust
            BuddyMenuAction::Pet => {
                let now = Instant::now();
                self.animation.start_pet(now);
                self.app
                    .set_last_pet_at_ms(Some(apply_pet(Utc::now().timestamp_millis())));
                self.status_message = Some("Buddy brightens a bit.".to_string());
            }
```

Reset animation when a Buddy is hatched or reborn:

```rust
                RuntimeEvent::HatchFinished { buddy, bones } => {
                    self.store.save_global(&buddy)?;
                    self.buddy = Some(*buddy);
                    self.bones = Some(*bones);
                    self.animation.reset(Instant::now());
                    self.app.set_has_buddy(true);
                    self.hatch_in_flight = false;
                    self.status_message = Some("Buddy is alive.".to_string());
                }
```

Advance animation at the top of `tick()`:

```rust
    fn tick(&mut self) {
        self.animation.tick(Instant::now());

        if let Some(set_at) = self.bubble_set_at
            && set_at.elapsed() >= QUIET_BUBBLE_LIFETIME
        {
            self.app.set_active_quip(None);
            self.bubble_set_at = None;
        }

        if self.quip_in_flight {
            return;
        }
```

- [ ] **Step 4: Run targeted tests, then the full wrapper suite**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_animation
cargo test --manifest-path wrapper/Cargo.toml --test buddy_pane
cargo test --manifest-path wrapper/Cargo.toml
```

Expected:

- `buddy_animation`: PASS with `4 passed; 0 failed`
- `buddy_pane`: PASS with `4 passed; 0 failed`
- full suite: PASS with `0 failed`

- [ ] **Step 5: Commit the runtime wiring**

```bash
git add wrapper/src/app/runtime.rs wrapper/tests/buddy_animation.rs wrapper/tests/buddy_pane.rs
git commit -m "Animate buddy idle and pet bursts"
```

### Task 4: Final Verification

**Files:**
- Modify: none

- [ ] **Step 1: Run format verification**

Run:

```bash
cargo fmt --manifest-path wrapper/Cargo.toml --check
```

Expected:

- PASS with no diff output

- [ ] **Step 2: Run lint verification**

Run:

```bash
cargo clippy --manifest-path wrapper/Cargo.toml --all-targets -- -D warnings
```

Expected:

- PASS with exit code `0`

- [ ] **Step 3: Manual smoke check**

Run:

```bash
cargo run --manifest-path wrapper/Cargo.toml
```

Expected:

- Buddy rests on frame `0`
- after a quiet pause, Buddy plays a short burst and returns to rest
- petting triggers an immediate faster burst and still shows `<3`
- animation never loops continuously
