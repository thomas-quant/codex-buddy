# Buddy Wrapper TUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust wrapper TUI that hosts stock Codex inside a PTY and adds an always-visible Buddy side pane with wrapper-owned persistence, hook-driven session context, and `codex exec`-generated hatch/quips.

**Architecture:** Implement a new Rust binary crate in `wrapper/` using `ratatui` + `crossterm` for the terminal UI, `portable-pty` + `vt100` for hosting the Codex PTY, and wrapper-owned Buddy modules for storage, rendering, lifecycle, summary, and quip policy. Launch Codex with a temp `CODEX_HOME` overlay that enables `codex_hooks` and injects wrapper-managed `hooks.json`; the running wrapper binary exposes a `hook-relay` subcommand that forwards raw hook payloads back into the live UI session over a Unix socket.

**Tech Stack:** Rust, ratatui, crossterm, tokio, portable-pty, vt100, serde, serde_json, toml, directories, fs4, chrono, clap, tempfile

**Reality check:** This snapshot is not a Git checkout, so the plan uses file/test checkpoints instead of mandatory commit steps. Once implementation moves into the real wrapper repo, add one commit per completed task.

---

## Planned File Structure

- `wrapper/Cargo.toml`
  Rust crate manifest and dependency list.
- `wrapper/src/main.rs`
  CLI entrypoint, `tokio` runtime bootstrap, and `hook-relay` subcommand dispatch.
- `wrapper/src/lib.rs`
  Public module exports for app, codex, buddy, ui, and util.
- `wrapper/src/app/mod.rs`
  Top-level app state, event loop, reducer, and integration wiring.
- `wrapper/src/app/event.rs`
  App-local event enum for keyboard, tick, PTY, hook, and quip events.
- `wrapper/src/ui/layout.rs`
  Ratatui layout split between PTY pane and Buddy pane.
- `wrapper/src/ui/pty_view.rs`
  Draws the vt100 screen buffer into ratatui widgets.
- `wrapper/src/ui/buddy_pane.rs`
  Draws the Buddy side pane, action menu, status view, and hint footer.
- `wrapper/src/buddy/types.rs`
  Core Buddy enums, structs, and serialized store types.
- `wrapper/src/buddy/roll.rs`
  Deterministic body generation from `hatch_seed`.
- `wrapper/src/buddy/sprites.rs`
  ASCII sprite frame data and sprite rendering helpers.
- `wrapper/src/buddy/store.rs`
  Persistent global state, in-memory session state, and file-locking writes.
- `wrapper/src/buddy/lifecycle.rs`
  Hatch, pet, mute/unmute, status, and rebirth business rules.
- `wrapper/src/buddy/events.rs`
  Normalized Buddy event model and event-specific adapters.
- `wrapper/src/buddy/summary.rs`
  Rolling per-session summary reducer.
- `wrapper/src/buddy/policy.rs`
  Quip timing gates, cooldowns, long-run logic, and blacklist checks.
- `wrapper/src/buddy/quips.rs`
  Quip sanitization, quip backend trait, and `codex exec` quip backend.
- `wrapper/src/codex/pty.rs`
  PTY process management, stdin forwarding, resize, and screen parsing.
- `wrapper/src/codex/home.rs`
  Temp `CODEX_HOME` overlay builder.
- `wrapper/src/codex/hooks.rs`
  `hooks.json` generation, raw hook payload structs, and socket relay protocol.
- `wrapper/src/codex/launch.rs`
  Stock Codex launch command assembly for the PTY session.
- `wrapper/src/codex/exec.rs`
  `codex exec` runner for hatch/quips with output-schema support.
- `wrapper/src/util/paths.rs`
  XDG/app-data paths and temp session paths.
- `wrapper/prompts/hatch.md`
  Hatch prompt template for `codex exec`.
- `wrapper/prompts/quip.md`
  Quip prompt template for `codex exec`.
- `wrapper/schemas/hatch.schema.json`
  JSON schema for hatch output.
- `wrapper/schemas/quip.schema.json`
  JSON schema for quip output.
- `wrapper/tests/*.rs`
  Unit and integration tests for roll logic, storage, focus/menu reducer, hook normalization, and quip policy.

## External References To Preserve

- `buddy/CLEAN_ROOM_SPEC.md`
  Deterministic roll rules, narrow/full rendering constraints, bubble timing, and pet burst behavior.
- `buddy/HOST_INTEGRATION_SPEC.md`
  Hatch/status/pet/mute/unmute semantics and observer behavior.
- `buddy/CODEX_WRAPPER_PORT_SPEC.md`
  Wrapper-specific architecture, multi-session model, and quip behavior.
- `buddy/sprites.ts`
  Canonical sprite frame content to transcribe into Rust.
- `buddy/companion.ts`
  Canonical Mulberry32/FNV-style deterministic roll flow to port into Rust.

## Platform Assumptions

- Target Unix-like systems first. Codex hooks are currently disabled on Windows according to the official hooks docs.
- Use `CODEX_HOME` to isolate wrapper-managed hook/config overlays from the user’s normal `~/.codex` directory.
- Use `codex exec --output-schema ... -o ... --ephemeral` for hatch/quips so the wrapper receives strict JSON instead of scraping free-form output.

### Task 1: Bootstrap The Rust Wrapper Crate

**Files:**
- Create: `wrapper/Cargo.toml`
- Create: `wrapper/src/main.rs`
- Create: `wrapper/src/lib.rs`
- Create: `wrapper/src/app/mod.rs`
- Create: `wrapper/src/app/event.rs`
- Test: `wrapper/tests/smoke_boot.rs`

- [ ] **Step 1: Create the crate and module directories**

Run:

```bash
cargo new wrapper --bin --vcs none
mkdir -p wrapper/src/{app,buddy,codex,ui,util} wrapper/prompts wrapper/schemas wrapper/tests
touch wrapper/src/lib.rs wrapper/src/app/event.rs
```

Expected: `wrapper/` exists with a compilable Cargo binary crate.

- [ ] **Step 2: Replace `wrapper/Cargo.toml` with the full dependency set**

