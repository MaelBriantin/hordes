//! The game's pixel art, read from `assets/sprites.txt` (built into the game): how that file is
//! written is told at its top.

use std::collections::HashMap;

use ratatui::style::Color;
use thiserror::Error;

pub use crate::game::Skin;
use crate::game::{Enemy, EnemyKind, Facing, Swelling};

const SOURCE: &str = include_str!("../../assets/sprites.txt");
const PALETTE: &str = "palette";

/// Every sprite the game draws, each named as in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteId {
    HeroFront,
    HeroBack,
    /// Facing right.
    HeroSide,
    HeroBikerFront,
    HeroBikerBack,
    HeroBikerSide,
    ClassicFront,
    ClassicBack,
    /// Facing right.
    ClassicSide,
    ClassicBareFront,
    ClassicBareBack,
    /// Facing right.
    ClassicBareSide,
    ExterminatorFront,
    ExterminatorBack,
    /// Facing right.
    ExterminatorSide,
    ExterminatorPhysicistFront,
    ExterminatorPhysicistBack,
    ExterminatorPhysicistSide,
    DemolisherFront,
    DemolisherBack,
    /// Facing right.
    DemolisherSide,
    DemolisherSmithFront,
    DemolisherSmithBack,
    DemolisherSmithSide,
    AgentUniformFront,
    AgentUniformBack,
    /// Facing right.
    AgentUniformSide,
    AgentFront,
    AgentBack,
    /// Facing right.
    AgentSide,
    NinjaFront,
    NinjaBack,
    /// Facing right.
    NinjaSide,
    NinjaScarletFront,
    NinjaScarletBack,
    NinjaScarletSide,
    MedicFront,
    MedicBack,
    /// Facing right.
    MedicSide,
    InfiltratorFront,
    InfiltratorBack,
    InfiltratorSide,
    InfiltratorDarkFront,
    InfiltratorDarkBack,
    InfiltratorDarkSide,
    InfiltratorJungleFront,
    InfiltratorJungleBack,
    InfiltratorJungleSide,
    InfiltratorJungleDarkFront,
    InfiltratorJungleDarkBack,
    InfiltratorJungleDarkSide,
    RiotFront,
    RiotBack,
    RiotSide,
    MedicGreenFront,
    MedicGreenBack,
    /// Facing right.
    MedicGreenSide,
    /// Facing right.
    ZombieSide,
    ZombieFront,
    /// Facing right.
    GhoulSide,
    GhoulFront,
    /// Facing right.
    Shadow,
    /// Facing right.
    Abomination,
    /// Facing right: in the air, and squashed as it lands.
    Parasite,
    ParasiteLanded,
    Bullet,
    Drone,
    Gem,
    GemBlue,
    GemGold,
    Tree,
    Rock,
    Barricade,
    BarricadeMetal,
    BarricadeFile,
    BarricadeFileMetal,
    Grass,
    Pebble,
    Herb,
    Campfire1,
    Campfire2,
    Ashes,
    IconCrowbar,
    IconMachete,
    IconPistol,
    IconMachineGun,
    IconDrones,
    IconShotgun,
    IconMolotov,
    IconVitality,
    IconSwiftness,
    IconMagnet,
    IconMight,
    IconHaste,
    IconFlashlight,
    IconWisdom,
    IconLoot,
    IconRegeneration,
    IconGlare,
    IconDrain,
    IconSniper,
    IconRam,
    IconDash,
    IconFlamethrower,
    IconHammer,
    IconBaton,
    IconKnife,
    IconKatana,
    IconShuriken,
    IconDefibrillator,
    IconChainsaw,
    IconFists,
    IconTruncheon,
    IconC4,
    IconKit,
    IconFireWall,
    IconGrenade,
    IconNapalm,
    IconBarricade,
    IconIllumination,
    IconHeal,
    IconReroll,
    IconHordeHunter,
    IconArcade,
    IconHidden,
    /// The skin worn, on its tile.
    Check,
    ArrowLeft,
    ArrowUp,
    ArrowDown,
    ArrowRight,
    Title,
}

