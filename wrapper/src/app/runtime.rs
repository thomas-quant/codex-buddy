use std::{
    fs,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
    },
    execute,
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect, widgets::Clear};
use serde_json::json;
use tempfile::Builder;
use uuid::Uuid;

use crate::{
    app::pty_input::{encode_key_for_pty, encode_mouse_for_pty},
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
    codex::{
        exec::{QuipRequest, generate_hatch_soul, generate_quip},
        home::{build_codex_home_overlay, resolve_base_codex_home},
        launch::build_codex_launch,
        pty::PtyHost,
        session::{SessionEvent, SessionEventNormalizer},
    },
    ui::{
        buddy_pane::{
            BuddyMenuEntry, render_action_menu_lines, render_buddy_widget, render_idle_lines,
            render_status_lines,
        },
        layout::{BUDDY_HINT_FOOTER, split_main_and_buddy},
        pty_view::{PtyRenderFilter, PtyView},
    },
    util::paths::{StoragePaths, resolve_codex_session_root},
};

use super::{App, AppAction, UiFocus};

const QUIET_BUBBLE_LIFETIME: Duration = Duration::from_secs(15);
const PET_BURST_LIFETIME_MS: i64 = 1_500;

pub fn run_default() -> Result<()> {
    let cwd = std::env::current_dir().context("failed to resolve current working directory")?;
    let wrapper_exe =
        std::env::current_exe().context("failed to resolve wrapper executable path")?;
    run(RuntimeOptions { cwd, wrapper_exe })
}

struct RuntimeOptions {
    cwd: PathBuf,
    wrapper_exe: PathBuf,
}

enum RuntimeEvent {
    BuddyEvent(BuddyEvent),
    Commentary(String),
    HatchFinished {
        buddy: Box<PersistedBuddy>,
        bones: Box<CompanionBones>,
    },
    QuipFinished(Option<String>),
    QuipFailed(String),
}

#[derive(Clone)]
struct ActiveToolPhase {
    started_at: DateTime<Utc>,
    tool_name: Option<String>,
    long_run_quip_fired: bool,
}

struct Runtime {
    app: App,
    pty: PtyHost,
    pty_view: PtyView,
    store: BuddyStore,
    buddy: Option<PersistedBuddy>,
    bones: Option<CompanionBones>,
    summary: RollingSummary,
    recent_turns: Vec<String>,
    active_tool_phase: Option<ActiveToolPhase>,
    animation: BuddyAnimation,
    session_id: String,
    cwd: PathBuf,
    events_rx: Receiver<RuntimeEvent>,
    events_tx: Sender<RuntimeEvent>,
    quip_policy: QuipPolicyConfig,
    quip_in_flight: bool,
    hatch_in_flight: bool,
    last_quip_at: Option<DateTime<Utc>>,
    bubble_set_at: Option<Instant>,
    hidden_commentary: Vec<String>,
    status_message: Option<String>,
    session_monitor_thread: thread::JoinHandle<()>,
    _session_dir: tempfile::TempDir,
}

impl Runtime {
    fn new(opts: RuntimeOptions) -> Result<Self> {
        let mut app = App::new_for_test();
        let storage_paths = StoragePaths::discover()?;
        let base_codex_home = resolve_base_codex_home()?;
        let session_root = resolve_codex_session_root(&storage_paths, &base_codex_home)?;
        let store = BuddyStore::new(storage_paths)?;
        let buddy = store.load_global()?;
        let bones = buddy
            .as_ref()
            .map(|persisted| roll_with_seed(&persisted.hatch_seed).bones);
        app.set_has_buddy(buddy.is_some());

        fs::create_dir_all(&session_root).context("failed to create wrapper session root")?;
        let session_dir = Builder::new()
            .prefix("buddy-wrapper-session-")
            .tempdir_in(&session_root)
            .context("failed to create session directory")?;
        let codex_home = session_dir.path().join("codex-home");
        let wrapper_exe = opts.wrapper_exe.display().to_string();
        let socket_display = session_dir.path().join("buddy.sock").display().to_string();
        build_codex_home_overlay(&base_codex_home, &codex_home, &wrapper_exe, &socket_display)?;

        let (terminal_cols, terminal_rows) = terminal::size()?;
        let main_rect = main_pane_rect(terminal_cols, terminal_rows);
        let launch = build_codex_launch(&opts.cwd, &codex_home);
        let pty = PtyHost::spawn(
            &launch.command,
            &launch.args,
            &launch.env,
            main_rect.width.max(2),
            main_rect.height.max(2),
        )?;

        let (events_tx, events_rx) = mpsc::channel();
        let session_monitor_thread =
            spawn_session_monitor(codex_home.join("sessions"), events_tx.clone());

        Ok(Self {
            app,
            pty,
            pty_view: PtyView::new(),
            store,
            buddy,
            bones,
            summary: RollingSummary::default(),
            recent_turns: Vec::new(),
            active_tool_phase: None,
            animation: BuddyAnimation::new(Instant::now()),
            session_id: Uuid::new_v4().to_string(),
            cwd: opts.cwd,
            events_rx,
            events_tx,
            quip_policy: QuipPolicyConfig::default(),
            quip_in_flight: false,
            hatch_in_flight: false,
            last_quip_at: None,
            bubble_set_at: None,
            hidden_commentary: Vec::new(),
            status_message: None,
            session_monitor_thread,
            _session_dir: session_dir,
        })
    }