```toml
[package]
name = "buddy-wrapper"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["clock", "serde"] }
clap = { version = "4.5", features = ["derive"] }
crossterm = "0.28"
directories = "6"
fs4 = "0.9"
portable-pty = "0.8"
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
tokio = { version = "1", features = ["fs", "io-util", "macros", "net", "rt-multi-thread", "sync", "time"] }
toml = "0.8"
unicode-width = "0.2"
uuid = { version = "1", features = ["serde", "v4"] }
vt100 = "0.15"

[dev-dependencies]
assert_cmd = "2"
pretty_assertions = "1"
tempfile = "3"
```

- [ ] **Step 3: Write the first failing smoke test**

Create `wrapper/tests/smoke_boot.rs`:

```rust
use buddy_wrapper::app::App;

#[test]
fn app_constructs_without_starting_codex() {
    let app = App::new_for_test();
    assert_eq!(app.focus_label(), "pty");
    assert!(!app.has_buddy());
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml app_constructs_without_starting_codex -- --exact
```

Expected: FAIL with unresolved imports or missing `App::new_for_test`.

- [ ] **Step 4: Add the minimal app/lib/bootstrap code to make the smoke test pass**

`wrapper/src/lib.rs`

```rust
pub mod app;
pub mod buddy;
pub mod codex;
pub mod ui;
pub mod util;
```

`wrapper/src/app/mod.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFocus {
    Pty,
    BuddyPane,
}

pub struct App {
    focus: UiFocus,
    has_buddy: bool,
}

impl App {
    pub fn new_for_test() -> Self {
        Self {
            focus: UiFocus::Pty,
            has_buddy: false,
        }
    }

    pub fn focus_label(&self) -> &'static str {
        match self.focus {
            UiFocus::Pty => "pty",
            UiFocus::BuddyPane => "buddy_pane",
        }
    }

    pub fn has_buddy(&self) -> bool {
        self.has_buddy
    }
}
```

`wrapper/src/main.rs`

```rust
fn main() -> anyhow::Result<()> {
    Ok(())
}
```

- [ ] **Step 5: Run the smoke test**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml app_constructs_without_starting_codex -- --exact
```

Expected: PASS.

### Task 2: Port Deterministic Buddy Types And Roll Logic

**Files:**
- Create: `wrapper/src/buddy/mod.rs`
- Create: `wrapper/src/buddy/types.rs`
- Create: `wrapper/src/buddy/roll.rs`
- Test: `wrapper/tests/roll_determinism.rs`

- [ ] **Step 1: Write failing determinism and invariant tests**

Create `wrapper/tests/roll_determinism.rs`:

```rust
use buddy_wrapper::buddy::roll::roll_with_seed;

#[test]
fn same_seed_produces_same_roll() {
    let first = roll_with_seed("alpha-seed");
    let second = roll_with_seed("alpha-seed");
    assert_eq!(first, second);
}

#[test]
fn common_rolls_never_wear_non_none_hats() {
    for i in 0..10_000 {
        let roll = roll_with_seed(&format!("seed-{i}"));
        if roll.bones.rarity == "common" {
            assert_eq!(roll.bones.hat, "none");
        }
    }
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test roll_determinism
```

Expected: FAIL because `buddy::roll` and the buddy types do not exist yet.

- [ ] **Step 2: Add the Buddy type definitions**

`wrapper/src/buddy/mod.rs`

```rust
pub mod roll;
pub mod types;
```

`wrapper/src/buddy/types.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const RARITIES: [&str; 5] = ["common", "uncommon", "rare", "epic", "legendary"];
pub const SPECIES: [&str; 18] = [
    "duck", "goose", "blob", "cat", "dragon", "octopus", "owl", "penguin", "turtle",
    "snail", "ghost", "axolotl", "capybara", "cactus", "robot", "rabbit", "mushroom", "chonk",
];
pub const EYES: [&str; 6] = ["·", "✦", "×", "◉", "@", "°"];
pub const HATS: [&str; 8] = ["none", "crown", "tophat", "propeller", "halo", "wizard", "beanie", "tinyduck"];
pub const STAT_NAMES: [&str; 5] = ["DEBUGGING", "PATIENCE", "CHAOS", "WISDOM", "SNARK"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionBones {
    pub rarity: String,
    pub species: String,
    pub eye: String,
    pub hat: String,
    pub shiny: bool,
    pub stats: BTreeMap<String, u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roll {
    pub bones: CompanionBones,
    pub inspiration_seed: u32,
}
```

- [ ] **Step 3: Implement the deterministic roll pipeline**

`wrapper/src/buddy/roll.rs`

```rust
use std::collections::BTreeMap;

use crate::buddy::types::{CompanionBones, Roll, EYES, HATS, RARITIES, SPECIES, STAT_NAMES};

const RARITY_WEIGHTS: [(&str, u32); 5] = [
    ("common", 60),
    ("uncommon", 25),
    ("rare", 10),
    ("epic", 4),
    ("legendary", 1),
];

fn fnv32(input: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for byte in input.as_bytes() {
        h ^= u32::from(*byte);
        h = h.wrapping_mul(16777619);
    }
    h
}

fn mulberry32(mut state: u32) -> impl FnMut() -> f64 {
    move || {
        state = state.wrapping_add(0x6d2b79f5);
        let mut t = state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from((t ^ (t >> 14)) >> 0) / 4294967296.0
    }
}

pub fn roll_with_seed(seed: &str) -> Roll {
    let mut rng = mulberry32(fnv32(seed));
    let rarity = roll_rarity(&mut rng).to_string();
    let hat = if rarity == "common" {
        "none".to_string()
    } else {
        pick(&mut rng, &HATS).to_string()
    };
    let bones = CompanionBones {
        rarity: rarity.clone(),
        species: pick(&mut rng, &SPECIES).to_string(),
        eye: pick(&mut rng, &EYES).to_string(),
        hat,
        shiny: rng() < 0.01,
        stats: roll_stats(&mut rng, &rarity),
    };
    Roll {
        bones,
        inspiration_seed: (rng() * 1_000_000_000.0).floor() as u32,
    }
}

fn pick<'a>(rng: &mut impl FnMut() -> f64, values: &'a [&str]) -> &'a str {
    let index = (rng() * values.len() as f64).floor() as usize;
    values[index.min(values.len() - 1)]
}

fn roll_rarity(rng: &mut impl FnMut() -> f64) -> &'static str {
    let total: u32 = RARITY_WEIGHTS.iter().map(|(_, weight)| *weight).sum();
    let mut roll = (rng() * total as f64) as i64;
    for (rarity, weight) in RARITY_WEIGHTS {
        roll -= i64::from(weight);
        if roll < 0 {
            return rarity;
        }
    }
    "common"
}

