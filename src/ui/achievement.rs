//! An achievement reached, told on the map: a box rising from its bottom right, the picture of
//! what it unlocks and its name, in its rarity's colour; it stays a while, then
//! goes down again.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::level_up::{self, COMMON, LEGENDARY, RARE};
use super::pixels;
use super::sprites::{SpriteId, Sprites};
use crate::game::{Achievement, Rarity, Unlock};

/// The frames the box takes to rise, stays up, takes to go down: all of them, its time on screen.
const RISE: u32 = 8;
const STAY: u32 = 120;
const FALL: u32 = 8;
pub const TOAST_FRAMES: u32 = RISE + STAY + FALL;
/// The box's ground, and its words: the name, and the rest.
const GROUND: Color = Color::Rgb(24, 26, 32);
const NAME: Color = Color::Rgb(235, 235, 240);
const WORDS: Color = Color::Rgb(140, 145, 160);
/// The box's rows: a row of room, the header, the name, a row apart, what it unlocks, a row of
/// room. And the cells left of the picture (its coloured edge, a space), the picture's, the
/// gap after it, the room after the words.
const HEIGHT: u16 = 6;
const EDGE: u16 = 2;
const PICTURE: u16 = 8;
const GAP: u16 = 2;
const ROOM: u16 = 2;

/// The achievement told, `since` frames after it started to show, over the bottom right of
/// `map`.
pub fn render(
    buffer: &mut Buffer,
    map: Rect,
    sprites: &Sprites,
    achievement: Achievement,
    since: u32,
) {
    if since >= TOAST_FRAMES {
        return;
    }
    let colour = match achievement.rarity() {
        Rarity::Common => COMMON,
        Rarity::Rare => RARE,
        Rarity::Legendary => LEGENDARY,
    };
    let header = match achievement.rarity() {
        Rarity::Common => fl!("achievement-new"),
        Rarity::Rare => fl!("achievement-new-rare"),
        Rarity::Legendary => fl!("achievement-new-legendary"),
    };
    let name = name(achievement);
    let (unlocked, what, picture) = match achievement.unlocks() {
        Unlock::Card(choice) => (
            fl!("achievement-unlocked"),
            level_up::name(choice),
            Some(level_up::icon(choice)),
        ),
        Unlock::Reroll => (fl!("achievement-unlocked"), fl!("achievement-reroll"), None),
        Unlock::Arcade => (
            fl!("achievement-unlocked"),
            fl!("achievement-arcade"),
            Some(SpriteId::IconArcade),
        ),
    };
    let width_of = |text: &str| u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
    let words = [&header, &name]
        .into_iter()
        .map(|text| width_of(text))
        .chain([width_of(&unlocked) + 1 + width_of(&what)])
        .max()
        .unwrap_or(0);
    let left = if picture.is_some() {
        EDGE + PICTURE + GAP
    } else {
        EDGE
    };
    let width = (left + words + ROOM).min(map.width);
    // The whole box, drawn apart, then as much of it as has risen.
    let area = Rect::new(0, 0, width, HEIGHT);
    let mut boxed = Buffer::empty(area);
    for y in 0..HEIGHT {
        for x in 0..width {
            boxed[(x, y)].set_symbol(" ").set_bg(GROUND);
        }
        boxed[(0, y)].set_symbol("▌").set_fg(colour);
    }
    if let Some(picture) = picture {
        pixels::render_sprite(&mut boxed, Rect::new(EDGE, 1, PICTURE, 4), sprites, picture);
    }
    let plain = Style::new().fg(WORDS).bg(GROUND);
    boxed.set_string(left, 1, &header, Style::new().fg(colour).bg(GROUND));
    boxed.set_string(
        left,
        2,
        &name,
        Style::new()
            .fg(NAME)
            .bg(GROUND)
            .add_modifier(Modifier::BOLD),
    );
    boxed.set_string(left, 4, &unlocked, plain);
    boxed.set_string(
        left + width_of(&unlocked) + 1,
        4,
        &what,
        Style::new()
            .fg(colour)
            .bg(GROUND)
            .add_modifier(Modifier::BOLD),
    );
    let risen = rows_shown(since).min(map.height);
    let (x, y) = (map.right().saturating_sub(width + 1), map.bottom() - risen);
    for row in 0..risen {
        for column in 0..width {
            buffer[(x + column, y + row)] = boxed[(column, row)].clone();
        }
    }
}

