use crate::buddy::types::CompanionBones;

pub type Frames = [[&'static str; 5]; 3];

const DUCK_FRAMES: Frames = [
    [
        r"            ",
        r"    __      ",
        r"  <({E} )___  ",
        r"   (  ._>   ",
        r"    `--´    ",
    ],
    [
        r"            ",
        r"    __      ",
        r"  <({E} )___  ",
        r"   (  ._>   ",
        r"    `--´~   ",
    ],
    [
        r"            ",
        r"    __      ",
        r"  <({E} )___  ",
        r"   (  .__>  ",
        r"    `--´    ",
    ],
];

const GOOSE_FRAMES: Frames = [
    [
        r"            ",
        r"     ({E}>    ",
        r"     ||     ",
        r"   _(__)_   ",
        r"    ^^^^    ",
    ],
    [
        r"            ",
        r"    ({E}>     ",
        r"     ||     ",
        r"   _(__)_   ",
        r"    ^^^^    ",
    ],
    [
        r"            ",
        r"     ({E}>>   ",
        r"     ||     ",
        r"   _(__)_   ",
        r"    ^^^^    ",
    ],
];

const BLOB_FRAMES: Frames = [
    [
        r"            ",
        r"   .----.   ",
        r"  ( {E}  {E} )  ",
        r"  (      )  ",
        r"   `----´   ",
    ],
    [
        r"            ",
        r"  .------.  ",
        r" (  {E}  {E}  ) ",
        r" (        ) ",
        r"  `------´  ",
    ],
    [
        r"            ",
        r"    .--.    ",
        r"   ({E}  {E})   ",
        r"   (    )   ",
        r"    `--´    ",
    ],
];

const CAT_FRAMES: Frames = [
    [
        r"            ",
        r"   /\_/\    ",
        r"  ( {E}   {E})  ",
        r"  (  ω  )   ",
        r#"  (")_(")   "#,
    ],
    [
        r"            ",
        r"   /\_/\    ",
        r"  ( {E}   {E})  ",
        r"  (  ω  )   ",
        r#"  (")_(")~  "#,
    ],
    [
        r"            ",
        r"   /\-/\    ",
        r"  ( {E}   {E})  ",
        r"  (  ω  )   ",
        r#"  (")_(")   "#,
    ],
];

const DRAGON_FRAMES: Frames = [
    [
        r"            ",
        r"  /^\  /^\  ",
        r" <  {E}  {E}  > ",
        r" (   ~~   ) ",
        r"  `-vvvv-´  ",
    ],
    [
        r"            ",
        r"  /^\  /^\  ",
        r" <  {E}  {E}  > ",
        r" (        ) ",
        r"  `-vvvv-´  ",
    ],
    [
        r"   ~    ~   ",
        r"  /^\  /^\  ",
        r" <  {E}  {E}  > ",
        r" (   ~~   ) ",
        r"  `-vvvv-´  ",
    ],
];

const OCTOPUS_FRAMES: Frames = [
    [
        r"            ",
        r"   .----.   ",
        r"  ( {E}  {E} )  ",
        r"  (______)  ",
        r"  /\/\/\/\  ",
    ],
    [
        r"            ",
        r"   .----.   ",
        r"  ( {E}  {E} )  ",
        r"  (______)  ",
        r"  \/\/\/\/  ",
    ],
    [
        r"     o      ",
        r"   .----.   ",
        r"  ( {E}  {E} )  ",
        r"  (______)  ",
        r"  /\/\/\/\  ",
    ],
];

const OWL_FRAMES: Frames = [
    [
        r"            ",
        r"   /\  /\   ",
        r"  (({E})({E}))  ",
        r"  (  ><  )  ",
        r"   `----´   ",
    ],
    [
        r"            ",
        r"   /\  /\   ",
        r"  (({E})({E}))  ",
        r"  (  ><  )  ",
        r"   .----.   ",
    ],
    [
        r"            ",
        r"   /\  /\   ",
        r"  (({E})(-))  ",
        r"  (  ><  )  ",
        r"   `----´   ",
    ],
];

const PENGUIN_FRAMES: Frames = [
    [
        r"            ",
        r"  .---.     ",
        r"  ({E}>{E})     ",
        r" /(   )\    ",
        r"  `---´     ",
    ],
    [
        r"            ",
        r"  .---.     ",
        r"  ({E}>{E})     ",
        r" |(   )|    ",
        r"  `---´     ",
    ],
    [
        r"  .---.     ",
        r"  ({E}>{E})     ",
        r" /(   )\    ",
        r"  `---´     ",
        r"   ~ ~      ",
    ],
];

const TURTLE_FRAMES: Frames = [
    [
        r"            ",
        r"   _,--._   ",
        r"  ( {E}  {E} )  ",
        r" /[______]\ ",
        r"  ``    ``  ",
    ],
    [
        r"            ",
        r"   _,--._   ",
        r"  ( {E}  {E} )  ",
        r" /[______]\ ",
        r"   ``  ``   ",
    ],
    [
        r"            ",
        r"   _,--._   ",
        r"  ( {E}  {E} )  ",
        r" /[======]\ ",
        r"  ``    ``  ",
    ],
];

const SNAIL_FRAMES: Frames = [
    [
        r"            ",
        r" {E}    .--.  ",
        r"  \  ( @ )  ",
        r"   \_`--´   ",
        r"  ~~~~~~~   ",
    ],
    [
        r"            ",
        r"  {E}   .--.  ",
        r"  |  ( @ )  ",
        r"   \_`--´   ",
        r"  ~~~~~~~   ",
    ],
    [
        r"            ",
        r" {E}    .--.  ",
        r"  \  ( @  ) ",
        r"   \_`--´   ",
        r"   ~~~~~~   ",
    ],
];

const GHOST_FRAMES: Frames = [
    [
        r"            ",
        r"   .----.   ",
        r"  / {E}  {E} \  ",
        r"  |      |  ",
        r"  ~`~``~`~  ",
    ],
    [
        r"            ",
        r"   .----.   ",
        r"  / {E}  {E} \  ",
        r"  |      |  ",
        r"  `~`~~`~`  ",
    ],
    [
        r"    ~  ~    ",
        r"   .----.   ",
        r"  / {E}  {E} \  ",
        r"  |      |  ",
        r"  ~~`~~`~~  ",
    ],
];

const AXOLOTL_FRAMES: Frames = [
    [
        r"            ",
        r"}~(______)~{",
        r"}~({E} .. {E})~{",
        r"  ( .--. )  ",
        r"  (_/  \_)  ",
    ],
    [
        r"            ",
        r"~}(______){~",
        r"~}({E} .. {E}){~",
        r"  ( .--. )  ",
        r"  (_/  \_)  ",
    ],
    [
        r"            ",
        r"}~(______)~{",
        r"}~({E} .. {E})~{",
        r"  (  --  )  ",
        r"  ~_/  \_~  ",
    ],
];

const CAPYBARA_FRAMES: Frames = [
    [
        r"            ",
        r"  n______n  ",
        r" ( {E}    {E} ) ",
        r" (   oo   ) ",
        r"  `------´  ",
    ],
    [
        r"            ",
        r"  n______n  ",
        r" ( {E}    {E} ) ",
        r" (   Oo   ) ",
        r"  `------´  ",
    ],
    [
        r"    ~  ~    ",
        r"  u______n  ",
        r" ( {E}    {E} ) ",
        r" (   oo   ) ",
        r"  `------´  ",
    ],
];

const CACTUS_FRAMES: Frames = [
    [
        r"            ",
        r" n  ____  n ",
        r" | |{E}  {E}| | ",
        r" |_|    |_| ",
        r"   |    |   ",
    ],
    [
        r"            ",
        r"    ____    ",
        r" n |{E}  {E}| n ",
        r" |_|    |_| ",
        r"   |    |   ",
    ],
    [
        r" n        n ",
        r" |  ____  | ",
        r" | |{E}  {E}| | ",
        r" |_|    |_| ",
        r"   |    |   ",
    ],
];

const ROBOT_FRAMES: Frames = [
    [
        r"            ",
        r"   .[||].   ",
        r"  [ {E}  {E} ]  ",
        r"  [ ==== ]  ",
        r"  `------´  ",
    ],
    [
        r"            ",
        r"   .[||].   ",
        r"  [ {E}  {E} ]  ",
        r"  [ -==- ]  ",
        r"  `------´  ",
    ],
    [
        r"     *      ",
        r"   .[||].   ",
        r"  [ {E}  {E} ]  ",
        r"  [ ==== ]  ",
        r"  `------´  ",
    ],
];

const RABBIT_FRAMES: Frames = [
    [
        r"            ",
        r"   (\__/)   ",
        r"  ( {E}  {E} )  ",
        r" =(  ..  )= ",
        r#"  (")__(")  "#,
    ],
    [
        r"            ",
        r"   (|__/)   ",
        r"  ( {E}  {E} )  ",
        r" =(  ..  )= ",
        r#"  (")__(")  "#,
    ],
    [
        r"            ",
        r"   (\__/)   ",
        r"  ( {E}  {E} )  ",
        r" =( .  . )= ",
        r#"  (")__(")  "#,
    ],
];

const MUSHROOM_FRAMES: Frames = [
    [
        r"            ",
        r" .-o-OO-o-. ",
        r"(__________)",
        r"   |{E}  {E}|   ",
        r"   |____|   ",
    ],
    [
        r"            ",
        r" .-O-oo-O-. ",
        r"(__________)",
        r"   |{E}  {E}|   ",
        r"   |____|   ",
    ],
    [
        r"   . o  .   ",
        r" .-o-OO-o-. ",
        r"(__________)",
        r"   |{E}  {E}|   ",
        r"   |____|   ",
    ],
];

const CHONK_FRAMES: Frames = [
    [
        r"            ",
        r"  /\    /\  ",
        r" ( {E}    {E} ) ",
        r" (   ..   ) ",
        r"  `------´  ",
    ],
    [
        r"            ",
        r"  /\    /|  ",
        r" ( {E}    {E} ) ",
        r" (   ..   ) ",
        r"  `------´  ",
    ],
    [
        r"            ",
        r"  /\    /\  ",
        r" ( {E}    {E} ) ",
        r" (   ..   ) ",
        r"  `------´~ ",
    ],
];

const HAT_LINES: [(&str, &str); 7] = [
    ("none", ""),
    ("crown", r"   \^^^/    "),
    ("tophat", r"   [___]    "),
    ("propeller", r"    -+-     "),
    ("halo", r"   (   )    "),
    ("wizard", r"    /^\     "),
    ("beanie", r"   (___)    "),
];

const TINY_DUCK_HAT: &str = r"    ,>      ";

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

fn hat_line(hat: &str) -> Option<&'static str> {
    match hat {
        "none" => Some(""),
        "tinyduck" => Some(TINY_DUCK_HAT),
        _ => HAT_LINES
            .iter()
            .find_map(|(name, line)| (*name == hat).then_some(*line)),
    }
}

pub fn render_sprite_frame(bones: &CompanionBones, frame: usize) -> Vec<String> {
    let frames = species_frames(&bones.species);
    let mut lines: Vec<String> = frames[frame % frames.len()]
        .iter()
        .map(|line| line.replace("{E}", &bones.eye))
        .collect();

    if let Some(hat) = hat_line(&bones.hat) {
        if !lines[0].trim().is_empty() {
            return lines;
        }

        if !hat.is_empty() {
            lines[0] = hat.to_string();
        }

        if lines[0].trim().is_empty() && frames.iter().all(|frame| frame[0].trim().is_empty()) {
            lines.remove(0);
        }
    }

    lines
}

pub fn render_face(bones: &CompanionBones) -> String {
    let eye = bones.eye.as_str();
    match bones.species.as_str() {
        "duck" | "goose" => format!("({eye}>"),
        "blob" => format!("({eye}{eye})"),
        "cat" => format!("={eye}ω{eye}="),
        "dragon" => format!("<{eye}~{eye}>"),
        "octopus" => format!("~({eye}{eye})~"),
        "owl" => format!("({eye})({eye})"),
        "penguin" => format!("({eye}>)"),
        "turtle" => format!("[{eye}_{eye}]"),
        "snail" => format!("{eye}(@)"),
        "ghost" => format!("/{eye}{eye}\\"),
        "axolotl" => format!("}}{eye}.{eye}{{"),
        "capybara" => format!("({eye}oo{eye})"),
        "cactus" => format!("|{eye}  {eye}|"),
        "robot" => format!("[{eye}{eye}]"),
        "rabbit" => format!("({eye}..{eye})"),
        "mushroom" => format!("|{eye}  {eye}|"),
        "chonk" => format!("({eye}.{eye})"),
        other => panic!("unknown species: {other}"),
    }
}
