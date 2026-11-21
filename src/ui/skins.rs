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
use super::sprites::{self, Skin, SpriteId, Sprites};
use super::{achievement, pixels};
use crate::game::{Achievement, Facing, Progress};

/// Every skin, in the order the tiles show them.
pub const SKINS: [Skin; 3] = [Skin::Survivor, Skin::Classic, Skin::Exterminator];
/// The frames the hero picked waits, from the front, before turning round: a second; and the
/// frames they show each way as they turn.
const TURN_AFTER: u32 = 30;
const TURN_FRAMES: u32 = 16;
/// The cells the check on the skin worn takes.
const CHECK: (u16, u16) = (5, 2);
/// The ways the hero picked faces as they turn round.
const TURN: [Facing; 4] = [Facing::Down, Facing::Right, Facing::Up, Facing::Left];

/// The achievement a skin waits for; none for those there from the start.
pub fn achievement_of(skin: Skin) -> Option<Achievement> {
    match skin {
        Skin::Survivor | Skin::Classic => None,
        Skin::Exterminator => Some(Achievement::Cleaner),
    }
}

/// Whether the player may wear a skin: there from the start, or its achievement reached.
pub fn unlocked(progress: &Progress, skin: Skin) -> bool {
    achievement_of(skin).is_none_or(|achievement| progress.achieved.contains(&achievement))
}

/// The skins, the tile at `picked` framed, its hero turning round once picked `since` frames,
/// and the box telling it; `worn` the one the hero wears.
pub fn render(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    progress: &Progress,
    picked: usize,
    worn: Skin,
    since: u32,
) {
    if area.height < collection::height_of(1) {
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
        collection::overlay(buffer, inside, has, |drawn, apart| {
            pixels::render_look_scaled(drawn, apart, sprites, sprites::hero(facing, skin), 2);
        });
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
    tell(
        buffer,
        area,
        progress,
        SKINS[picked],
        worn,
        y + TILE_HEIGHT + 1,
    );
}

/// The box under the tiles, at row `y`, telling `skin`: its name, where it comes from, and, a
/// row apart, worn, unlocked or locked.
fn tell(buffer: &mut Buffer, area: Rect, progress: &Progress, skin: Skin, worn: Skin, y: u16) {
    let has = unlocked(progress, skin);
    let lines = words(skin);
    let longest = SKINS
        .iter()
        .flat_map(|skin| words(*skin))
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
    for row in y..y + BOX_HEIGHT {
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
    let (status, fg) = if skin == worn {
        (fl!("skin-worn"), LEGENDARY)
    } else if has {
        (fl!("collection-unlocked"), UNLOCKED)
    } else {
        (fl!("collection-locked"), GREY)
    };
    let last = Rect::new(left, top + 4, width.saturating_sub(2 * BOX_PADDING), 1);
    let style = ground.fg(fg).add_modifier(Modifier::BOLD);
    collection::centred(buffer, last, last.y, &status.to_uppercase(), style);
}

/// What the box says of a skin, but for its last row: its name and kind, then where it comes
/// from.
fn words(skin: Skin) -> Vec<String> {
    let mut lines = vec![format!("{}  {}", name(skin), fl!("card-skin"))];
    match achievement_of(skin) {
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

/// A skin's name, its first letter a capital.
fn name(skin: Skin) -> String {
    let word = match skin {
        Skin::Survivor => fl!("skin-survivor"),
        Skin::Classic => fl!("skin-classic"),
        Skin::Exterminator => fl!("skin-exterminator"),
    };
    let mut letters = word.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exterminator_waits_for_cleaner_the_others_are_there_from_the_start() {
        let mut progress = Progress::default();
        assert!(unlocked(&progress, Skin::Survivor));
        assert!(unlocked(&progress, Skin::Classic));
        assert!(!unlocked(&progress, Skin::Exterminator));
        progress.reach(Achievement::Cleaner);
        assert!(unlocked(&progress, Skin::Exterminator));
        assert_eq!(
            unlocked(&progress, Skin::Exterminator),
            progress.exterminator()
        );
    }
}
