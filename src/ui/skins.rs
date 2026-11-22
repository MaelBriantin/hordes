//! The skins, on the title screen, S showing them as the collection shows its cards: a tile
//! each, the hero in it from the front, twice as big, the one picked turning round; grey until
//! unlocked. A box under them tells the one picked: there from the start, or the achievement
//! that unlocks it; worn, unlocked or locked.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::collection::{
    self, BOX, BOX_HEIGHT, BOX_PADDING, BOX_WIDTH, GREY, PICTURE, TILE_HEIGHT, TILE_WIDTH,
    UNLOCKED, WHITE,
};
use super::level_up::{COMMON, LEGENDARY};
use super::sprites::{self, Outfit, Skin, SpriteId, Sprites};
use super::{ViewState, achievement, level_up, pixels};
use crate::game::{Facing, Perk, Progress};

/// Every skin, in the order the tiles show them.
pub const SKINS: [Skin; 7] = Skin::ALL;
/// The frames the hero picked waits, from the front, before turning round: a second; and the
/// frames they show each way as they turn.
const TURN_AFTER: u32 = 30;
const TURN_FRAMES: u32 = 16;
/// The box under the tiles: as the collection's, six rows taller, room for what the hero plays
/// with (their weapon, their HP and speed, their perk) and, apart, how to unlock the colours
/// shown.
const SKIN_BOX_HEIGHT: u16 = BOX_HEIGHT + 6;
/// The rows the skins take: their title and a row of room, the tiles, a row of room and the box.
pub const HEIGHT: u16 = 2 + TILE_HEIGHT + 1 + SKIN_BOX_HEIGHT;

/// The cells the check on the skin worn takes.
const CHECK: (u16, u16) = (5, 2);
/// The ways the hero picked faces as they turn round.
const TURN: [Facing; 4] = [Facing::Down, Facing::Right, Facing::Up, Facing::Left];

/// Whether the player may wear a skin: there from the start, or its achievement reached.
pub fn unlocked(progress: &Progress, skin: Skin) -> bool {
    skin.achievement()
        .is_none_or(|achievement| progress.achieved.contains(&achievement))
}

/// The skins, the tile at `picked` framed, its hero turning round once picked `since` frames,
/// and the box telling it; `worn` the one the hero wears.
pub fn render(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    progress: &Progress,
    view: ViewState,
) {
    let (picked, since) = view.skins.unwrap_or_default();
    let (worn, colours) = (view.skin, view.colours);
    if area.height < HEIGHT {
        return;
    }
    let had = SKINS
        .iter()
        .filter(|skin| unlocked(progress, **skin))
        .count();
    let title = format!("{}  {had} / {}", fl!("skins-title"), SKINS.len());
    let title_style = Style::new().fg(WHITE).add_modifier(Modifier::BOLD);
    collection::centred(buffer, area, area.y, &title, title_style);
    let y = area.y + 2;
    let width = TILE_WIDTH * u16::try_from(SKINS.len()).unwrap_or(1);
    let left = area.x + area.width.saturating_sub(width) / 2;
    for (column, (index, skin)) in (0_u16..).zip(SKINS.into_iter().enumerate()) {
        let at = (left + column * TILE_WIDTH, y);
        let has = unlocked(progress, skin);
        let facing = if index == picked {
            collection::frame(buffer, at, if has { COMMON } else { GREY });
            let turning = since.saturating_sub(TURN_AFTER) / TURN_FRAMES;
            TURN[usize::try_from(turning).unwrap_or(0) % TURN.len()]
        } else {
            Facing::Down
        };
        let inside = Rect::new(at.0 + 1, at.1 + 1, PICTURE.0, PICTURE.1);
        // A hero still locked shows in their first colours alone, the others hidden.
        let outfit = Outfit {
            skin,
            colours: if has { colours[index] } else { 0 },
        };
        // In grey until the hero and their colours are unlocked.
        let shown = has && progress.wears(skin, outfit.colours);
        collection::overlay(buffer, inside, shown, |drawn, apart| {
            pixels::render_look_scaled(drawn, apart, sprites, sprites::hero(facing, outfit), 2);
        });
        // The hero picked in more colours than one: arrows over and under the tile, up and down
        // going through them.
        if index == picked && has && sprites::colours_of(skin) > 1 {
            let middle = at.0 + TILE_WIDTH / 2 - 1;
            for (sprite, row) in [
                (SpriteId::ArrowUp, at.1.saturating_sub(1)),
                (SpriteId::ArrowDown, at.1 + TILE_HEIGHT),
            ] {
                collection::overlay(
                    buffer,
                    Rect::new(middle, row, 3, 1),
                    true,
                    |drawn, apart| {
                        pixels::render_sprite(drawn, apart, sprites, sprite);
                    },
                );
            }
        }
        // The one worn, a green check on the bottom right of its picture.
        if skin == worn {
            let check = Rect::new(
                inside.x + PICTURE.0 - CHECK.0,
                inside.y + PICTURE.1 - CHECK.1,
                CHECK.0,
                CHECK.1,
            );
            collection::overlay(buffer, check, true, |drawn, apart| {
                pixels::render_sprite(drawn, apart, sprites, SpriteId::Check);
            });
        }
    }
    let picked_skin = SKINS[picked];
    let colours_locked =
        unlocked(progress, picked_skin) && !progress.wears(picked_skin, colours[picked]);
    let below = y + TILE_HEIGHT + 1;
    tell(
        buffer,
        area,
        progress,
        (SKINS[picked], worn, colours_locked),
        below,
    );
}