fn roll_stats(rng: &mut impl FnMut() -> f64, rarity: &str) -> BTreeMap<String, u8> {
    let floor = match rarity {
        "common" => 5,
        "uncommon" => 15,
        "rare" => 25,
        "epic" => 35,
        "legendary" => 50,
        _ => 5,
    };
    let peak = pick(rng, &STAT_NAMES);
    let mut dump = pick(rng, &STAT_NAMES);
    while dump == peak {
        dump = pick(rng, &STAT_NAMES);
    }

    STAT_NAMES
        .into_iter()
        .map(|name| {
            let value = if name == peak {
                (floor + 50 + (rng() * 30.0).floor() as i32).min(100)
            } else if name == dump {
                (floor - 10 + (rng() * 15.0).floor() as i32).max(1)
            } else {
                floor + (rng() * 40.0).floor() as i32
            };
            (name.to_string(), value as u8)
        })
        .collect()
}
```

- [ ] **Step 4: Run the deterministic-roll tests**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test roll_determinism
```

Expected: PASS.

### Task 3: Implement Wrapper-Owned Storage And Lifecycle Rules

**Files:**
- Create: `wrapper/src/util/mod.rs`
- Create: `wrapper/src/util/paths.rs`
- Create: `wrapper/src/buddy/store.rs`
- Create: `wrapper/src/buddy/lifecycle.rs`
- Test: `wrapper/tests/store_lifecycle.rs`

- [ ] **Step 1: Write failing storage and rebirth-cooldown tests**

Create `wrapper/tests/store_lifecycle.rs`:

```rust
use chrono::{Duration, Utc};
use tempfile::tempdir;

use buddy_wrapper::buddy::lifecycle::can_rebirth_at;
use buddy_wrapper::buddy::store::{BuddyStore, PersistedBuddy};
use buddy_wrapper::util::paths::StoragePaths;

#[test]
fn persisted_buddy_round_trips() {
    let dir = tempdir().unwrap();
    let paths = StoragePaths::for_test(dir.path());
    let store = BuddyStore::new(paths).unwrap();
    let buddy = PersistedBuddy::new_for_test("seed-1", "Mochi", "A chaotic little debugger.");
    store.save_global(&buddy).unwrap();
    assert_eq!(store.load_global().unwrap().unwrap(), buddy);
}

#[test]
fn rebirth_requires_fourteen_days() {
    let born = Utc::now();
    assert!(!can_rebirth_at(born, None, born + Duration::days(13)));
    assert!(can_rebirth_at(born, None, born + Duration::days(14)));
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test store_lifecycle
```

Expected: FAIL because storage and lifecycle modules do not exist yet.

- [ ] **Step 2: Add path helpers and persistent store types**

`wrapper/src/util/mod.rs`

```rust
pub mod paths;
```

`wrapper/src/util/paths.rs`

```rust
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub state_dir: PathBuf,
    pub global_buddy_file: PathBuf,
}

impl StoragePaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("dev", "openai", "buddy-wrapper")
            .context("unable to resolve project directories")?;
        let state_dir = dirs.state_dir().to_path_buf();
        Ok(Self {
            global_buddy_file: state_dir.join("buddy-state.json"),
            state_dir,
        })
    }

    pub fn for_test(root: &Path) -> Self {
        Self {
            state_dir: root.to_path_buf(),
            global_buddy_file: root.join("buddy-state.json"),
        }
    }
}
```

`wrapper/src/buddy/store.rs`

```rust
use std::{fs::{self, File, OpenOptions}, io::{Read, Write}};

use anyhow::Result;
use chrono::{DateTime, Utc};
use fs4::FileExt;
use serde::{Deserialize, Serialize};

use crate::util::paths::StoragePaths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedBuddy {
    pub hatch_seed: String,
    pub name: String,
    pub personality_paragraph: String,
    pub hatched_at: DateTime<Utc>,
    pub last_rebirth_at: Option<DateTime<Utc>>,
    pub muted: bool,
}

impl PersistedBuddy {
    pub fn new_for_test(seed: &str, name: &str, personality: &str) -> Self {
        Self {
            hatch_seed: seed.to_string(),
            name: name.to_string(),
            personality_paragraph: personality.to_string(),
            hatched_at: Utc::now(),
            last_rebirth_at: None,
            muted: false,
        }
    }
}

pub struct BuddyStore {
    paths: StoragePaths,
}

impl BuddyStore {
    pub fn new(paths: StoragePaths) -> Result<Self> {
        fs::create_dir_all(&paths.state_dir)?;
        Ok(Self { paths })
    }

    pub fn load_global(&self) -> Result<Option<PersistedBuddy>> {
        if !self.paths.global_buddy_file.exists() {
            return Ok(None);
        }
        let mut file = File::open(&self.paths.global_buddy_file)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(Some(serde_json::from_str(&buf)?))
    }

    pub fn save_global(&self, buddy: &PersistedBuddy) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.paths.global_buddy_file)?;
        file.lock_exclusive()?;
        file.write_all(serde_json::to_string_pretty(buddy)?.as_bytes())?;
        file.unlock()?;
        Ok(())
    }
}
```

- [ ] **Step 3: Add rebirth gating helpers**

`wrapper/src/buddy/lifecycle.rs`

