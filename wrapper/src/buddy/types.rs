use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const RARITIES: [&str; 5] = ["common", "uncommon", "rare", "epic", "legendary"];
pub const SPECIES: [&str; 18] = [
    "duck", "goose", "blob", "cat", "dragon", "octopus", "owl", "penguin", "turtle", "snail",
    "ghost", "axolotl", "capybara", "cactus", "robot", "rabbit", "mushroom", "chonk",
];
pub const EYES: [&str; 6] = ["·", "✦", "×", "◉", "@", "°"];
pub const HATS: [&str; 8] = [
    "none",
    "crown",
    "tophat",
    "propeller",
    "halo",
    "wizard",
    "beanie",
    "tinyduck",
];
pub const STAT_NAMES: [&str; 5] = ["DEBUGGING", "PATIENCE", "CHAOS", "WISDOM", "SNARK"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompanionBones {
    pub rarity: String,
    pub species: String,
    pub eye: String,
    pub hat: String,
    pub shiny: bool,
    pub stats: BTreeMap<String, u8>,
}

impl CompanionBones {
    pub fn test_fixture() -> Self {
        Self {
            rarity: "rare".to_string(),
            species: "duck".to_string(),
            eye: "·".to_string(),
            hat: "none".to_string(),
            shiny: false,
            stats: [
                ("DEBUGGING".to_string(), 70),
                ("PATIENCE".to_string(), 50),
                ("CHAOS".to_string(), 25),
                ("WISDOM".to_string(), 55),
                ("SNARK".to_string(), 65),
            ]
            .into_iter()
            .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Roll {
    pub bones: CompanionBones,
    pub inspiration_seed: u32,
}