impl SpriteId {
    const ALL: [SpriteId; 131] = [
        SpriteId::HeroFront,
        SpriteId::HeroBack,
        SpriteId::HeroSide,
        SpriteId::HeroBikerFront,
        SpriteId::HeroBikerBack,
        SpriteId::HeroBikerSide,
        SpriteId::ClassicFront,
        SpriteId::ClassicBack,
        SpriteId::ClassicSide,
        SpriteId::ClassicBareFront,
        SpriteId::ClassicBareBack,
        SpriteId::ClassicBareSide,
        SpriteId::ExterminatorFront,
        SpriteId::ExterminatorBack,
        SpriteId::ExterminatorSide,
        SpriteId::ExterminatorPhysicistFront,
        SpriteId::ExterminatorPhysicistBack,
        SpriteId::ExterminatorPhysicistSide,
        SpriteId::DemolisherFront,
        SpriteId::DemolisherBack,
        SpriteId::DemolisherSide,
        SpriteId::DemolisherSmithFront,
        SpriteId::DemolisherSmithBack,
        SpriteId::DemolisherSmithSide,
        SpriteId::AgentUniformFront,
        SpriteId::AgentUniformBack,
        SpriteId::AgentUniformSide,
        SpriteId::AgentFront,
        SpriteId::AgentBack,
        SpriteId::AgentSide,
        SpriteId::NinjaFront,
        SpriteId::NinjaBack,
        SpriteId::NinjaSide,
        SpriteId::NinjaScarletFront,
        SpriteId::NinjaScarletBack,
        SpriteId::NinjaScarletSide,
        SpriteId::MedicFront,
        SpriteId::MedicBack,
        SpriteId::MedicSide,
        SpriteId::InfiltratorFront,
        SpriteId::InfiltratorBack,
        SpriteId::InfiltratorSide,
        SpriteId::InfiltratorDarkFront,
        SpriteId::InfiltratorDarkBack,
        SpriteId::InfiltratorDarkSide,
        SpriteId::InfiltratorJungleFront,
        SpriteId::InfiltratorJungleBack,
        SpriteId::InfiltratorJungleSide,
        SpriteId::InfiltratorJungleDarkFront,
        SpriteId::InfiltratorJungleDarkBack,
        SpriteId::InfiltratorJungleDarkSide,
        SpriteId::RiotFront,
        SpriteId::RiotBack,
        SpriteId::RiotSide,
        SpriteId::MedicGreenFront,
        SpriteId::MedicGreenBack,
        SpriteId::MedicGreenSide,
        SpriteId::ZombieSide,
        SpriteId::ZombieFront,
        SpriteId::GhoulSide,
        SpriteId::GhoulFront,
        SpriteId::Shadow,
        SpriteId::Abomination,
        SpriteId::Parasite,
        SpriteId::ParasiteLanded,
        SpriteId::Bullet,
        SpriteId::Drone,
        SpriteId::Gem,
        SpriteId::GemBlue,
        SpriteId::GemGold,
        SpriteId::Tree,
        SpriteId::Rock,
        SpriteId::Barricade,
        SpriteId::BarricadeMetal,
        SpriteId::BarricadeFile,
        SpriteId::BarricadeFileMetal,
        SpriteId::Grass,
        SpriteId::Pebble,
        SpriteId::Herb,
        SpriteId::Campfire1,
        SpriteId::Campfire2,
        SpriteId::Ashes,
        SpriteId::IconCrowbar,
        SpriteId::IconMachete,
        SpriteId::IconPistol,
        SpriteId::IconMachineGun,
        SpriteId::IconDrones,
        SpriteId::IconShotgun,
        SpriteId::IconMolotov,
        SpriteId::IconVitality,
        SpriteId::IconSwiftness,
        SpriteId::IconMagnet,
        SpriteId::IconMight,
        SpriteId::IconHaste,
        SpriteId::IconFlashlight,
        SpriteId::IconWisdom,
        SpriteId::IconLoot,
        SpriteId::IconRegeneration,
        SpriteId::IconGlare,
        SpriteId::IconDrain,
        SpriteId::IconSniper,
        SpriteId::IconRam,
        SpriteId::IconDash,
        SpriteId::IconFlamethrower,
        SpriteId::IconHammer,
        SpriteId::IconBaton,
        SpriteId::IconKnife,
        SpriteId::IconKatana,
        SpriteId::IconShuriken,
        SpriteId::IconDefibrillator,
        SpriteId::IconChainsaw,
        SpriteId::IconFists,
        SpriteId::IconTruncheon,
        SpriteId::IconC4,
        SpriteId::IconKit,
        SpriteId::IconFireWall,
        SpriteId::IconGrenade,
        SpriteId::IconNapalm,
        SpriteId::IconBarricade,
        SpriteId::IconIllumination,
        SpriteId::IconHeal,
        SpriteId::IconReroll,
        SpriteId::IconHordeHunter,
        SpriteId::IconArcade,
        SpriteId::IconHidden,
        SpriteId::Check,
        SpriteId::ArrowLeft,
        SpriteId::ArrowUp,
        SpriteId::ArrowDown,
        SpriteId::ArrowRight,
        SpriteId::Title,
    ];