```rust
use chrono::{DateTime, Duration, Utc};

pub fn can_rebirth_at(
    hatched_at: DateTime<Utc>,
    last_rebirth_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    let gate = last_rebirth_at.unwrap_or(hatched_at);
    now >= gate + Duration::days(14)
}
```

- [ ] **Step 4: Run the lifecycle tests**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test store_lifecycle
```

Expected: PASS.

### Task 4: Port Buddy Sprite Rendering And Pane View Rules

**Files:**
- Create: `wrapper/src/buddy/sprites.rs`
- Create: `wrapper/src/ui/buddy_pane.rs`
- Test: `wrapper/tests/buddy_pane.rs`

- [ ] **Step 1: Write failing Buddy-pane rendering tests**

Create `wrapper/tests/buddy_pane.rs`:

```rust
use buddy_wrapper::buddy::store::PersistedBuddy;
use buddy_wrapper::buddy::types::CompanionBones;
use buddy_wrapper::ui::buddy_pane::{render_idle_lines, render_status_lines};

#[test]
fn idle_view_hides_personality_text() {
    let lines = render_idle_lines(
        &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
        &CompanionBones::test_fixture(),
        None,
        false,
    );
    assert!(lines.iter().all(|line| !line.contains("observant little goblin")));
}

#[test]
fn status_view_shows_personality_text() {
    let lines = render_status_lines(
        &PersistedBuddy::new_for_test("seed", "Mochi", "An observant little goblin."),
        &CompanionBones::test_fixture(),
    );
    assert!(lines.iter().any(|line| line.contains("observant little goblin")));
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_pane
```

Expected: FAIL because pane renderer helpers do not exist.

- [ ] **Step 2: Port the sprite frames and face renderer**

Create `wrapper/src/buddy/sprites.rs` with these public helpers:

```rust
use crate::buddy::types::CompanionBones;

pub type Frames = [[&'static str; 5]; 3];

pub fn render_face(bones: &CompanionBones) -> String {
    format!("({} {})", bones.eye, bones.eye)
}

pub fn render_sprite_frame(bones: &CompanionBones, frame: usize) -> Vec<String> {
    let table = species_frames(&bones.species);
    table[frame % table.len()]
        .iter()
        .map(|line| line.replace("{E}", &bones.eye))
        .collect()
}

fn species_frames(species: &str) -> &'static Frames {
    match species {
        "duck" => &DUCK_FRAMES,
        "goose" => &GOOSE_FRAMES,
        "blob" => &BLOB_FRAMES,
        "cat" => &CAT_FRAMES,
        "dragon" => &DRAGON_FRAMES,
        "octopus" => &OCTOPUS_FRAMES,
        "owl" => &OWL_FRAMES,
        "penguin" => &PENGUIN_FRAMES,
        "turtle" => &TURTLE_FRAMES,
        "snail" => &SNAIL_FRAMES,
        "ghost" => &GHOST_FRAMES,
        "axolotl" => &AXOLOTL_FRAMES,
        "capybara" => &CAPYBARA_FRAMES,
        "cactus" => &CACTUS_FRAMES,
        "robot" => &ROBOT_FRAMES,
        "rabbit" => &RABBIT_FRAMES,
        "mushroom" => &MUSHROOM_FRAMES,
        "chonk" => &CHONK_FRAMES,
        other => panic!("unknown species: {other}"),
    }
}
```

Above these helpers, define `DUCK_FRAMES`, `GOOSE_FRAMES`, `BLOB_FRAMES`, and the
rest of the per-species `Frames` constants by transcribing the canonical 3-frame,
5-line tables from `buddy/sprites.ts`. Preserve the hat slot and exact line
widths from the clean-room source.

- [ ] **Step 3: Implement the Buddy pane render helpers**

Create `wrapper/src/ui/buddy_pane.rs`:

```rust
use crate::buddy::{roll::roll_with_seed, store::PersistedBuddy, types::CompanionBones};

pub fn render_idle_lines(
    buddy: &PersistedBuddy,
    bones: &CompanionBones,
    quip: Option<&str>,
    focused: bool,
) -> Vec<String> {
    let mut lines = crate::buddy::sprites::render_sprite_frame(bones, 0);
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

pub fn render_status_lines(buddy: &PersistedBuddy, bones: &CompanionBones) -> Vec<String> {
    vec![
        format!("{} the {}", buddy.name, bones.species),
        format!("{} companion", bones.rarity),
        buddy.personality_paragraph.clone(),
    ]
}
```

Also add a `CompanionBones::test_fixture()` helper in `wrapper/src/buddy/types.rs`:

```rust
impl CompanionBones {
    pub fn test_fixture() -> Self {
        Self {
            rarity: "rare".into(),
            species: "duck".into(),
            eye: "·".into(),
            hat: "none".into(),
            shiny: false,
            stats: [
                ("DEBUGGING".into(), 70),
                ("PATIENCE".into(), 50),
                ("CHAOS".into(), 25),
                ("WISDOM".into(), 55),
                ("SNARK".into(), 65),
            ]
            .into_iter()
            .collect(),
        }
    }
}
```

- [ ] **Step 4: Run the pane tests**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test buddy_pane
```

Expected: PASS.

### Task 5: Build The PTY Host And Terminal Screen Buffer

**Files:**
- Create: `wrapper/src/codex/mod.rs`
- Create: `wrapper/src/codex/pty.rs`
- Create: `wrapper/src/ui/pty_view.rs`
- Test: `wrapper/tests/pty_parser.rs`

- [ ] **Step 1: Write a failing PTY parser test using `/bin/sh` instead of Codex**

Create `wrapper/tests/pty_parser.rs`:

```rust
use std::time::Duration;

use buddy_wrapper::codex::pty::PtyHost;

#[test]
fn pty_host_captures_screen_output() {
    let mut host = PtyHost::spawn_for_test("/bin/sh", &["-lc", "printf 'hello'"], 80, 24).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    host.pump_output().unwrap();
    assert!(host.screen_text().contains("hello"));
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test pty_parser
```

Expected: FAIL because the PTY host module does not exist.

- [ ] **Step 2: Implement the PTY host**

`wrapper/src/codex/mod.rs`

```rust
pub mod exec;
pub mod hooks;
pub mod home;
pub mod launch;
pub mod pty;
```

`wrapper/src/codex/pty.rs`

```rust
use std::{io::{Read, Write}, sync::Arc};

use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use vt100::Parser;

pub struct PtyHost {
    parser: Parser,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl PtyHost {
    pub fn spawn_for_test(command: &str, args: &[&str], cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;
        let mut cmd = CommandBuilder::new(command);
        for arg in args {
            cmd.arg(arg);
        }
        pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        Ok(Self {
            parser: Parser::new(rows, cols, 0),
            reader,
            writer,
        })
    }

    pub fn pump_output(&mut self) -> Result<()> {
        let mut buf = [0_u8; 4096];
        let size = self.reader.read(&mut buf)?;
        self.parser.process(&buf[..size]);
        Ok(())
    }

    pub fn screen_text(&self) -> String {
        self.parser.screen().contents()
    }
}
```

- [ ] **Step 3: Run the PTY test**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test pty_parser
```

Expected: PASS.

### Task 6: Add The Wrapper TUI Shell, Focus Switching, And Buddy Menu

**Files:**
- Modify: `wrapper/src/app/mod.rs`
- Modify: `wrapper/src/app/event.rs`
- Create: `wrapper/src/ui/layout.rs`
- Test: `wrapper/tests/focus_menu.rs`

- [ ] **Step 1: Write failing reducer tests for focus and action-menu behavior**

Create `wrapper/tests/focus_menu.rs`:

```rust
use buddy_wrapper::app::{App, AppAction, UiFocus};

#[test]
fn tab_toggles_between_pty_and_buddy() {
    let mut app = App::new_for_test();
    app.apply(AppAction::ToggleFocus);
    assert_eq!(app.focus(), UiFocus::BuddyPane);
    app.apply(AppAction::ToggleFocus);
    assert_eq!(app.focus(), UiFocus::Pty);
}

#[test]
fn enter_opens_action_menu_only_when_buddy_is_focused() {
    let mut app = App::new_for_test();
    app.apply(AppAction::OpenBuddyMenu);
    assert!(!app.is_buddy_menu_open());
    app.apply(AppAction::ToggleFocus);
    app.apply(AppAction::OpenBuddyMenu);
    assert!(app.is_buddy_menu_open());
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test focus_menu
```

Expected: FAIL because `AppAction` and reducer methods do not exist yet.

- [ ] **Step 2: Implement the reducer and keybinding assumptions**

Use these concrete keybindings:

- `Tab` switches focus between PTY and Buddy pane
- `Enter` opens the Buddy action menu when Buddy is focused
- `Up` / `Down` move inside the action menu
- `Enter` activates the selected action
- `Esc` closes status view or the action menu

Update `wrapper/src/app/mod.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    ToggleFocus,
    OpenBuddyMenu,
    CloseBuddyMenu,
}

pub struct App {
    focus: UiFocus,
    has_buddy: bool,
    buddy_menu_open: bool,
}

impl App {
    pub fn apply(&mut self, action: AppAction) {
        match action {
            AppAction::ToggleFocus => {
                self.focus = match self.focus {
                    UiFocus::Pty => UiFocus::BuddyPane,
                    UiFocus::BuddyPane => UiFocus::Pty,
                };
            }
            AppAction::OpenBuddyMenu if self.focus == UiFocus::BuddyPane => {
                self.buddy_menu_open = true;
            }
            AppAction::CloseBuddyMenu => {
                self.buddy_menu_open = false;
            }
            _ => {}
        }
    }

    pub fn focus(&self) -> UiFocus {
        self.focus
    }

    pub fn is_buddy_menu_open(&self) -> bool {
        self.buddy_menu_open
    }
}
```

- [ ] **Step 3: Add the layout split and Buddy hint footer**

Create `wrapper/src/ui/layout.rs`:

```rust
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};

pub fn split_main_and_buddy(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(60), Constraint::Length(28)])
        .split(area);
    [chunks[0], chunks[1]]
}
```

The Buddy pane footer text should be fixed to:

```text
Tab: focus  Enter: actions
```

- [ ] **Step 4: Run the reducer tests**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test focus_menu
```