/// The box under the tiles, at row `y`, telling `skin`: its name, where it comes from, and, a
/// row apart, worn, unlocked or locked.
fn tell(
    buffer: &mut Buffer,
    area: Rect,
    progress: &Progress,
    (skin, worn, colours_locked): (Skin, Skin, bool),
    y: u16,
) {
    let has = unlocked(progress, skin);
    let lines = words(skin);
    let longest = SKINS
        .iter()
        .flat_map(|skin| words(*skin))
        .chain(
            SKINS
                .iter()
                .flat_map(|skin| [weapon_line(*skin), stats_line(*skin), perk_line(*skin)]),
        )
        .chain([fl!("skin-colours-locked")])
        .map(|line| u16::try_from(line.chars().count()).unwrap_or(u16::MAX))
        .max()
        .unwrap_or(0);
    let width = BOX_WIDTH.max(longest + 2 * BOX_PADDING).min(area.width);
    let x = area.x + (area.width - width) / 2;
    let ground = Style::new().bg(BOX);
    let edge = if has {
        COMMON
    } else {
        collection::grey(COMMON)
    };
    for row in y..y + SKIN_BOX_HEIGHT {
        buffer.set_string(x, row, " ".repeat(usize::from(width)), ground);
        buffer.set_string(x, row, "▌", ground.fg(edge));
    }
    let (left, top) = (x + BOX_PADDING, y + 1);
    let name_colour = if has { COMMON } else { GREY };
    buffer.set_string(
        left,
        top,
        name(skin),
        ground.fg(name_colour).add_modifier(Modifier::BOLD),
    );
    let kind = fl!("card-skin");
    let kind_width = u16::try_from(kind.chars().count()).unwrap_or(0);
    buffer.set_string(
        x + width.saturating_sub(BOX_PADDING + kind_width),
        top,
        kind,
        ground.fg(GREY),
    );
    for (row, line) in (1..).zip(&lines[1..]) {
        let fg = if row == 1 && has { WHITE } else { GREY };
        buffer.set_string(left, top + row, line, ground.fg(fg));
    }
    // Under where it comes from: the weapon the hero starts with.
    // What the hero plays with: their weapon, their HP and speed, their perk.
    buffer.set_string(left, top + 4, weapon_line(skin), ground.fg(WHITE));
    buffer.set_string(left, top + 5, stats_line(skin), ground.fg(WHITE));
    buffer.set_string(left, top + 6, perk_line(skin), ground.fg(GREY));
    // The colours shown, still locked: how to have them.
    if colours_locked {
        let line = fl!("skin-colours-locked");
        buffer.set_string(left, top + 8, line, ground.fg(GREY));
    }
    let (status, fg) = if skin == worn {
        (fl!("skin-worn"), LEGENDARY)
    } else if has {
        (fl!("collection-unlocked"), UNLOCKED)
    } else {
        (fl!("collection-locked"), GREY)
    };
    let last = Rect::new(left, top + 10, width.saturating_sub(2 * BOX_PADDING), 1);
    let style = ground.fg(fg).add_modifier(Modifier::BOLD);
    collection::centred(buffer, last, last.y, &status.to_uppercase(), style);
}

