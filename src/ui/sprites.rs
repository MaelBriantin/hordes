//! The game's pixel art, read from `assets/sprites.txt` (built into the game): how that file is
//! written is told at its top.

use std::collections::HashMap;

use ratatui::style::Color;
use thiserror::Error;

use crate::game::{Enemy, EnemyKind, Facing};

const SOURCE: &str = include_str!("../../assets/sprites.txt");
const PALETTE: &str = "palette";

/// Every sprite the game draws, each named as in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteId {
    HeroFront,
    HeroBack,
    /// Facing right.
    HeroSide,
    /// Facing right.
    ZombieSide,
    ZombieFront,
    /// Facing right.
    GhoulSide,
    GhoulFront,
    BatUp,
    BatDown,
    /// Facing right.
    Abomination,
    Bullet,
    Drone,
    Gem,
    Tree,
    Rock,
    Grass,
    Pebble,
    Herb,
    Campfire1,
    Campfire2,
    Ashes,
    IconMachete,
    IconPistol,
    IconMachineGun,
    IconDrones,
    IconVitality,
    IconSwiftness,
    IconMagnet,
    IconMight,
    IconHaste,
    IconFlashlight,
    IconCompass,
    IconWisdom,
    IconIllumination,
    IconHeal,
    Title,
}

impl SpriteId {
    const ALL: [SpriteId; 36] = [
        SpriteId::HeroFront,
        SpriteId::HeroBack,
        SpriteId::HeroSide,
        SpriteId::ZombieSide,
        SpriteId::ZombieFront,
        SpriteId::GhoulSide,
        SpriteId::GhoulFront,
        SpriteId::BatUp,
        SpriteId::BatDown,
        SpriteId::Abomination,
        SpriteId::Bullet,
        SpriteId::Drone,
        SpriteId::Gem,
        SpriteId::Tree,
        SpriteId::Rock,
        SpriteId::Grass,
        SpriteId::Pebble,
        SpriteId::Herb,
        SpriteId::Campfire1,
        SpriteId::Campfire2,
        SpriteId::Ashes,
        SpriteId::IconMachete,
        SpriteId::IconPistol,
        SpriteId::IconMachineGun,
        SpriteId::IconDrones,
        SpriteId::IconVitality,
        SpriteId::IconSwiftness,
        SpriteId::IconMagnet,
        SpriteId::IconMight,
        SpriteId::IconHaste,
        SpriteId::IconFlashlight,
        SpriteId::IconCompass,
        SpriteId::IconWisdom,
        SpriteId::IconIllumination,
        SpriteId::IconHeal,
        SpriteId::Title,
    ];

    fn name(self) -> &'static str {
        match self {
            SpriteId::HeroFront => "hero_front",
            SpriteId::HeroBack => "hero_back",
            SpriteId::HeroSide => "hero_side",
            SpriteId::ZombieSide => "zombie_side",
            SpriteId::ZombieFront => "zombie_front",
            SpriteId::GhoulSide => "ghoul_side",
            SpriteId::GhoulFront => "ghoul_front",
            SpriteId::BatUp => "bat_up",
            SpriteId::BatDown => "bat_down",
            SpriteId::Abomination => "abomination",
            SpriteId::Bullet => "bullet",
            SpriteId::Drone => "drone",
            SpriteId::Gem => "gem",
            SpriteId::Tree => "tree",
            SpriteId::Rock => "rock",
            SpriteId::Grass => "grass",
            SpriteId::Pebble => "pebble",
            SpriteId::Herb => "herb",
            SpriteId::Campfire1 => "campfire_1",
            SpriteId::Campfire2 => "campfire_2",
            SpriteId::Ashes => "ashes",
            SpriteId::IconMachete => "icon_machete",
            SpriteId::IconPistol => "icon_pistol",
            SpriteId::IconMachineGun => "icon_machine_gun",
            SpriteId::IconDrones => "icon_drones",
            SpriteId::IconVitality => "icon_vitality",
            SpriteId::IconSwiftness => "icon_swiftness",
            SpriteId::IconMagnet => "icon_magnet",
            SpriteId::IconMight => "icon_might",
            SpriteId::IconHaste => "icon_haste",
            SpriteId::IconFlashlight => "icon_flashlight",
            SpriteId::IconCompass => "icon_compass",
            SpriteId::IconWisdom => "icon_wisdom",
            SpriteId::IconIllumination => "icon_illumination",
            SpriteId::IconHeal => "icon_heal",
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

/// The hero, turned the way they face.
pub fn hero(facing: Facing) -> Look {
    match facing {
        Facing::Down => Look::plain(SpriteId::HeroFront),
        Facing::Up => Look::plain(SpriteId::HeroBack),
        Facing::Right => Look::plain(SpriteId::HeroSide),
        Facing::Left => Look::sideways(SpriteId::HeroSide, Facing::Left),
    }
}

/// The colour of an enemy just hit.
const FLASH: Color = Color::Rgb(255, 255, 255);

/// An enemy as it shows: turned the way it faces, a bat's wings up or down, white when just hit.
pub fn enemy(enemy: &Enemy) -> Look {
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
        // Three wingbeats to a swerve of its zigzag.
        EnemyKind::Bat if (enemy.phase * 3.0).sin() > 0.0 => Look::plain(SpriteId::BatUp),
        EnemyKind::Bat => Look::plain(SpriteId::BatDown),
        EnemyKind::Abomination => Look::sideways(SpriteId::Abomination, enemy.facing),
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