    fn run(mut self) -> Result<()> {
        let mut terminal = TerminalSession::enter()?;
        let mut previous_size = terminal::size()?;

        loop {
            self.drain_runtime_events()?;
            self.pty.pump_output()?;
            self.tick();

            terminal.draw(|frame| self.draw(frame))?;

            if let Some(exit_status) = self.pty.try_wait()? {
                self.handle_session_end(exit_status.success());
                if exit_status.success() {
                    break;
                }
                self.status_message = Some(format!("Codex exited: {exit_status}"));
                break;
            }

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                        if self.handle_key(key)? {
                            break;
                        }
                    }
                    CrosstermEvent::Paste(text) => {
                        if self.app.focus() == UiFocus::Pty {
                            self.pty.scroll_to_bottom();
                            self.pty.write_all(text.as_bytes())?;
                        }
                    }
                    CrosstermEvent::Mouse(mouse) => {
                        self.handle_mouse(mouse, Rect::new(0, 0, previous_size.0, previous_size.1))?
                    }
                    CrosstermEvent::Resize(cols, rows) => {
                        let main_rect = main_pane_rect(cols, rows);
                        self.pty
                            .resize(main_rect.width.max(2), main_rect.height.max(2))?;
                        previous_size = (cols, rows);
                    }
                    _ => {}
                }
            } else {
                let (cols, rows) = terminal::size()?;
                if (cols, rows) != previous_size {
                    let main_rect = main_pane_rect(cols, rows);
                    self.pty
                        .resize(main_rect.width.max(2), main_rect.height.max(2))?;
                    previous_size = (cols, rows);
                }
            }
        }

        let _ = &self.session_monitor_thread;
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame<'_>) {
        let [main_rect, buddy_rect] = split_main_and_buddy(frame.area());
        let filter = PtyRenderFilter::new(self.hidden_commentary.iter().map(String::as_str));
        frame.render_widget(self.pty_view.render(self.pty.screen(), filter), main_rect);

        frame.render_widget(Clear, buddy_rect);
        frame.render_widget(render_buddy_widget(self.render_buddy_text()), buddy_rect);
    }

    fn render_buddy_text(&self) -> String {
        let frame = self.animation.current_frame();
        if self.app.is_buddy_menu_open() {
            let items = self.menu_items();
            let entries = items
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    BuddyMenuEntry::new(
                        &item.label,
                        idx == self.selected_menu_index(),
                        item.enabled,
                    )
                })
                .collect::<Vec<_>>();
            return render_action_menu_lines(&entries, self.status_message.as_deref()).join("\n");
        }

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

        if let Some(last_pet_at_ms) = self.app.last_pet_at_ms() {
            let now_ms = Utc::now().timestamp_millis();
            if now_ms - last_pet_at_ms <= PET_BURST_LIFETIME_MS {
                lines.push("<3".to_string());
            }
        }

        if let Some(status_message) = &self.status_message {
            lines.push(String::new());
            lines.push(status_message.clone());
        }

        if self.app.focus() == UiFocus::BuddyPane {
            lines.push(String::new());
            lines.push(BUDDY_HINT_FOOTER.to_string());
        }

        if self.app.is_buddy_status_open() {
            lines.push(String::new());
            lines.push("Esc: back".to_string());
        }

        lines.join("\n")
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            return Ok(true);
        }

        match self.app.focus() {
            UiFocus::Pty => self.handle_pty_key(key),
            UiFocus::BuddyPane => self.handle_buddy_key(key),
        }
    }

    fn handle_pty_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Tab => {
                self.app.apply(AppAction::ToggleFocus);
                Ok(false)
            }
            KeyCode::PageUp => {
                self.pty.scroll_up(page_scroll_rows(self.pty.screen()));
                Ok(false)
            }
            KeyCode::PageDown => {
                self.pty.scroll_down(page_scroll_rows(self.pty.screen()));
                Ok(false)
            }
            _ => {
                if let Some(bytes) = encode_key_for_pty(key) {
                    self.pty.scroll_to_bottom();
                    self.pty.write_all(&bytes)?;
                }
                Ok(false)
            }
        }
    }

    fn handle_buddy_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Tab => self.app.apply(AppAction::ToggleFocus),
            KeyCode::Esc => {
                if self.app.is_buddy_menu_open() {
                    self.app.apply(AppAction::CloseBuddyMenu);
                } else if self.app.is_buddy_status_open() {
                    self.app.apply(AppAction::CloseBuddyStatus);
                } else {
                    self.app.apply(AppAction::ToggleFocus);
                }
            }
            KeyCode::Enter => {
                if self.app.is_buddy_menu_open() {
                    self.activate_menu_item()?;
                } else if self.app.is_buddy_status_open() {
                    self.app.apply(AppAction::CloseBuddyStatus);
                } else {
                    self.app.apply(AppAction::OpenBuddyMenu);
                }
            }
            KeyCode::Up | KeyCode::Char('k') if self.app.is_buddy_menu_open() => {
                self.app.apply(AppAction::PrevBuddyAction);
            }
            KeyCode::Down | KeyCode::Char('j') if self.app.is_buddy_menu_open() => {
                self.app.apply(AppAction::NextBuddyAction);
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Result<()> {
        if self.app.focus() != UiFocus::Pty {
            return Ok(());
        }

        let [main_rect, buddy_rect] = split_main_and_buddy(area);
        if rect_contains(buddy_rect, mouse.column, mouse.row) {
            return Ok(());
        }

        if !rect_contains(main_rect, mouse.column, mouse.row) {
            return Ok(());
        }

        if let Some(bytes) = encode_mouse_for_pty(mouse, main_rect, self.pty.screen()) {
            self.pty.scroll_to_bottom();
            self.pty.write_all(&bytes)?;
        } else {
            match mouse.kind {
                crossterm::event::MouseEventKind::ScrollUp => self.pty.scroll_up(3),
                crossterm::event::MouseEventKind::ScrollDown => self.pty.scroll_down(3),
                _ => {}
            }
        }

        Ok(())
    }

    fn activate_menu_item(&mut self) -> Result<()> {
        let Some(item) = self.menu_items().get(self.selected_menu_index()).cloned() else {
            return Ok(());
        };

        if !item.enabled {
            self.status_message = Some(format!("{} is not available yet.", item.label));
            return Ok(());
        }

        match item.action {
            BuddyMenuAction::Hatch => self.spawn_hatch(false),
            BuddyMenuAction::Status => self.app.apply(AppAction::OpenBuddyStatus),
            BuddyMenuAction::Pet => {
                self.animation.start_pet(Instant::now());
                self.app
                    .set_last_pet_at_ms(Some(apply_pet(Utc::now().timestamp_millis())));
                self.status_message = Some("Buddy brightens a bit.".to_string());
            }
            BuddyMenuAction::Mute => {
                if let Some(mut buddy) = self.buddy.clone() {
                    buddy.muted = true;
                    self.store.save_global(&buddy)?;
                    self.buddy = Some(buddy);
                    self.status_message = Some("Buddy is muted.".to_string());
                }
            }
            BuddyMenuAction::Unmute => {
                if let Some(mut buddy) = self.buddy.clone() {
                    buddy.muted = false;
                    self.store.save_global(&buddy)?;
                    self.buddy = Some(buddy);
                    self.status_message = Some("Buddy is listening again.".to_string());
                }
            }
            BuddyMenuAction::Rebirth => self.spawn_hatch(true),
        }

        if !matches!(item.action, BuddyMenuAction::Status) {
            self.app.apply(AppAction::CloseBuddyMenu);
        }

        Ok(())
    }

    fn spawn_hatch(&mut self, rebirth: bool) {
        if self.hatch_in_flight {
            return;
        }

        self.hatch_in_flight = true;
        self.status_message = Some(if rebirth {
            "Rebirthing Buddy...".to_string()
        } else {
            "Hatching Buddy...".to_string()
        });

        let seed = Uuid::new_v4().to_string();
        let bones = roll_with_seed(&seed).bones;
        let cwd = self.cwd.clone();
        let tx = self.events_tx.clone();

        thread::spawn(move || {
            let soul = generate_hatch_soul(&cwd, &seed, &bones)
                .unwrap_or_else(|_| hatch_fallback(&seed, &bones.rarity, &bones.species));
            let now = Utc::now();
            let buddy = PersistedBuddy {
                hatch_seed: seed,
                name: soul.name,
                personality_paragraph: soul.personality_paragraph,
                hatched_at: now,
                last_rebirth_at: if rebirth { Some(now) } else { None },
                muted: false,
            };
            let _ = tx.send(RuntimeEvent::HatchFinished {
                buddy: Box::new(buddy),
                bones: Box::new(bones),
            });
        });
    }

    fn drain_runtime_events(&mut self) -> Result<()> {
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                RuntimeEvent::BuddyEvent(event) => self.handle_buddy_event(event),
                RuntimeEvent::Commentary(message) => self.push_hidden_commentary(message),
                RuntimeEvent::HatchFinished { buddy, bones } => {
                    self.store.save_global(&buddy)?;
                    self.buddy = Some(*buddy);
                    self.bones = Some(*bones);
                    self.animation.reset(Instant::now());
                    self.app.set_has_buddy(true);
                    self.hatch_in_flight = false;
                    self.status_message = Some("Buddy is alive.".to_string());
                }
                RuntimeEvent::QuipFinished(quip) => {
                    self.quip_in_flight = false;
                    if quip.is_some() {
                        self.last_quip_at = Some(Utc::now());
                        self.bubble_set_at = Some(Instant::now());
                    } else {
                        self.bubble_set_at = None;
                    }
                    self.app.set_active_quip(quip);
                }
                RuntimeEvent::QuipFailed(error) => {
                    self.quip_in_flight = false;
                    self.app.handle_quip_failure();
                    self.status_message = Some(format!("Quip dropped: {error}"));
                }
            }
        }

        Ok(())
    }

    fn handle_buddy_event(&mut self, event: BuddyEvent) {
        self.session_id = event.session_id.clone();

        if let Some(prompt) = &event.user_excerpt {
            self.push_recent_turn(format!("user: {prompt}"));
        }
        if matches!(event.kind, BuddyEventKind::TurnCompleted)
            && let Some(message) = &event.assistant_excerpt
        {
            self.push_recent_turn(format!("assistant: {message}"));
        }

        if matches!(event.kind, BuddyEventKind::ToolStarted) {
            self.active_tool_phase = Some(ActiveToolPhase {
                started_at: Utc::now(),
                tool_name: event.tool_name.clone(),
                long_run_quip_fired: false,
            });
        } else if matches!(event.kind, BuddyEventKind::ToolFinished) {
            self.active_tool_phase = None;
        }

        self.summary.apply(&event);
        self.maybe_spawn_quip(event);
    }

    fn maybe_spawn_quip(&mut self, event: BuddyEvent) {
        let Some(buddy) = self.buddy.clone() else {
            return;
        };
        if buddy.muted || self.quip_in_flight {
            return;
        }

        let now = Utc::now();
        if self
            .last_quip_at
            .is_some_and(|last_quip| now - last_quip < self.quip_policy.cooldown)
        {
            return;
        }

        let should_attempt = matches!(
            event.kind,
            BuddyEventKind::ToolFinished | BuddyEventKind::TurnCompleted
        );

        if !should_attempt || quip_blacklisted(&event) {
            return;
        }

        self.spawn_quip_worker(event, buddy);
    }

    fn spawn_quip_worker(&mut self, event: BuddyEvent, buddy: PersistedBuddy) {
        self.quip_in_flight = true;
        let tx = self.events_tx.clone();
        let cwd = self.cwd.clone();
        let request = QuipRequest {
            buddy_name: buddy.name,
            personality_paragraph: buddy.personality_paragraph,
            event_type: format!("{:?}", event.kind),
            cwd: event.cwd.clone(),
            rolling_summary: serde_json::to_value(&self.summary).unwrap_or_else(|_| json!({})),
            recent_turn_digest: json!({ "recent_turns": self.recent_turns }),
            raw_excerpts: [event.user_excerpt, event.assistant_excerpt]
                .into_iter()
                .flatten()
                .take(2)
                .collect(),
        };

        thread::spawn(move || match generate_quip(&cwd, &request) {
            Ok(response) if response.emit => {
                let sanitized = response.text.as_deref().and_then(sanitize_quip);
                let _ = tx.send(RuntimeEvent::QuipFinished(sanitized));
            }
            Ok(_) => {
                let _ = tx.send(RuntimeEvent::QuipFinished(None));
            }
            Err(err) => {
                let _ = tx.send(RuntimeEvent::QuipFailed(err.to_string()));
            }
        });
    }

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

        let Some(buddy) = self.buddy.clone() else {
            return;
        };
        if buddy.muted {
            return;
        }

        let Some(active_phase) = self.active_tool_phase.clone() else {
            return;
        };
        let now = Utc::now();
        if self
            .last_quip_at
            .is_some_and(|last_quip| now - last_quip < self.quip_policy.cooldown)
        {
            return;
        }

        if can_attempt_long_run_quip(
            active_phase.started_at,
            now,
            active_phase.long_run_quip_fired,
            &self.quip_policy,
        ) {
            let event = BuddyEvent {
                kind: BuddyEventKind::ToolStarted,
                session_id: self.session_id.clone(),
                turn_id: None,
                cwd: self.cwd.display().to_string(),
                tool_name: active_phase.tool_name.clone(),
                tool_command: None,
                tool_success: None,
                assistant_excerpt: Some("Long-running tool phase still active.".to_string()),
                user_excerpt: None,
            };
            self.spawn_quip_worker(event, buddy);
            if let Some(active) = &mut self.active_tool_phase {
                active.long_run_quip_fired = true;
            }
        }
    }

    fn push_recent_turn(&mut self, line: String) {
        self.recent_turns.push(line);
        if self.recent_turns.len() > 4 {
            self.recent_turns.remove(0);
        }
    }

    fn push_hidden_commentary(&mut self, line: String) {
        let normalized = line.trim().to_string();
        if normalized.is_empty() {
            return;
        }
        if self
            .hidden_commentary
            .last()
            .is_some_and(|last| last == &normalized)
        {
            return;
        }

        self.hidden_commentary.push(normalized);
        if self.hidden_commentary.len() > 16 {
            self.hidden_commentary.remove(0);
        }
    }

    fn menu_items(&self) -> Vec<MenuItem> {
        match &self.buddy {
            None => vec![MenuItem::enabled(BuddyMenuAction::Hatch, "Hatch")],
            Some(buddy) => {
                let rebirth_ready =
                    can_rebirth_at(buddy.hatched_at, buddy.last_rebirth_at, Utc::now());
                let rebirth_label = if rebirth_ready {
                    "Rebirth".to_string()
                } else {
                    let gate = buddy.last_rebirth_at.unwrap_or(buddy.hatched_at)
                        + chrono::Duration::days(14);
                    let days_remaining = (gate - Utc::now()).num_days().max(0);
                    format!("Rebirth ({days_remaining}d)")
                };

                vec![
                    MenuItem::enabled(BuddyMenuAction::Status, "Status"),
                    MenuItem::enabled(BuddyMenuAction::Pet, "Pet"),
                    if buddy.muted {
                        MenuItem::enabled(BuddyMenuAction::Unmute, "Unmute")
                    } else {
                        MenuItem::enabled(BuddyMenuAction::Mute, "Mute")
                    },
                    MenuItem {
                        action: BuddyMenuAction::Rebirth,
                        label: rebirth_label,
                        enabled: rebirth_ready,
                    },
                ]
            }
        }
    }

    fn selected_menu_index(&self) -> usize {
        let items = self.menu_items();
        if items.is_empty() {
            0
        } else {
            self.app.menu_index().min(items.len() - 1)
        }
    }

    fn handle_session_end(&mut self, success: bool) {
        let event = BuddyEvent {
            kind: BuddyEventKind::SessionEnded,
            session_id: self.session_id.clone(),
            turn_id: None,
            cwd: self.cwd.display().to_string(),
            tool_name: None,
            tool_command: None,
            tool_success: Some(success),
            assistant_excerpt: None,
            user_excerpt: None,
        };
        self.summary.apply(&event);
        self.active_tool_phase = None;
    }
}

