use super::types::{CompanionBones, EYES, HATS, RARITIES, Roll, SPECIES, STAT_NAMES};
use std::collections::BTreeMap;

const RARITY_WEIGHTS: [u32; 5] = [60, 25, 10, 4, 1];
const RARITY_FLOORS: [u8; 5] = [5, 15, 25, 35, 50];

fn hash_string(seed: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for code_unit in seed.encode_utf16() {
        hash ^= u32::from(code_unit);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn mulberry32(seed: u32) -> impl FnMut() -> f64 {
    let mut a = seed;
    move || {
        a = a.wrapping_add(0x6d2b79f5);
        let mut t = a ^ (a >> 15);
        t = t.wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        let result = t ^ (t >> 14);
        result as f64 / 4_294_967_296.0
    }
}

fn pick<'a, T>(rng: &mut impl FnMut() -> f64, values: &'a [T]) -> &'a T {
    let idx = (rng() * values.len() as f64).floor() as usize;
    &values[idx.min(values.len().saturating_sub(1))]
}

fn roll_rarity(rng: &mut impl FnMut() -> f64) -> &'static str {
    let total: u32 = RARITY_WEIGHTS.iter().sum();
    let mut roll = rng() * f64::from(total);
    for (idx, rarity) in RARITIES.iter().enumerate() {
        roll -= f64::from(RARITY_WEIGHTS[idx]);
        if roll < 0.0 {
            return rarity;
        }
    }
    "common"
}

fn rarity_index(rarity: &str) -> usize {
    RARITIES
        .iter()
        .position(|candidate| *candidate == rarity)
        .unwrap_or(0)
}

fn roll_stats(rng: &mut impl FnMut() -> f64, rarity: &str) -> BTreeMap<String, u8> {
    let floor = RARITY_FLOORS[rarity_index(rarity)];
    let peak = *pick(rng, &STAT_NAMES);
    let mut dump = *pick(rng, &STAT_NAMES);
    while dump == peak {
        dump = *pick(rng, &STAT_NAMES);
    }

    let mut stats = BTreeMap::new();
    for name in STAT_NAMES {
        let value = if name == peak {
            (u16::from(floor) + 50 + (rng() * 30.0).floor() as u16).min(100) as u8
        } else if name == dump {
            (i16::from(floor) - 10 + (rng() * 15.0).floor() as i16).max(1) as u8
        } else {
            floor + (rng() * 40.0).floor() as u8
        };
        stats.insert(name.to_string(), value);
    }
    stats
}

fn roll_from_rng(rng: &mut impl FnMut() -> f64) -> Roll {
    let rarity = roll_rarity(rng);
    let bones = CompanionBones {
        rarity: rarity.to_string(),
        species: pick(rng, &SPECIES).to_string(),
        eye: pick(rng, &EYES).to_string(),
        hat: if rarity == "common" {
            "none".to_string()
        } else {
            pick(rng, &HATS).to_string()
        },
        shiny: rng() < 0.01,
        stats: roll_stats(rng, rarity),
    };

    Roll {
        bones,
        inspiration_seed: (rng() * 1_000_000_000.0).floor() as u32,
    }
}

pub fn roll_with_seed(seed: &str) -> Roll {
    let mut rng = mulberry32(hash_string(seed));
    roll_from_rng(&mut rng)
}