    #[allow(clippy::too_many_lines, reason = "an arm a sprite")]
    fn name(self) -> &'static str {
        match self {
            SpriteId::HeroFront => "hero_front",
            SpriteId::HeroBack => "hero_back",
            SpriteId::HeroSide => "hero_side",
            SpriteId::HeroBikerFront => "hero_biker_front",
            SpriteId::HeroBikerBack => "hero_biker_back",
            SpriteId::HeroBikerSide => "hero_biker_side",
            SpriteId::ClassicFront => "classic_front",
            SpriteId::ClassicBack => "classic_back",
            SpriteId::ClassicSide => "classic_side",
            SpriteId::ClassicBareFront => "classic_bare_front",
            SpriteId::ClassicBareBack => "classic_bare_back",
            SpriteId::ClassicBareSide => "classic_bare_side",
            SpriteId::ExterminatorFront => "exterminator_front",
            SpriteId::ExterminatorBack => "exterminator_back",
            SpriteId::ExterminatorSide => "exterminator_side",
            SpriteId::ExterminatorPhysicistFront => "exterminator_physicist_front",
            SpriteId::ExterminatorPhysicistBack => "exterminator_physicist_back",
            SpriteId::ExterminatorPhysicistSide => "exterminator_physicist_side",
            SpriteId::DemolisherFront => "demolisher_front",
            SpriteId::DemolisherBack => "demolisher_back",
            SpriteId::DemolisherSide => "demolisher_side",
            SpriteId::DemolisherSmithFront => "demolisher_smith_front",
            SpriteId::DemolisherSmithBack => "demolisher_smith_back",
            SpriteId::DemolisherSmithSide => "demolisher_smith_side",
            SpriteId::AgentUniformFront => "agent_uniform_front",
            SpriteId::AgentUniformBack => "agent_uniform_back",
            SpriteId::AgentUniformSide => "agent_uniform_side",
            SpriteId::AgentFront => "agent_front",
            SpriteId::AgentBack => "agent_back",
            SpriteId::AgentSide => "agent_side",
            SpriteId::NinjaFront => "ninja_front",
            SpriteId::NinjaBack => "ninja_back",
            SpriteId::NinjaSide => "ninja_side",
            SpriteId::NinjaScarletFront => "ninja_scarlet_front",
            SpriteId::NinjaScarletBack => "ninja_scarlet_back",
            SpriteId::NinjaScarletSide => "ninja_scarlet_side",
            SpriteId::MedicFront => "medic_front",
            SpriteId::MedicBack => "medic_back",
            SpriteId::MedicSide => "medic_side",
            SpriteId::InfiltratorFront => "infiltrator_front",
            SpriteId::InfiltratorBack => "infiltrator_back",
            SpriteId::InfiltratorSide => "infiltrator_side",
            SpriteId::InfiltratorDarkFront => "infiltrator_dark_front",
            SpriteId::InfiltratorDarkBack => "infiltrator_dark_back",
            SpriteId::InfiltratorDarkSide => "infiltrator_dark_side",
            SpriteId::InfiltratorJungleFront => "infiltrator_jungle_front",
            SpriteId::InfiltratorJungleBack => "infiltrator_jungle_back",
            SpriteId::InfiltratorJungleSide => "infiltrator_jungle_side",
            SpriteId::InfiltratorJungleDarkFront => "infiltrator_jungle_dark_front",
            SpriteId::InfiltratorJungleDarkBack => "infiltrator_jungle_dark_back",
            SpriteId::InfiltratorJungleDarkSide => "infiltrator_jungle_dark_side",
            SpriteId::RiotFront => "riot_front",
            SpriteId::RiotBack => "riot_back",
            SpriteId::RiotSide => "riot_side",
            SpriteId::MedicGreenFront => "medic_green_front",
            SpriteId::MedicGreenBack => "medic_green_back",
            SpriteId::MedicGreenSide => "medic_green_side",
            SpriteId::ZombieSide => "zombie_side",
            SpriteId::ZombieFront => "zombie_front",
            SpriteId::GhoulSide => "ghoul_side",
            SpriteId::GhoulFront => "ghoul_front",
            SpriteId::Shadow => "shadow",
            SpriteId::Abomination => "abomination",
            SpriteId::Parasite => "parasite",
            SpriteId::ParasiteLanded => "parasite_landed",
            SpriteId::Bullet => "bullet",
            SpriteId::Drone => "drone",
            SpriteId::Gem => "gem",
            SpriteId::GemBlue => "gem_blue",
            SpriteId::GemGold => "gem_gold",
            SpriteId::Tree => "tree",
            SpriteId::Rock => "rock",
            SpriteId::Barricade => "barricade",
            SpriteId::BarricadeMetal => "barricade_metal",
            SpriteId::BarricadeFile => "barricade_file",
            SpriteId::BarricadeFileMetal => "barricade_file_metal",
            SpriteId::Grass => "grass",
            SpriteId::Pebble => "pebble",
            SpriteId::Herb => "herb",
            SpriteId::Campfire1 => "campfire_1",
            SpriteId::Campfire2 => "campfire_2",
            SpriteId::Ashes => "ashes",
            SpriteId::IconCrowbar => "icon_crowbar",
            SpriteId::IconMachete => "icon_machete",
            SpriteId::IconPistol => "icon_pistol",
            SpriteId::IconMachineGun => "icon_machine_gun",
            SpriteId::IconDrones => "icon_drones",
            SpriteId::IconShotgun => "icon_shotgun",
            SpriteId::IconMolotov => "icon_molotov",
            SpriteId::IconVitality => "icon_vitality",
            SpriteId::IconSwiftness => "icon_swiftness",
            SpriteId::IconMagnet => "icon_magnet",
            SpriteId::IconMight => "icon_might",
            SpriteId::IconHaste => "icon_haste",
            SpriteId::IconFlashlight => "icon_flashlight",
            SpriteId::IconWisdom => "icon_wisdom",
            SpriteId::IconLoot => "icon_loot",
            SpriteId::IconRegeneration => "icon_regeneration",
            SpriteId::IconGlare => "icon_glare",
            SpriteId::IconDrain => "icon_drain",
            SpriteId::IconSniper => "icon_sniper",
            SpriteId::IconRam => "icon_ram",
            SpriteId::IconDash => "icon_dash",
            SpriteId::IconFlamethrower => "icon_flamethrower",
            SpriteId::IconHammer => "icon_hammer",
            SpriteId::IconBaton => "icon_baton",
            SpriteId::IconKnife => "icon_knife",
            SpriteId::IconKatana => "icon_katana",
            SpriteId::IconShuriken => "icon_shuriken",
            SpriteId::IconDefibrillator => "icon_defibrillator",
            SpriteId::IconChainsaw => "icon_chainsaw",
            SpriteId::IconFists => "icon_fists",
            SpriteId::IconTruncheon => "icon_truncheon",
            SpriteId::IconC4 => "icon_c4",
            SpriteId::IconKit => "icon_kit",
            SpriteId::IconFireWall => "icon_fire_wall",
            SpriteId::IconGrenade => "icon_grenade",
            SpriteId::IconNapalm => "icon_napalm",
            SpriteId::IconBarricade => "icon_barricade",
            SpriteId::IconIllumination => "icon_illumination",
            SpriteId::IconHeal => "icon_heal",
            SpriteId::IconReroll => "icon_reroll",
            SpriteId::IconHordeHunter => "icon_horde_hunter",
            SpriteId::IconArcade => "icon_arcade",
            SpriteId::IconHidden => "icon_hidden",
            SpriteId::Check => "check",
            SpriteId::ArrowLeft => "arrow_left",
            SpriteId::ArrowUp => "arrow_up",
            SpriteId::ArrowDown => "arrow_down",
            SpriteId::ArrowRight => "arrow_right",
            SpriteId::Title => "title",
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpriteError {
    #[error("sprites: palette line `{0}` is not `key red green blue`")]
    PaletteLine(String),
    #[error("sprites: `{sprite}` uses `{key}`, which is not in the palette")]
    UnknownKey { sprite: String, key: char },
    #[error("sprites: the rows of `{0}` are not all as wide")]
    Ragged(String),
    #[error("sprites: `{0}` has no pixels")]
    Empty(String),
    #[error("sprites: `{0}` is missing")]
    Missing(&'static str),
    #[error("sprites: `{0}` is not a sprite of the game")]
    Unknown(String),
    #[error("sprites: `{0}` is drawn twice")]
    Twice(String),
}

/// A picture in pixels, row by row: a colour, or nothing.
#[derive(Debug)]
pub struct Sprite {
    width: i32,
    pixels: Vec<Option<Color>>,
}

/// Every sprite of the game.
#[derive(Debug)]
pub struct Sprites(Vec<Sprite>);

/// A sprite as it shows: as drawn, or mirrored left to right.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub sprite: SpriteId,
    pub mirrored: bool,
    /// Every pixel in one colour: white for an enemy just hit, grey for a broken drone.
    pub tint: Option<Color>,
}

impl Look {
    pub fn plain(sprite: SpriteId) -> Self {
        Self {
            sprite,
            mirrored: false,
            tint: None,
        }
    }

    /// A sprite drawn facing right, mirrored when facing left.
    fn sideways(sprite: SpriteId, facing: Facing) -> Self {
        Self {
            mirrored: facing == Facing::Left,
            ..Self::plain(sprite)
        }
    }
}

/// A hero as they look: who, and in which of their colours (0, their first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outfit {
    pub skin: Skin,
    pub colours: u8,
}

/// How many colours a hero comes in: two each, their own and a second; the riot officer one so
/// far.
pub fn colours_of(skin: Skin) -> u8 {
    if skin == Skin::Riot { 1 } else { 2 }
}

/// The hero, as `outfit` says, turned the way they face, in the dark or not: the infiltrator's
/// goggles over their eyes once the torch is out.
pub fn hero_in(facing: Facing, outfit: Outfit, dark: bool) -> Look {
    let look = hero(facing, outfit);
    if !(dark && outfit.skin == Skin::Infiltrator) {
        return look;
    }
    let sprite = match look.sprite {
        SpriteId::InfiltratorFront => SpriteId::InfiltratorDarkFront,
        SpriteId::InfiltratorBack => SpriteId::InfiltratorDarkBack,
        SpriteId::InfiltratorSide => SpriteId::InfiltratorDarkSide,
        SpriteId::InfiltratorJungleFront => SpriteId::InfiltratorJungleDarkFront,
        SpriteId::InfiltratorJungleBack => SpriteId::InfiltratorJungleDarkBack,
        _ => SpriteId::InfiltratorJungleDarkSide,
    };
    Look { sprite, ..look }
}

/// The hero, as `outfit` says, turned the way they face.
pub fn hero(facing: Facing, outfit: Outfit) -> Look {
    let (front, back, side) = match outfit.skin {
        Skin::Survivor if outfit.colours == 1 => (
            SpriteId::HeroBikerFront,
            SpriteId::HeroBikerBack,
            SpriteId::HeroBikerSide,
        ),
        Skin::Survivor => (SpriteId::HeroFront, SpriteId::HeroBack, SpriteId::HeroSide),
        Skin::Classic if outfit.colours == 1 => (
            SpriteId::ClassicBareFront,
            SpriteId::ClassicBareBack,
            SpriteId::ClassicBareSide,
        ),
        Skin::Classic => (
            SpriteId::ClassicFront,
            SpriteId::ClassicBack,
            SpriteId::ClassicSide,
        ),
        Skin::Exterminator if outfit.colours == 1 => (
            SpriteId::ExterminatorPhysicistFront,
            SpriteId::ExterminatorPhysicistBack,
            SpriteId::ExterminatorPhysicistSide,
        ),
        Skin::Exterminator => (
            SpriteId::ExterminatorFront,
            SpriteId::ExterminatorBack,
            SpriteId::ExterminatorSide,
        ),
        Skin::Demolisher if outfit.colours == 1 => (
            SpriteId::DemolisherSmithFront,
            SpriteId::DemolisherSmithBack,
            SpriteId::DemolisherSmithSide,
        ),
        Skin::Demolisher => (
            SpriteId::DemolisherFront,
            SpriteId::DemolisherBack,
            SpriteId::DemolisherSide,
        ),
        Skin::Agent if outfit.colours == 1 => (
            SpriteId::AgentUniformFront,
            SpriteId::AgentUniformBack,
            SpriteId::AgentUniformSide,
        ),
        Skin::Agent => (
            SpriteId::AgentFront,
            SpriteId::AgentBack,
            SpriteId::AgentSide,
        ),
        Skin::Ninja if outfit.colours == 1 => (
            SpriteId::NinjaScarletFront,
            SpriteId::NinjaScarletBack,
            SpriteId::NinjaScarletSide,
        ),
        Skin::Ninja => (
            SpriteId::NinjaFront,
            SpriteId::NinjaBack,
            SpriteId::NinjaSide,
        ),
        Skin::Medic if outfit.colours == 1 => (
            SpriteId::MedicGreenFront,
            SpriteId::MedicGreenBack,
            SpriteId::MedicGreenSide,
        ),
        Skin::Medic => (
            SpriteId::MedicFront,
            SpriteId::MedicBack,
            SpriteId::MedicSide,
        ),
        Skin::Infiltrator if outfit.colours == 1 => (
            SpriteId::InfiltratorJungleFront,
            SpriteId::InfiltratorJungleBack,
            SpriteId::InfiltratorJungleSide,
        ),
        Skin::Infiltrator => (
            SpriteId::InfiltratorFront,
            SpriteId::InfiltratorBack,
            SpriteId::InfiltratorSide,
        ),
        Skin::Riot => (SpriteId::RiotFront, SpriteId::RiotBack, SpriteId::RiotSide),
    };
    match facing {
        Facing::Down => Look::plain(front),
        Facing::Up => Look::plain(back),
        Facing::Right => Look::plain(side),
        Facing::Left => Look::sideways(side, Facing::Left),
    }
}

/// A gem, by what it is worth, in the cards' colours: silver for a point, blue for a few, gold for many.
pub fn gem(value: u32) -> SpriteId {
    match value {
        0 | 1 => SpriteId::Gem,
        2..=4 => SpriteId::GemBlue,
        _ => SpriteId::GemGold,
    }
}

/// The colour of an enemy just hit.
pub const FLASH: Color = Color::Rgb(255, 255, 255);

/// The ticks a swelling abomination blinks slowly, before it blinks fast.
const SWELL_SLOW_TICKS: u32 = 10;

/// A parasite's hops: the ticks each takes, how high it goes, in tiles, and the ticks it lies
/// squashed as it lands.
const HOP_TICKS: u32 = 12;
const HOP_HEIGHT: f32 = 1.5;
const HOP_LANDED: u32 = 2;

/// Where an enemy is in its hop, `ticks` into the run: each its own, never quite as a
/// neighbour's.
fn hop_tick(enemy: &Enemy, ticks: u32) -> u32 {
    ticks.wrapping_add(enemy.id.wrapping_mul(5)) % HOP_TICKS
}

/// How high an enemy shows above where it stands, in tiles: a parasite hops, the rest don't.
pub fn lift(enemy: &Enemy, ticks: u32) -> f32 {
    if enemy.kind != EnemyKind::Parasite {
        return 0.0;
    }
    let hop = u16::try_from(hop_tick(enemy, ticks)).unwrap_or(0);
    let ticks = u16::try_from(HOP_TICKS).unwrap_or(u16::MAX);
    HOP_HEIGHT * (std::f32::consts::PI * f32::from(hop) / f32::from(ticks)).sin()
}

/// A felled abomination swelling: blinking white, faster as it is about to burst.
pub fn swelling(swelling: &Swelling) -> Look {
    let age = swelling.age();
    let every = if age < SWELL_SLOW_TICKS { 4 } else { 2 };
    Look {
        tint: (age / every).is_multiple_of(2).then_some(FLASH),
        ..Look::sideways(SpriteId::Abomination, swelling.facing)
    }
}

/// An enemy as it shows, `ticks` into the run: turned the way it faces, white when just hit.
pub fn enemy(enemy: &Enemy, ticks: u32) -> Look {
    let look = match enemy.kind {
        // A zombie come with a horde shows as a ghoul.
        EnemyKind::Zombie => {
            let (side, front) = if enemy.horde.is_some() {
                (SpriteId::GhoulSide, SpriteId::GhoulFront)
            } else {
                (SpriteId::ZombieSide, SpriteId::ZombieFront)
            };
            match enemy.facing {
                Facing::Left | Facing::Right => Look::sideways(side, enemy.facing),
                Facing::Up | Facing::Down => Look::plain(front),
            }
        }
        EnemyKind::Shadow => Look::sideways(SpriteId::Shadow, enemy.facing),
        EnemyKind::Abomination => Look::sideways(SpriteId::Abomination, enemy.facing),
        EnemyKind::Parasite => {
            let shape = if hop_tick(enemy, ticks) < HOP_LANDED {
                SpriteId::ParasiteLanded
            } else {
                SpriteId::Parasite
            };
            Look::sideways(shape, enemy.facing)
        }
    };
    // Paralysed by a shock, it blinks blue; hit, it flashes white over that.
    let shocked = enemy.shocked > 0 && (ticks / SHOCK_BLINK + enemy.id).is_multiple_of(2);
    let tint = if enemy.flash > 0 {
        Some(FLASH)
    } else {
        shocked.then_some(SHOCK)
    };
    Look { tint, ..look }
}

/// The blue an enemy paralysed by a shock blinks in, every so many ticks.
pub const SHOCK: Color = Color::Rgb(70, 140, 255);
const SHOCK_BLINK: u32 = 3;

impl Sprites {
    /// The game's sprites, from the file built into it.
    pub fn load() -> Result<Self, SpriteError> {
        Self::parse(SOURCE)
    }

