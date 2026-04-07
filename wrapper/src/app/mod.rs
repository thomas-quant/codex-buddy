pub mod event;
mod runtime;

pub use runtime::run_default;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppAction {
    ToggleFocus,
    OpenBuddyMenu,
    CloseBuddyMenu,
    NextBuddyAction,
    PrevBuddyAction,
    OpenBuddyStatus,
    CloseBuddyStatus,
}

pub struct App {
    focus: UiFocus,
    has_buddy: bool,
    buddy_menu_open: bool,
    buddy_status_open: bool,
    menu_index: usize,
    active_quip: Option<String>,
    last_pet_at_ms: Option<i64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFocus {
    Pty,
    BuddyPane,
}

impl App {
    pub fn new_for_test() -> Self {
        Self {
            focus: UiFocus::Pty,
            has_buddy: false,
            buddy_menu_open: false,
            buddy_status_open: false,
            menu_index: 0,
            active_quip: None,
            last_pet_at_ms: None,
        }
    }

    pub fn apply(&mut self, action: AppAction) {
        match action {
            AppAction::ToggleFocus => {
                self.focus = match self.focus {
                    UiFocus::Pty => UiFocus::BuddyPane,
                    UiFocus::BuddyPane => UiFocus::Pty,
                };
                if self.focus == UiFocus::Pty {
                    self.buddy_menu_open = false;
                    self.buddy_status_open = false;
                }
            }
            AppAction::OpenBuddyMenu => {
                if self.focus == UiFocus::BuddyPane {
                    self.buddy_menu_open = true;
                    self.buddy_status_open = false;
                    self.menu_index = 0;
                }
            }
            AppAction::CloseBuddyMenu => {
                self.buddy_menu_open = false;
            }
            AppAction::NextBuddyAction => {
                if self.buddy_menu_open {
                    self.menu_index = self.menu_index.saturating_add(1);
                }
            }
            AppAction::PrevBuddyAction => {
                if self.buddy_menu_open {
                    self.menu_index = self.menu_index.saturating_sub(1);
                }
            }
            AppAction::OpenBuddyStatus => {
                if self.focus == UiFocus::BuddyPane {
                    self.buddy_status_open = true;
                    self.buddy_menu_open = false;
                }
            }
            AppAction::CloseBuddyStatus => {
                self.buddy_status_open = false;
            }
        }
    }

    pub fn focus(&self) -> UiFocus {
        self.focus
    }

    pub fn is_buddy_menu_open(&self) -> bool {
        self.buddy_menu_open
    }

    pub fn is_buddy_status_open(&self) -> bool {
        self.buddy_status_open
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index
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

    pub fn set_has_buddy(&mut self, value: bool) {
        self.has_buddy = value;
    }

    pub fn set_active_quip(&mut self, value: Option<String>) {
        self.active_quip = value;
    }

    pub fn set_active_quip_for_test(&mut self, value: &str) {
        self.active_quip = Some(value.to_string());
    }

    pub fn handle_quip_failure(&mut self) {
        self.active_quip = None;
    }

    pub fn active_quip(&self) -> Option<&str> {
        self.active_quip.as_deref()
    }

    pub fn set_last_pet_at_ms_for_test(&mut self, value: i64) {
        self.last_pet_at_ms = Some(value);
    }

    pub fn last_pet_at_ms(&self) -> Option<i64> {
        self.last_pet_at_ms
    }

    pub fn set_last_pet_at_ms(&mut self, value: Option<i64>) {
        self.last_pet_at_ms = value;
    }
}