/// What the box says of a skin, but for its last row: its name and kind, then where it comes
/// from.
fn words(skin: Skin) -> Vec<String> {
    let mut lines = vec![format!("{}  {}", name(skin), fl!("card-skin"))];
    match skin.achievement() {
        None => lines.push(fl!("collection-from-start")),
        Some(achievement) => {
            lines.push(fl!(
                "collection-achievement",
                name = achievement::name(achievement)
            ));
            lines.push(achievement::goal(achievement));
        }
    }
    lines
}

/// The line telling the weapon a hero starts with.
fn weapon_line(skin: Skin) -> String {
    fl!("skin-weapon", weapon = level_up::weapon_name(skin.weapon()))
}

/// The line telling a hero's HP and speed.
fn stats_line(skin: Skin) -> String {
    let speed = match skin.speed_percent() {
        0 => fl!("skin-speed-normal"),
        more if more > 0 => fl!("skin-speed-more", percent = more),
        less => fl!("skin-speed-less", percent = less.unsigned_abs()),
    };
    fl!("skin-stats", hp = skin.hp(), speed = speed)
}

/// The line telling what sets a hero apart.
fn perk_line(skin: Skin) -> String {
    match skin.perk() {
        Perk::Versatile => fl!("perk-versatile"),
        Perk::Commando => fl!("perk-commando"),
        Perk::Pyromaniac => fl!("perk-pyromaniac"),
        Perk::Unshakable => fl!("perk-unshakable"),
        Perk::Reflexes => fl!("perk-reflexes"),
        Perk::Dodge => fl!("perk-dodge"),
        Perk::Healer => fl!("perk-healer"),
    }
}

/// A skin's name, its first letter a capital.
pub fn name(skin: Skin) -> String {
    let word = match skin {
        Skin::Survivor => fl!("skin-survivor"),
        Skin::Classic => fl!("skin-classic"),
        Skin::Exterminator => fl!("skin-exterminator"),
        Skin::Demolisher => fl!("skin-demolisher"),
        Skin::Agent => fl!("skin-agent"),
        Skin::Ninja => fl!("skin-ninja"),
        Skin::Medic => fl!("skin-medic"),
    };
    let mut letters = word.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Achievement;

    #[test]
    fn the_survivor_is_there_from_the_start_each_other_hero_waits_for_their_achievement() {
        let mut progress = Progress::default();
        assert!(unlocked(&progress, Skin::Survivor));
        assert!(!unlocked(&progress, Skin::Classic));
        progress.reach(Achievement::Legend);
        assert!(unlocked(&progress, Skin::Classic));
        assert!(!unlocked(&progress, Skin::Exterminator));
        progress.reach(Achievement::Cleaner);
        assert!(unlocked(&progress, Skin::Exterminator));
        assert!(!unlocked(&progress, Skin::Demolisher));
        progress.reach(Achievement::Demolisher);
        assert!(unlocked(&progress, Skin::Demolisher));
    }
}