    fn parse(source: &str) -> Result<Self, SpriteError> {
        // Blocks of lines between blank lines, comments left out: the palette, then sprites.
        let mut blocks: Vec<Vec<&str>> = vec![Vec::new()];
        for line in source.lines().map(str::trim_end) {
            if line.starts_with('#') {
                continue;
            }
            if line.is_empty() {
                blocks.push(Vec::new());
            } else if let Some(block) = blocks.last_mut() {
                block.push(line);
            }
        }
        let blocks: Vec<_> = blocks
            .into_iter()
            .filter(|block| !block.is_empty())
            .collect();

        let mut palette = HashMap::new();
        for block in blocks.iter().filter(|block| block[0] == PALETTE) {
            for line in &block[1..] {
                let (key, color) = palette_entry(line)?;
                palette.insert(key, color);
            }
        }

        let mut sprites = HashMap::new();
        for block in blocks.iter().filter(|block| block[0] != PALETTE) {
            let name = block[0];
            if !SpriteId::ALL.iter().any(|id| id.name() == name) {
                return Err(SpriteError::Unknown(name.to_owned()));
            }
            if sprites
                .insert(name, sprite(name, &block[1..], &palette)?)
                .is_some()
            {
                return Err(SpriteError::Twice(name.to_owned()));
            }
        }

        SpriteId::ALL
            .iter()
            .map(|id| {
                sprites
                    .remove(id.name())
                    .ok_or(SpriteError::Missing(id.name()))
            })
            .collect::<Result<_, _>>()
            .map(Sprites)
    }