#[derive(Clone)]
struct MenuItem {
    action: BuddyMenuAction,
    label: String,
    enabled: bool,
}

impl MenuItem {
    fn enabled(action: BuddyMenuAction, label: &str) -> Self {
        Self {
            action,
            label: label.to_string(),
            enabled: true,
        }
    }
}

#[derive(Clone, Copy)]
enum BuddyMenuAction {
    Hatch,
    Status,
    Pet,
    Mute,
    Unmute,
    Rebirth,
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableBracketedPaste,
            EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    fn draw<F>(&mut self, render: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame<'_>),
    {
        self.terminal.draw(render)?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            DisableBracketedPaste,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = self.terminal.show_cursor();
    }
}

fn quip_blacklisted(event: &BuddyEvent) -> bool {
    [
        event.user_excerpt.as_deref(),
        event.assistant_excerpt.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|text| {
        let lowered = text.to_ascii_lowercase();
        lowered.contains("api_key")
            || lowered.contains("authorization:")
            || lowered.contains("bearer ")
            || lowered.contains("traceback")
            || lowered.contains("stack trace")
            || lowered.contains("i'm frustrated")
            || lowered.contains("this is stupid")
    })
}

fn main_pane_rect(cols: u16, rows: u16) -> Rect {
    let area = Rect::new(0, 0, cols, rows);
    let [main, _buddy] = split_main_and_buddy(area);
    main
}

fn page_scroll_rows(screen: &vt100::Screen) -> usize {
    usize::from(screen.size().0.saturating_sub(1).max(1))
}

fn rect_contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn spawn_session_monitor(
    sessions_root: PathBuf,
    tx: Sender<RuntimeEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut tracked_file: Option<PathBuf> = None;
        let mut offset = 0_u64;
        let mut remainder = String::new();
        let mut normalizer = SessionEventNormalizer::default();

        loop {
            let Some(session_file) = newest_session_file(&sessions_root) else {
                thread::sleep(Duration::from_millis(50));
                continue;
            };

            if tracked_file.as_ref() != Some(&session_file) {
                tracked_file = Some(session_file.clone());
                offset = 0;
                remainder.clear();
                normalizer = SessionEventNormalizer::default();
            }

            let Ok(metadata) = fs::metadata(&session_file) else {
                thread::sleep(Duration::from_millis(50));
                continue;
            };
            if metadata.len() < offset {
                offset = 0;
                remainder.clear();
                normalizer = SessionEventNormalizer::default();
            }

            let Ok(mut file) = File::open(&session_file) else {
                thread::sleep(Duration::from_millis(50));
                continue;
            };
            if file.seek(SeekFrom::Start(offset)).is_err() {
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            let mut chunk = String::new();
            if file.read_to_string(&mut chunk).is_ok() && !chunk.is_empty() {
                offset += u64::try_from(chunk.len()).unwrap_or(offset);
                remainder.push_str(&chunk);

                while let Some(index) = remainder.find('\n') {
                    let line = remainder[..index].to_string();
                    remainder.drain(..=index);

                    let Ok(events) = normalizer.push_line(&line) else {
                        continue;
                    };
                    for event in events {
                        let runtime_event = match event {
                            SessionEvent::Buddy(event) => RuntimeEvent::BuddyEvent(event),
                            SessionEvent::Commentary(message) => RuntimeEvent::Commentary(message),
                        };
                        if tx.send(runtime_event).is_err() {
                            return;
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    })
}

fn newest_session_file(root: &std::path::Path) -> Option<PathBuf> {
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    let mut pending = vec![root.to_path_buf()];

    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("jsonl") {
                continue;
            }

            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let Ok(modified_at) = metadata.modified() else {
                continue;
            };

            let replace = newest
                .as_ref()
                .is_none_or(|(current_time, _)| modified_at >= *current_time);
            if replace {
                newest = Some((modified_at, path));
            }
        }
    }

    newest.map(|(_, path)| path)
}

fn run(opts: RuntimeOptions) -> Result<()> {
    Runtime::new(opts)?.run()
}