Expected: PASS.

### Task 7: Launch Stock Codex With A Temp `CODEX_HOME` Overlay And Hook Injection

**Files:**
- Create: `wrapper/src/codex/home.rs`
- Create: `wrapper/src/codex/hooks.rs`
- Create: `wrapper/src/codex/launch.rs`
- Modify: `wrapper/src/main.rs`
- Test: `wrapper/tests/codex_overlay.rs`

- [ ] **Step 1: Write a failing overlay-generation test**

Create `wrapper/tests/codex_overlay.rs`:

```rust
use tempfile::tempdir;

use buddy_wrapper::codex::home::build_codex_home_overlay;

#[test]
fn overlay_writes_config_and_hooks_json() {
    let dir = tempdir().unwrap();
    let overlay = build_codex_home_overlay(dir.path(), "/tmp/buddy-wrapper", "/tmp/buddy.sock").unwrap();
    assert!(overlay.config_toml.exists());
    assert!(overlay.hooks_json.exists());
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test codex_overlay
```

Expected: FAIL because overlay builder does not exist.

- [ ] **Step 2: Implement the overlay builder using documented `CODEX_HOME` and hook behavior**

`wrapper/src/codex/home.rs`

```rust
use std::{fs, path::{Path, PathBuf}};

use anyhow::Result;

pub struct CodexHomeOverlay {
    pub root: PathBuf,
    pub config_toml: PathBuf,
    pub hooks_json: PathBuf,
}

pub fn build_codex_home_overlay(
    root: &Path,
    wrapper_exe: &str,
    socket_path: &str,
) -> Result<CodexHomeOverlay> {
    fs::create_dir_all(root)?;
    let config_toml = root.join("config.toml");
    let hooks_json = root.join("hooks.json");
    fs::write(
        &config_toml,
        r#"[features]
codex_hooks = true

[history]
persistence = "none"
"#,
    )?;
    fs::write(&hooks_json, crate::codex::hooks::render_hooks_json(wrapper_exe, socket_path))?;
    Ok(CodexHomeOverlay {
        root: root.to_path_buf(),
        config_toml,
        hooks_json,
    })
}
```