    pub fn get(&self, id: SpriteId) -> &Sprite {
        // `parse` keeps one sprite per id, in the order of `SpriteId::ALL`.
        &self.0[id as usize]
    }
}

/// A line of the palette: `key red green blue`, a comment after it.
fn palette_entry(line: &str) -> Result<(char, Color), SpriteError> {
    let wrong = || SpriteError::PaletteLine(line.to_owned());
    let mut words = line.split_whitespace();
    let mut key = words.next().ok_or_else(wrong)?.chars();
    let (Some(key), None) = (key.next(), key.next()) else {
        return Err(wrong());
    };
    let mut channel = || -> Result<u8, SpriteError> {
        words
            .next()
            .and_then(|word| word.parse().ok())
            .ok_or_else(wrong)
    };
    Ok((key, Color::Rgb(channel()?, channel()?, channel()?)))
}

fn sprite(
    name: &str,
    rows: &[&str],
    palette: &HashMap<char, Color>,
) -> Result<Sprite, SpriteError> {
    let width = rows.first().map_or(0, |row| row.chars().count());
    if width == 0 {
        return Err(SpriteError::Empty(name.to_owned()));
    }
    if rows.iter().any(|row| row.chars().count() != width) {
        return Err(SpriteError::Ragged(name.to_owned()));
    }
    let pixels = rows
        .iter()
        .flat_map(|row| row.chars())
        .map(|key| match key {
            '.' => Ok(None),
            key => palette
                .get(&key)
                .copied()
                .map(Some)
                .ok_or_else(|| SpriteError::UnknownKey {
                    sprite: name.to_owned(),
                    key,
                }),
        })
        .collect::<Result<_, _>>()?;
    Ok(Sprite {
        width: i32::try_from(width).map_err(|_| SpriteError::Ragged(name.to_owned()))?,
        pixels,
    })
}

impl Sprite {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        i32::try_from(self.pixels.len()).unwrap_or(0) / self.width
    }