/// How many of the box's rows show, `since` frames in: rising, up, going down.
fn rows_shown(since: u32) -> u16 {
    let part = if since < RISE {
        since + 1
    } else if since < RISE + STAY {
        RISE
    } else {
        TOAST_FRAMES - since
    };
    u16::try_from(u32::from(HEIGHT) * part / RISE).unwrap_or(HEIGHT)
}

/// What an achievement asks.
pub fn goal(achievement: Achievement) -> String {
    match achievement {
        Achievement::Hunter => fl!("achievement-hunter-goal"),
        Achievement::Drizzle => fl!("achievement-drizzle-goal"),
        Achievement::Carnage => fl!("achievement-carnage-goal"),
        Achievement::Swarm => fl!("achievement-swarm-goal"),
        Achievement::Rain => fl!("achievement-rain-goal"),
        Achievement::Storm => fl!("achievement-storm-goal"),
        Achievement::ThousandDown => fl!("achievement-thousand-goal"),
        Achievement::Shadows => fl!("achievement-shadows-goal"),
        Achievement::Dawn => fl!("achievement-dawn-goal"),
        Achievement::Cleaner => fl!("achievement-cleaner-goal"),
        Achievement::Demolisher => fl!("achievement-demolisher-goal"),
        Achievement::ColdBlood => fl!("achievement-cold-blood-goal"),
        Achievement::ShadowHunter => fl!("achievement-shadow-hunter-goal"),
        Achievement::Legend => fl!("achievement-legend-goal"),
        Achievement::FirstAid => fl!("achievement-first-aid-goal"),
        Achievement::Shadow => fl!("achievement-shadow-goal"),
        Achievement::HoldTheLine => fl!("achievement-hold-the-line-goal"),
        Achievement::CloseCall => fl!("achievement-close-call-goal"),
        Achievement::Regular => fl!("achievement-regular-goal"),
        Achievement::Veteran => fl!("achievement-veteran-goal"),
    }
}

/// An achievement's name.
pub fn name(achievement: Achievement) -> String {
    match achievement {
        Achievement::Hunter => fl!("achievement-hunter"),
        Achievement::Drizzle => fl!("achievement-drizzle"),
        Achievement::Carnage => fl!("achievement-carnage"),
        Achievement::Swarm => fl!("achievement-swarm"),
        Achievement::Rain => fl!("achievement-rain"),
        Achievement::Storm => fl!("achievement-storm"),
        Achievement::ThousandDown => fl!("achievement-thousand"),
        Achievement::Shadows => fl!("achievement-shadows"),
        Achievement::Dawn => fl!("achievement-dawn"),
        Achievement::Cleaner => fl!("achievement-cleaner"),
        Achievement::Demolisher => fl!("achievement-demolisher"),
        Achievement::ColdBlood => fl!("achievement-cold-blood"),
        Achievement::ShadowHunter => fl!("achievement-shadow-hunter"),
        Achievement::Legend => fl!("achievement-legend"),
        Achievement::FirstAid => fl!("achievement-first-aid"),
        Achievement::Shadow => fl!("achievement-shadow"),
        Achievement::HoldTheLine => fl!("achievement-hold-the-line"),
        Achievement::CloseCall => fl!("achievement-close-call"),
        Achievement::Regular => fl!("achievement-regular"),
        Achievement::Veteran => fl!("achievement-veteran"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_box_rises_stays_and_goes_down() {
        assert_eq!(rows_shown(0), 0);
        assert_eq!(rows_shown(RISE / 2), HEIGHT / 2);
        assert_eq!(rows_shown(RISE - 1), HEIGHT);
        assert_eq!(rows_shown(RISE + STAY / 2), HEIGHT);
        assert_eq!(rows_shown(TOAST_FRAMES - 1), 0);
    }
}
