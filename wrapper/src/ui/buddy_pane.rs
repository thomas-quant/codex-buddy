use chrono::{DateTime, Utc};

use crate::buddy::{
    lifecycle::can_rebirth_at, sprites::render_sprite_frame, store::PersistedBuddy,
    types::CompanionBones,
};

pub fn render_idle_lines(
    buddy: &PersistedBuddy,
    bones: &CompanionBones,
    quip: Option<&str>,
    focused: bool,
) -> Vec<String> {
    let mut lines = render_sprite_frame(bones, 0);
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
    now: DateTime<Utc>,
) -> Vec<String> {
    let mut lines = render_sprite_frame(bones, 0);
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