    /// Each pixel that shows, where it is in the sprite (mirrored or not), and its colour.
    pub fn pixels(&self, mirrored: bool) -> impl Iterator<Item = (i32, i32, Color)> + '_ {
        (0..).zip(&self.pixels).filter_map(move |(index, pixel)| {
            let (x, y) = (index % self.width, index / self.width);
            let x = if mirrored { self.width - 1 - x } else { x };
            pixel.map(|color| (x, y, color))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hero_but_the_riot_officer_comes_in_two_outfits_each_its_own_sprites_every_way() {
        assert_eq!(colours_of(Skin::Riot), 1, "one so far");
        for skin in Skin::ALL.into_iter().filter(|skin| *skin != Skin::Riot) {
            assert_eq!(colours_of(skin), 2);
            let looks = |colours| {
                [Facing::Down, Facing::Up, Facing::Right]
                    .map(|facing| hero(facing, Outfit { skin, colours }).sprite)
            };
            let (first, second) = (looks(0), looks(1));
            assert!(
                first.iter().all(|sprite| !second.contains(sprite)),
                "{skin:?}"
            );
        }
    }

    #[test]
    fn the_infiltrator_s_goggles_come_down_over_their_eyes_in_the_dark() {
        for (colours, facing) in [0, 1].into_iter().flat_map(|colours| {
            [Facing::Down, Facing::Up, Facing::Right, Facing::Left].map(|facing| (colours, facing))
        }) {
            let outfit = Outfit {
                skin: Skin::Infiltrator,
                colours,
            };
            let (lit, dark) = (
                hero_in(facing, outfit, false).sprite,
                hero_in(facing, outfit, true).sprite,
            );
            assert_eq!(lit, hero(facing, outfit).sprite);
            assert_ne!(lit, dark, "{facing:?}");
        }
        let survivor = Outfit {
            skin: Skin::Survivor,
            colours: 0,
        };
        assert_eq!(
            hero_in(Facing::Down, survivor, true).sprite,
            hero(Facing::Down, survivor).sprite
        );
    }

    const PALETTE_ONLY: &str = "palette\nA 1 2 3  a comment\n\n";

    fn with_all_sprites(extra: &str) -> String {
        let mut source = format!("{PALETTE_ONLY}{extra}");
        for id in SpriteId::ALL {
            source.push_str(id.name());
            source.push_str("\nA.\n\n");
        }
        source
    }

    #[test]
    fn the_games_file_holds_every_sprite() {
        Sprites::load().unwrap();
    }

    #[test]
    fn a_sprite_reads_row_by_row() {
        let sprites = Sprites::parse(&with_all_sprites("")).unwrap();
        let sprite = sprites.get(SpriteId::Tree);
        assert_eq!((sprite.width(), sprite.height()), (2, 1));
        let pixels: Vec<_> = sprite.pixels(false).collect();
        assert_eq!(pixels, [(0, 0, Color::Rgb(1, 2, 3))]);
        let mirrored: Vec<_> = sprite.pixels(true).collect();
        assert_eq!(mirrored, [(1, 0, Color::Rgb(1, 2, 3))]);
    }

    #[test]
    fn a_missing_sprite_is_an_error() {
        let source = format!("{PALETTE_ONLY}tree\nA\n");
        assert_eq!(
            Sprites::parse(&source).unwrap_err(),
            SpriteError::Missing("hero_front")
        );
    }

    #[test]
    fn a_key_out_of_the_palette_is_an_error() {
        let source = with_all_sprites("hero_front\nAZ\n\n");
        assert_eq!(
            Sprites::parse(&source).unwrap_err(),
            SpriteError::UnknownKey {
                sprite: "hero_front".to_owned(),
                key: 'Z'
            }
        );
    }

    #[test]
    fn ragged_rows_are_an_error() {
        let source = with_all_sprites("rock\nAA\nA\n\n");
        assert_eq!(
            Sprites::parse(&source).unwrap_err(),
            SpriteError::Ragged("rock".to_owned())
        );
    }

    #[test]
    fn an_unknown_sprite_is_an_error() {
        let source = with_all_sprites("dragon\nA\n\n");
        assert_eq!(
            Sprites::parse(&source).unwrap_err(),
            SpriteError::Unknown("dragon".to_owned())
        );
    }

    #[test]
    fn a_sprite_drawn_twice_is_an_error() {
        let source = with_all_sprites("rock\nA\n\n");
        assert_eq!(
            Sprites::parse(&source).unwrap_err(),
            SpriteError::Twice("rock".to_owned())
        );
    }

    #[test]
    fn a_bad_palette_line_is_an_error() {
        let source = "palette\nA 1 2\n";
        assert_eq!(
            Sprites::parse(source).unwrap_err(),
            SpriteError::PaletteLine("A 1 2".to_owned())
        );
    }
}
