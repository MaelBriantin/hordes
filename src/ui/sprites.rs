//! The game's pixel art, read from `assets/sprites.txt` (built into the game): how that file is
//! written is told at its top.

use std::collections::HashMap;

use ratatui::style::Color;
use thiserror::Error;

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
    ClassicFront,
    ClassicBack,
    /// Facing right.
    ClassicSide,
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
    IconDash,
    IconIllumination,
    IconHeal,
    IconReroll,
    IconHordeHunter,
    IconArcade,
    ArrowLeft,
    ArrowRight,
    Title,
}

impl SpriteId {
    const ALL: [SpriteId; 53] = [
        SpriteId::HeroFront,
        SpriteId::HeroBack,
        SpriteId::HeroSide,
        SpriteId::ClassicFront,
        SpriteId::ClassicBack,
        SpriteId::ClassicSide,
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
        SpriteId::IconDash,
        SpriteId::IconIllumination,
        SpriteId::IconHeal,
        SpriteId::IconReroll,
        SpriteId::IconHordeHunter,
        SpriteId::IconArcade,
        SpriteId::ArrowLeft,
        SpriteId::ArrowRight,
        SpriteId::Title,
    ];

    fn name(self) -> &'static str {
        match self {
            SpriteId::HeroFront => "hero_front",
            SpriteId::HeroBack => "hero_back",
            SpriteId::HeroSide => "hero_side",
            SpriteId::ClassicFront => "classic_front",
            SpriteId::ClassicBack => "classic_back",
            SpriteId::ClassicSide => "classic_side",
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
            SpriteId::IconDash => "icon_dash",
            SpriteId::IconIllumination => "icon_illumination",
            SpriteId::IconHeal => "icon_heal",
            SpriteId::IconReroll => "icon_reroll",
            SpriteId::IconHordeHunter => "icon_horde_hunter",
            SpriteId::IconArcade => "icon_arcade",
            SpriteId::ArrowLeft => "arrow_left",
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

/// How the hero looks: a setting, the survivor at first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skin {
    /// In a leather jacket and jeans.
    Survivor,
    /// The hero of the first versions, in a red headband.
    Classic,
}

impl Skin {
    /// The other one, round and round.
    pub fn next(self) -> Self {
        match self {
            Skin::Survivor => Skin::Classic,
            Skin::Classic => Skin::Survivor,
        }
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Skin::Survivor => "survivor",
            Skin::Classic => "classic",
        }
    }
}

/// The hero, as `skin` says, turned the way they face.
pub fn hero(facing: Facing, skin: Skin) -> Look {
    let (front, back, side) = match skin {
        Skin::Survivor => (SpriteId::HeroFront, SpriteId::HeroBack, SpriteId::HeroSide),
        Skin::Classic => (
            SpriteId::ClassicFront,
            SpriteId::ClassicBack,
            SpriteId::ClassicSide,
        ),
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
const FLASH: Color = Color::Rgb(255, 255, 255);

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
    Look {
        tint: (enemy.flash > 0).then_some(FLASH),
        ..look
    }
}

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
