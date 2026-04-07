use chrono::{DateTime, Duration, Utc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HatchSoul {
    pub name: String,
    pub personality_paragraph: String,
}

pub fn can_rebirth_at(
    hatched_at: DateTime<Utc>,
    last_rebirth_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    let gate = last_rebirth_at.unwrap_or(hatched_at);
    now >= gate + Duration::days(14)
}

fn hash_string(seed: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for code_unit in seed.encode_utf16() {
        hash ^= u32::from(code_unit);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn pick<T>(seed: u32, values: &[T]) -> &T {
    let idx = seed as usize % values.len();
    &values[idx]
}

pub fn hatch_fallback(seed: &str, rarity: &str, species: &str) -> HatchSoul {
    let hash = hash_string(seed);
    let names = [
        "Mochi", "Pip", "Noodle", "Biscuit", "Pebble", "Tater", "Wisp", "Mallow", "Bean", "Clover",
        "Doodle", "Fable", "Kismet", "Puddle", "Quill", "Sparrow", "Tumble", "Velvet", "Waffles",
        "Zuzu",
    ];
    let moods = [
        "steady",
        "gentle",
        "mischievous",
        "bright",
        "curious",
        "cozy",
        "oddly helpful",
        "soft-spoken",
        "clever",
        "patient",
    ];
    let verbs = [
        "keeps notes tidy",
        "nudges stuck thoughts loose",
        "brings a small spark of calm",
        "turns noisy moments into usable next steps",
        "loves a clean plan and a warm mug",
        "finds the useful shape inside chaos",
    ];

    let name = pick(hash ^ 0xA5A5_5A5A, &names).to_string();
    let mood = pick(hash.rotate_left(7), &moods);
    let verb = pick(hash.rotate_right(3), &verbs);
    let personality_paragraph = format!(
        "A {rarity} {species} with a {mood} streak, {verb}. It stays warm, quirky, and useful when the coding context gets muddy, and it never forgets to keep the next tiny step in view."
    );

    HatchSoul {
        name,
        personality_paragraph,
    }
}

pub fn apply_pet(now_ms: i64) -> i64 {
    now_ms
}
