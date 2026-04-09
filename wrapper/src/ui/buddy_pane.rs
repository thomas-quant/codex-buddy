use chrono::{DateTime, Utc};
use ratatui::widgets::{Paragraph, Wrap};

use crate::buddy::{
    lifecycle::can_rebirth_at, sprites::render_sprite_frame, store::PersistedBuddy,
    types::CompanionBones,
};

const BUDDY_SPRITE_CANVAS_HEIGHT: usize = 5;

pub struct BuddyMenuEntry<'a> {
    label: &'a str,
    selected: bool,
    enabled: bool,
}

impl<'a> BuddyMenuEntry<'a> {
    pub fn new(label: &'a str, selected: bool, enabled: bool) -> Self {
        Self {
            label,
            selected,
            enabled,
        }
    }
}

fn render_buddy_sprite_lines(bones: &CompanionBones, frame: usize) -> Vec<String> {
    let mut lines = render_sprite_frame(bones, frame);
    if lines.len() < BUDDY_SPRITE_CANVAS_HEIGHT {
        let padding_rows = BUDDY_SPRITE_CANVAS_HEIGHT - lines.len();
        let mut padded_lines = Vec::with_capacity(BUDDY_SPRITE_CANVAS_HEIGHT);
        padded_lines.extend(std::iter::repeat_n(String::new(), padding_rows));
        padded_lines.extend(lines);
        lines = padded_lines;
    }

    lines
}

pub fn render_idle_lines(
    buddy: &PersistedBuddy,
    bones: &CompanionBones,
    frame: usize,
    quip: Option<&str>,
    focused: bool,
) -> Vec<String> {
    let mut lines = render_buddy_sprite_lines(bones, frame);
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
    let mut lines = render_buddy_sprite_lines(bones, frame);
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

pub fn render_action_menu_lines(
    entries: &[BuddyMenuEntry<'_>],
    status_message: Option<&str>,
) -> Vec<String> {
    let mut lines = vec!["Actions".to_string()];

    if let Some(message) = status_message {
        lines.push(message.to_string());
        lines.push(String::new());
    }

    for entry in entries {
        let cursor = if entry.selected { ">" } else { " " };
        let suffix = if entry.enabled { "" } else { " [locked]" };
        lines.push(format!("{cursor} {}{}", entry.label, suffix));
    }

    lines.push(String::new());
    lines.push("Enter: choose  Esc: back".to_string());
    lines
}

pub fn render_buddy_widget<'a>(text: String) -> Paragraph<'a> {
    Paragraph::new(text).wrap(Wrap { trim: false })
}