`wrapper/src/codex/hooks.rs`

```rust
pub fn render_hooks_json(wrapper_exe: &str, socket_path: &str) -> String {
    format!(
        r#"{{
  "hooks": {{
    "SessionStart": [{{ "matcher": "startup|resume", "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "UserPromptSubmit": [{{ "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "PreToolUse": [{{ "matcher": "Bash", "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "PostToolUse": [{{ "matcher": "Bash", "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}],
    "Stop": [{{ "hooks": [{{ "type": "command", "command": "{wrapper_exe} hook-relay --socket {socket_path}" }}] }}]
  }}
}}"#
    )
}
```

- [ ] **Step 3: Implement the Codex PTY launch command builder**

Create `wrapper/src/codex/launch.rs`:

```rust
use std::{collections::BTreeMap, path::Path};

pub struct CodexLaunch {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

pub fn build_codex_launch(cwd: &Path, codex_home: &Path) -> CodexLaunch {
    let mut env = BTreeMap::new();
    env.insert("CODEX_HOME".into(), codex_home.display().to_string());
    CodexLaunch {
        command: "codex".into(),
        args: vec!["--enable".into(), "codex_hooks".into(), "--cd".into(), cwd.display().to_string()],
        env,
    }
}
```

- [ ] **Step 4: Add a `hook-relay` subcommand stub to `main.rs`**

Update `wrapper/src/main.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    HookRelay {
        #[arg(long)]
        socket: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::HookRelay { socket }) => {
            let _ = socket;
            Ok(())
        }
        None => Ok(()),
    }
}
```

- [ ] **Step 5: Run the overlay test**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test codex_overlay
```

Expected: PASS.

### Task 8: Implement Hook Relay, Event Normalization, And Rolling Summaries

**Files:**
- Create: `wrapper/src/buddy/events.rs`
- Create: `wrapper/src/buddy/summary.rs`
- Modify: `wrapper/src/codex/hooks.rs`
- Modify: `wrapper/src/main.rs`
- Test: `wrapper/tests/event_normalizer.rs`

- [ ] **Step 1: Write failing event-normalizer tests**

Create `wrapper/tests/event_normalizer.rs`:

```rust
use buddy_wrapper::buddy::events::{normalize_hook_event, BuddyEventKind};
use serde_json::json;

#[test]
fn user_prompt_submit_normalizes_to_user_turn_submitted() {
    let raw = json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": "s1",
        "turn_id": "t1",
        "cwd": "/tmp/project",
        "prompt": "fix the failing test"
    });
    let normalized = normalize_hook_event(&raw).unwrap();
    assert_eq!(normalized.kind, BuddyEventKind::UserTurnSubmitted);
}

#[test]
fn post_tool_use_normalizes_tool_name_and_result() {
    let raw = json!({
        "hook_event_name": "PostToolUse",
        "session_id": "s1",
        "turn_id": "t1",
        "cwd": "/tmp/project",
        "tool_name": "Bash",
        "tool_input": { "command": "cargo test" },
        "tool_response": "{\"exit_code\":1}"
    });
    let normalized = normalize_hook_event(&raw).unwrap();
    assert_eq!(normalized.kind, BuddyEventKind::ToolFinished);
    assert_eq!(normalized.tool_name.as_deref(), Some("Bash"));
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test event_normalizer
```

Expected: FAIL because the event normalizer does not exist.

- [ ] **Step 2: Define the normalized event model**

Create `wrapper/src/buddy/events.rs`:

```rust
use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuddyEventKind {
    SessionStarted,
    UserTurnSubmitted,
    ToolStarted,
    ToolFinished,
    TurnCompleted,
    SessionEnded,
}

#[derive(Debug, Clone)]
pub struct BuddyEvent {
    pub kind: BuddyEventKind,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub cwd: String,
    pub tool_name: Option<String>,
    pub assistant_excerpt: Option<String>,
    pub user_excerpt: Option<String>,
}

pub fn normalize_hook_event(raw: &Value) -> Result<BuddyEvent> {
    let event_name = raw["hook_event_name"].as_str().ok_or_else(|| anyhow!("missing hook_event_name"))?;
    let kind = match event_name {
        "SessionStart" => BuddyEventKind::SessionStarted,
        "UserPromptSubmit" => BuddyEventKind::UserTurnSubmitted,
        "PreToolUse" => BuddyEventKind::ToolStarted,
        "PostToolUse" => BuddyEventKind::ToolFinished,
        "Stop" => BuddyEventKind::TurnCompleted,
        other => return Err(anyhow!("unsupported hook event: {other}")),
    };

    Ok(BuddyEvent {
        kind,
        session_id: raw["session_id"].as_str().unwrap_or_default().to_string(),
        turn_id: raw["turn_id"].as_str().map(ToString::to_string),
        cwd: raw["cwd"].as_str().unwrap_or_default().to_string(),
        tool_name: raw["tool_name"].as_str().map(ToString::to_string),
        assistant_excerpt: raw["last_assistant_message"].as_str().map(ToString::to_string),
        user_excerpt: raw["prompt"].as_str().map(ToString::to_string),
    })
}
```

- [ ] **Step 3: Add the rolling summary reducer**

Create `wrapper/src/buddy/summary.rs`:

```rust
use crate::buddy::events::{BuddyEvent, BuddyEventKind};

#[derive(Debug, Clone, Default)]
pub struct RollingSummary {
    pub current_task: Option<String>,
    pub last_status: Option<String>,
    pub notable_files: Vec<String>,
    pub unresolved_issue: Option<String>,
}

impl RollingSummary {
    pub fn apply(&mut self, event: &BuddyEvent) {
        match event.kind {
            BuddyEventKind::UserTurnSubmitted => {
                self.current_task = event.user_excerpt.clone();
            }
            BuddyEventKind::ToolFinished => {
                self.last_status = Some(format!("tool {} finished", event.tool_name.as_deref().unwrap_or("unknown")));
            }
            BuddyEventKind::TurnCompleted => {
                if self.unresolved_issue.is_none() {
                    self.unresolved_issue = event.assistant_excerpt.clone();
                }
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 4: Implement the real `hook-relay` command**

Update the `HookRelay` branch in `wrapper/src/main.rs` so it:

```rust
use tokio::{io::AsyncReadExt, net::UnixStream};

let mut stdin = String::new();
tokio::io::stdin().read_to_string(&mut stdin).await?;
let mut stream = UnixStream::connect(socket).await?;
stream.writable().await?;
stream.try_write(stdin.as_bytes())?;
```

Also update `wrapper/src/codex/hooks.rs` to expose a newline-delimited socket protocol:

```rust
use serde_json::Value;

pub fn parse_hook_payload(bytes: &[u8]) -> anyhow::Result<Value> {
    Ok(serde_json::from_slice(bytes)?)
}
```

- [ ] **Step 5: Run the event-normalizer test**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test event_normalizer
```

Expected: PASS.

### Task 9: Implement Hatch/Status/Pet/Mute/Unmute/Rebirth Actions

**Files:**
- Modify: `wrapper/src/buddy/lifecycle.rs`
- Create: `wrapper/src/codex/exec.rs`
- Create: `wrapper/prompts/hatch.md`
- Create: `wrapper/schemas/hatch.schema.json`
- Test: `wrapper/tests/hatch_actions.rs`

- [ ] **Step 1: Write failing hatch-action and fallback tests**

Create `wrapper/tests/hatch_actions.rs`:

```rust
use buddy_wrapper::buddy::lifecycle::{apply_pet, hatch_fallback};

#[test]
fn hatch_fallback_produces_name_and_personality() {
    let soul = hatch_fallback("seed-123", "rare", "duck");
    assert!(!soul.name.is_empty());
    assert!(soul.personality_paragraph.len() > 20);
}

#[test]
fn pet_action_sets_timestamp() {
    let ts = apply_pet(1_700_000_000_000);
    assert_eq!(ts, 1_700_000_000_000);
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test hatch_actions
```

Expected: FAIL because hatch fallback and pet action helpers do not exist.

- [ ] **Step 2: Add the hatch schema and prompt**

Create `wrapper/schemas/hatch.schema.json`:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["name", "personality_paragraph"],
  "properties": {
    "name": { "type": "string", "minLength": 1, "maxLength": 24 },
    "personality_paragraph": { "type": "string", "minLength": 20, "maxLength": 500 }
  }
}
```

Create `wrapper/prompts/hatch.md`:

```md
You are generating the canonical soul for a terminal companion.

Return strict JSON that matches the provided schema.

Inputs:
- hatch seed
- rarity
- species
- eye glyph
- hat
- shiny flag
- stats

Rules:
- do not mention repositories, cwd, accounts, or the current task
- create a short memorable name
- create exactly one personality paragraph
- the paragraph should be warm, quirky, and useful for future coding-context quips
```

- [ ] **Step 3: Implement `codex exec` hatch command assembly**

Create `wrapper/src/codex/exec.rs`:

```rust
use std::{path::Path, process::Command};

pub fn build_hatch_command(
    prompt: &str,
    schema_path: &Path,
    output_path: &Path,
) -> Command {
    let mut cmd = Command::new("codex");
    cmd.arg("exec")
        .arg("--ephemeral")
        .arg("--skip-git-repo-check")
        .arg("--output-schema").arg(schema_path)
        .arg("-o").arg(output_path)
        .arg("-m").arg("gpt-5.4-mini")
        .arg("-c").arg("model_reasoning_effort=\"medium\"")
        .arg(prompt);
    cmd
}
```

- [ ] **Step 4: Implement hatch fallback and the pane actions**

Extend `wrapper/src/buddy/lifecycle.rs`:

```rust
#[derive(Debug, Clone)]
pub struct HatchSoul {
    pub name: String,
    pub personality_paragraph: String,
}

pub fn hatch_fallback(seed: &str, rarity: &str, species: &str) -> HatchSoul {
    let name = format!("{}{}", &species[..1].to_uppercase(), &seed[..4.min(seed.len())]);
    let personality_paragraph = format!(
        "{name} is a {rarity} little {species} with a sharp eye for broken edges, a habit of hovering around half-finished fixes, and a tendency to react with affectionate commentary instead of silence."
    );
    HatchSoul { name, personality_paragraph }
}

pub fn apply_pet(now_ms: i64) -> i64 {
    now_ms
}
```

- [ ] **Step 5: Run the hatch tests**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test hatch_actions
```

Expected: PASS.

### Task 10: Implement Quip Backend, Policy Gates, And Sanitization

**Files:**
- Create: `wrapper/src/buddy/policy.rs`
- Create: `wrapper/src/buddy/quips.rs`
- Create: `wrapper/prompts/quip.md`
- Create: `wrapper/schemas/quip.schema.json`
- Test: `wrapper/tests/quip_policy.rs`

- [ ] **Step 1: Write failing quip-policy tests**

Create `wrapper/tests/quip_policy.rs`:

```rust
use chrono::{Duration, Utc};

use buddy_wrapper::buddy::policy::{can_attempt_long_run_quip, sanitize_quip, QuipPolicyConfig};

#[test]
fn quip_text_is_single_line_and_capped() {
    let text = sanitize_quip("hello\nworld ".repeat(20).as_str()).unwrap();
    assert!(!text.contains('\n'));
    assert!(text.chars().count() <= 80);
}

#[test]
fn long_run_quip_requires_twenty_minutes_and_only_fires_once() {
    let cfg = QuipPolicyConfig::default();
    let started = Utc::now();
    assert!(!can_attempt_long_run_quip(started, started + Duration::minutes(19), false, &cfg));
    assert!(can_attempt_long_run_quip(started, started + Duration::minutes(20), false, &cfg));
    assert!(!can_attempt_long_run_quip(started, started + Duration::minutes(21), true, &cfg));
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test quip_policy
```

Expected: FAIL because the policy module does not exist yet.

- [ ] **Step 2: Add quip schema and prompt**

Create `wrapper/schemas/quip.schema.json`:

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["emit"],
  "properties": {
    "emit": { "type": "boolean" },
    "text": { "type": "string", "maxLength": 120 },
    "tone": {
      "type": "string",
      "enum": ["pleased", "amused", "concerned", "sleepy", "impressed", "neutral"]
    }
  }
}
```

Create `wrapper/prompts/quip.md`:

```md
Return strict JSON matching the provided schema.

You are generating one short Buddy quip for a terminal coding companion.

Inputs:
- buddy name
- buddy personality paragraph
- current event type
- session cwd
- rolling session summary
- recent turn digest
- optional short raw excerpts

Rules:
- emit at most one short line
- max 80 visible characters after sanitization
- do not mock, moralize, or restate raw secrets
- it is acceptable to return {"emit": false}
```

- [ ] **Step 3: Implement policy config, gates, blacklist, and sanitization**

Create `wrapper/src/buddy/policy.rs`:

```rust
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct QuipPolicyConfig {
    pub cooldown: Duration,
    pub long_run_threshold: Duration,
}

impl Default for QuipPolicyConfig {
    fn default() -> Self {
        Self {
            cooldown: Duration::minutes(10),
            long_run_threshold: Duration::minutes(20),
        }
    }
}

pub fn can_attempt_long_run_quip(
    started_at: DateTime<Utc>,
    now: DateTime<Utc>,
    already_fired: bool,
    cfg: &QuipPolicyConfig,
) -> bool {
    !already_fired && now >= started_at + cfg.long_run_threshold
}
```

Create `wrapper/src/buddy/quips.rs`:

```rust
pub fn sanitize_quip(raw: &str) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.chars().take(80).collect())
}
```

- [ ] **Step 4: Run the quip-policy tests**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test quip_policy
```

Expected: PASS.

### Task 11: Integrate The Running App And Verify Degraded-Mode Behavior

**Files:**
- Modify: `wrapper/src/app/mod.rs`
- Modify: `wrapper/src/codex/pty.rs`
- Modify: `wrapper/src/codex/launch.rs`
- Modify: `wrapper/src/ui/buddy_pane.rs`
- Test: `wrapper/tests/degraded_mode.rs`

- [ ] **Step 1: Write failing degraded-mode tests**

Create `wrapper/tests/degraded_mode.rs`:

```rust
use buddy_wrapper::app::App;

#[test]
fn quip_failure_clears_the_active_bubble() {
    let mut app = App::new_for_test();
    app.set_active_quip_for_test("hello");
    app.handle_quip_failure();
    assert_eq!(app.active_quip(), None);
}
```

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml --test degraded_mode
```

Expected: FAIL because `App` does not yet expose quip-failure helpers.

- [ ] **Step 2: Wire the live app together**

Expand `wrapper/src/app/mod.rs` so `App` owns:

```rust
pub struct App {
    focus: UiFocus,
    has_buddy: bool,
    buddy_menu_open: bool,
    active_quip: Option<String>,
    last_pet_at_ms: Option<i64>,
}
```

The live app loop should:

- tick every 500 ms for sprite animation
- poll PTY output and update the vt100 parser
- poll the Unix socket for hook payloads
- normalize hook payloads into `BuddyEvent`
- update the rolling summary before evaluating `TurnCompleted` quips
- drop malformed hatch/quip JSON without crashing

Also add these test-facing helpers:

```rust
impl App {
    pub fn set_active_quip_for_test(&mut self, value: &str) {
        self.active_quip = Some(value.to_string());
    }

    pub fn handle_quip_failure(&mut self) {
        self.active_quip = None;
    }

    pub fn active_quip(&self) -> Option<&str> {
        self.active_quip.as_deref()
    }
}
```

- [ ] **Step 3: Run the full automated suite**

Run:

```bash
cargo test --manifest-path wrapper/Cargo.toml
cargo fmt --manifest-path wrapper/Cargo.toml --check
cargo clippy --manifest-path wrapper/Cargo.toml --all-targets -- -D warnings
```

Expected:

- all tests PASS
- `cargo fmt --check` exits `0`
- `cargo clippy` exits `0`

- [ ] **Step 4: Perform the manual wrapper verification loop**

Run:

```bash
cargo run --manifest-path wrapper/Cargo.toml
```

Verify manually:

1. Codex renders in the main PTY pane and remains usable.
2. The Buddy side pane is always visible.
3. `Tab` moves focus into the Buddy pane and back.
4. `Enter` opens the Buddy action menu when the Buddy pane is focused.
5. Hatch creates a Buddy and persists it across wrapper restarts.
6. `status` reveals the full personality paragraph in-pane.
7. `pet` shows the heart burst.
8. `mute` suppresses visual quips and `unmute` restores them.
9. Running a short Codex turn produces normalized hook traffic without crashing the app.
10. A malformed or unavailable `codex exec` quip call results in no bubble, not a crash.

## Self-Review Checklist

- Spec coverage:
  - Wrapper PTY host: Tasks 5, 7, 11
  - Always-visible focusable Buddy side pane: Tasks 4, 6, 11
  - Wrapper-owned persistence and `hatch_seed`: Tasks 2, 3, 9
  - Hatch/rebirth lifecycle and silent fallback: Tasks 3, 9
  - Hook normalization and rolling summary: Task 8
  - `codex exec` hatch/quip backends: Tasks 9, 10
  - Quip cooldowns, long-run gate, and sanitization: Task 10
  - Degraded-mode behavior: Task 11
- Placeholder scan:
  - No `TBD` or “implement later” placeholders remain.
  - The only transcription step is the canonical sprite table port from `buddy/sprites.ts`, which is an explicit reference source already present in this repo.
- Type consistency:
  - `PersistedBuddy` stores `hatch_seed`, `name`, `personality_paragraph`, `hatched_at`, `last_rebirth_at`, and `muted`.
  - `BuddyEventKind` names match the wrapper spec.
  - Quip tone enum matches the wrapper spec.
