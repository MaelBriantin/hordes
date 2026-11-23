//! The collection, on the title screen in standard mode: every card, and the reroll, a tile
//! each, in colour once the player has it, grey until then. The arrows go over the tiles; a box
//! under them tells the one picked: there from the start, or the achievement that unlocks it.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;

use super::level_up::{self, COMMON, LEGENDARY, RARE};
use super::sprites::{SpriteId, Sprites};
use super::{XP_COLOR, achievement, bar_of, embossed, labelled, pixels, thousands};
use crate::game::{COLLECTION, Choice, Progress, Rarity, Unlock};

/// A tile: a corner's column, its picture, a corner's column; a corner's row, its picture's
/// rows, a corner's row.
pub(super) const TILE_WIDTH: u16 = 10;
pub(super) const TILE_HEIGHT: u16 = 6;
pub(super) const PICTURE: (u16, u16) = (8, 4);
/// The cells an arrow beside a row takes: as tall as a picture, its pixels in its middle.
const ARROW: (u16, u16) = (3, 4);
/// The most tiles in a row, however wide the screen.
const MOST_IN_A_ROW: usize = 10;
/// The box telling the tile picked: its size, its ground, its words, its bar.
pub(super) const BOX_WIDTH: u16 = 30;
pub(super) const BOX_HEIGHT: u16 = 7;
pub(super) const BOX: Color = Color::Rgb(28, 30, 38);
pub(super) const WHITE: Color = Color::Rgb(225, 225, 230);
pub(super) const GREY: Color = Color::Rgb(140, 145, 160);
/// The room on each side of the box's words and bar.
pub(super) const BOX_PADDING: u16 = 3;
/// The word telling a card the player has.
pub(super) const UNLOCKED: Color = Color::Rgb(110, 200, 110);
/// How dark a tile the player doesn't have is.
const LOCKED_SHARE: f32 = 0.4;

/// How many tiles a row holds, `width` wide: room kept on its sides for the arrows; the rows
/// as even as can be, none left with a few tiles alone.
pub fn in_a_row(width: u16) -> usize {
    let room = width.saturating_sub(2 * (ARROW.0 + 1));
    let fit = (usize::from(room) / usize::from(TILE_WIDTH)).clamp(1, MOST_IN_A_ROW);
    let rows = COLLECTION.len().div_ceil(fit);
    COLLECTION.len().div_ceil(rows)
}

/// The tile under (`down`) or over the one at `at`, rows of `per_row` tiles, each row centred:
/// the one that stands there on the screen, the nearest to it if none quite does; `at` itself on
/// the last row or the first.
pub fn beside(at: usize, per_row: usize, down: bool) -> usize {
    beside_among(at, per_row, COLLECTION.len(), down)
}

/// The tile under or over the one at `at`, among `total` tiles in rows of `per_row`, each row
/// centred.
fn beside_among(at: usize, per_row: usize, total: usize, down: bool) -> usize {
    let per_row = per_row.max(1);
    let (row, column) = (at / per_row, at % per_row);
    let rows = total.div_ceil(per_row);
    let next = match (down, row) {
        (true, row) if row + 1 < rows => row + 1,
        (false, row) if row > 0 => row - 1,
        _ => return at,
    };
    let length = |row: usize| (total - row * per_row).min(per_row);
    // Where it stands, in half tiles from the left of a full row: a shorter row is centred.
    let standing = column * 2 + (per_row - length(row));
    let below = standing.saturating_sub(per_row - length(next)) / 2;
    next * per_row + below.min(length(next) - 1)
}

/// The rows the collection takes, `rows` rows of tiles shown: its title and a row of room, the
/// tiles, a row of room and the box.
pub(super) fn height_of(rows: usize) -> u16 {
    let rows = u16::try_from(rows).unwrap_or(1);
    2 + TILE_HEIGHT * rows + 1 + BOX_HEIGHT
}

/// The rows the collection takes, `width` wide, with `room` rows at most: every row of tiles,
/// or only the picked one's, or none if even that doesn't fit.
pub fn height(width: u16, room: u16) -> u16 {
    let rows = COLLECTION.len().div_ceil(in_a_row(width));
    [rows, 1]
        .into_iter()
        .map(height_of)
        .find(|height| *height <= room)
        .unwrap_or(0)
}

/// The collection in `area`, shown: `picked` the tile the arrows are on, every tile and the box
/// telling the one picked; hidden, nothing.
pub fn render(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    progress: &Progress,
    picked: Option<usize>,
) {
    if area.height == 0 {
        return;
    }
    let per_row = in_a_row(area.width);
    let rows = COLLECTION.len().div_ceil(per_row);
    // Short of room, only the row of the tile picked.
    let shown = if height_of(rows) <= area.height {
        0..rows
    } else {
        let row = picked.unwrap_or(0) / per_row;
        row..row + 1
    };
    let had = COLLECTION
        .iter()
        .filter(|unlock| progress.has(**unlock))
        .count();
    let title = format!("{}  {had} / {}", fl!("collection-title"), COLLECTION.len());
    let title_style = Style::new().fg(WHITE).add_modifier(Modifier::BOLD);
    // Hidden: nothing; its key is at the bottom of the screen.
    if picked.is_none() {
        return;
    }
    centred(buffer, area, area.y, &title, title_style);
    let mut y = area.y + 2;
    let all = shown.len() == rows;
    for row in shown {
        let tiles =
            (row * per_row..((row + 1) * per_row).min(COLLECTION.len())).collect::<Vec<_>>();
        let width = TILE_WIDTH * u16::try_from(tiles.len()).unwrap_or(1);
        let left = area.x + area.width.saturating_sub(width) / 2;
        for (column, &index) in (0_u16..).zip(&tiles) {
            let x = left + column * TILE_WIDTH;
            tile(
                buffer,
                sprites,
                progress,
                index,
                (x, y),
                picked == Some(index),
            );
        }
        // A row alone: arrows on its sides, halfway up the pictures, where more tiles lie.
        if !all {
            let beside = |x| Rect::new(x, y + 1, ARROW.0, ARROW.1);
            if row > 0 {
                let x = left.saturating_sub(ARROW.0 + 1);
                pixels::render_sprite(buffer, beside(x), sprites, SpriteId::ArrowLeft);
            }
            if row + 1 < rows {
                pixels::render_sprite(
                    buffer,
                    beside(left + width + 1),
                    sprites,
                    SpriteId::ArrowRight,
                );
            }
        }
        y += TILE_HEIGHT;
    }
    y += 1;
    if let Some(index) = picked {
        tell(buffer, area, progress, COLLECTION[index], y);
    }
}

/// A tile's frame, its top left at `at`, in `colour`: around a picture, for the tile picked.
pub(super) fn frame(buffer: &mut Buffer, at: (u16, u16), colour: Color) {
    let (x, y) = (at.0 + 1, at.1 + 1);
    let bottom = y + PICTURE.1;
    let style = Style::new().fg(colour);
    let line = "─".repeat(usize::from(PICTURE.0));
    buffer.set_string(at.0, at.1, format!("┌{line}┐"), style);
    buffer.set_string(at.0, bottom, format!("└{line}┘"), style);
    for row in y..bottom {
        buffer.set_string(at.0, row, "│", style);
        buffer.set_string(x + PICTURE.0, row, "│", style);
    }
}

/// A tile, its top left at `at`: its picture, in colour or grey; framed if picked.
fn tile(
    buffer: &mut Buffer,
    sprites: &Sprites,
    progress: &Progress,
    index: usize,
    at: (u16, u16),
    picked: bool,
) {
    let unlock = COLLECTION[index];
    let has = progress.has(unlock);
    let (x, y) = (at.0 + 1, at.1 + 1);
    let inside = Rect::new(x, y, PICTURE.0, PICTURE.1);
    if picked {
        frame(buffer, at, if has { colour(unlock) } else { GREY });
    }
    let picture_of = if hidden(progress, unlock) {
        SpriteId::IconHidden
    } else {
        icon(unlock)
    };
    picture(buffer, sprites, picture_of, inside, has);
}

/// A picture over what is under it, in grey if `coloured` isn't.
fn picture(buffer: &mut Buffer, sprites: &Sprites, sprite: SpriteId, area: Rect, coloured: bool) {
    overlay(buffer, area, coloured, |drawn, apart| {
        pixels::render_sprite(drawn, apart, sprites, sprite);
    });
}

/// What `draw` draws in `area`, over what is under it, in grey if `coloured` isn't.
pub(super) fn overlay(
    buffer: &mut Buffer,
    area: Rect,
    coloured: bool,
    draw: impl FnOnce(&mut Buffer, Rect),
) {
    // Drawn apart first: where it leaves its ground unset, what is under shows.
    let apart = Rect::new(0, 0, area.width, area.height);
    let mut drawn = Buffer::empty(apart);
    for cell in &mut drawn.content {
        cell.set_bg(Color::Reset);
    }
    draw(&mut drawn, apart);
    let shade = |colour: Color| if coloured { colour } else { grey(colour) };
    for row in 0..area.height {
        for column in 0..area.width {
            let from = &drawn[(column, row)];
            if from.symbol() == " " {
                continue;
            }
            let (symbol, fg, bg) = (from.symbol().to_string(), from.fg, from.bg);
            // Off the screen, nothing: the room may run out under it.
            let Some(cell) = buffer.cell_mut((area.x + column, area.y + row)) else {
                continue;
            };
            cell.set_symbol(&symbol).set_fg(shade(fg));
            if bg != Color::Reset {
                cell.set_bg(shade(bg));
            }
        }
    }
}

/// A colour gone grey and dark: what the player doesn't have yet.
pub(super) fn grey(colour: Color) -> Color {
    match colour {
        Color::Rgb(red, green, blue) => {
            let mean = (u16::from(red) + u16::from(green) + u16::from(blue)) / 3;
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a channel, 0 to 255"
            )]
            let level = (f32::from(mean) * LOCKED_SHARE).round() as u8;
            Color::Rgb(level, level, level.saturating_add(6))
        }
        other => other,
    }
}

/// The box under the tiles, at row `y`, a row of room above and below its words, telling
/// `unlock`: its name; there from the start, or the achievement unlocking it, its goal and, a
/// row apart, unlocked, how far along on a bar, or not yet.
fn tell(buffer: &mut Buffer, area: Rect, progress: &Progress, unlock: Unlock, y: u16) {
    let has = progress.has(unlock);
    let achievement = unlock.achievement();
    let reached = achievement.is_none_or(|achievement| progress.achieved.contains(&achievement));
    let counted = achievement
        .and_then(|achievement| progress.count_of(achievement))
        .filter(|_| !reached);
    // What it says, line by line: its name, then where it comes from; hidden, nothing of it.
    let hidden = hidden(progress, unlock);
    let lines = words(unlock);
    let (name, kind) = (name(unlock), kind(unlock));
    let status = if has {
        fl!("collection-unlocked")
    } else {
        fl!("collection-locked")
    }
    .to_uppercase();
    // The same width whatever the tile: as wide as the longest any of them says.
    let longest = COLLECTION
        .iter()
        .flat_map(|unlock| words(*unlock))
        .chain([fl!("collection-unlocked"), fl!("collection-locked")])
        .map(|line| u16::try_from(line.chars().count()).unwrap_or(u16::MAX))
        .max()
        .unwrap_or(0);
    let width = BOX_WIDTH.max(longest + 2 * BOX_PADDING).min(area.width);
    let x = area.x + (area.width - width) / 2;
    let colour = colour(unlock);
    let ground = Style::new().bg(BOX);
    for row in y..y + BOX_HEIGHT {
        buffer.set_string(x, row, " ".repeat(usize::from(width)), ground);
        buffer.set_string(
            x,
            row,
            "▌",
            ground.fg(if has { colour } else { grey(colour) }),
        );
    }
    // Hidden: that alone, in capitals, in its middle.
    if hidden {
        let middle = Rect::new(x, y + BOX_HEIGHT / 2, width, 1);
        let words = fl!("collection-hidden").to_uppercase();
        centred(
            buffer,
            middle,
            middle.y,
            &words,
            ground.fg(GREY).add_modifier(Modifier::BOLD),
        );
        return;
    }
    let (left, top) = (x + BOX_PADDING, y + 1);
    let name_colour = if has { colour } else { GREY };
    buffer.set_string(
        left,
        top,
        name,
        ground.fg(name_colour).add_modifier(Modifier::BOLD),
    );
    // Its kind on the same row, on the right.
    let kind_width = u16::try_from(kind.chars().count()).unwrap_or(0);
    buffer.set_string(
        x + width.saturating_sub(BOX_PADDING + kind_width),
        top,
        kind,
        ground.fg(GREY),
    );
    for (row, line) in (1..).zip(&lines[1..]) {
        let fg = if row == 1 && reached { WHITE } else { GREY };
        buffer.set_string(left, top + row, line, ground.fg(fg));
    }
    // A row apart: how far along on a bar; or, in capitals in the middle of the box, unlocked
    // or locked.
    let wide = width.saturating_sub(2 * BOX_PADDING);
    let last = Rect::new(left, top + 4, wide, 1);
    if let Some((count, of)) = counted {
        bar(buffer, (left, last.y), wide, count, of);
    } else {
        let fg = if has { UNLOCKED } else { GREY };
        let style = ground.fg(fg).add_modifier(Modifier::BOLD);
        centred(buffer, last, last.y, &status, style);
    }
}

/// Whether a tile is hidden: its achievement hidden, and not reached yet.
fn hidden(progress: &Progress, unlock: Unlock) -> bool {
    unlock.achievement().is_some_and(|achievement| {
        achievement.hidden() && !progress.achieved.contains(&achievement)
    })
}

/// What the box says of a tile, but for its last row: its name, then where it comes from.
fn words(unlock: Unlock) -> Vec<String> {
    // Its name, its kind after it, a couple of cells apart: as wide as both.
    let mut lines = vec![format!("{}  {}", name(unlock), kind(unlock))];
    match unlock.achievement() {
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

/// A bar `wide` cells wide, as full as `count` is of `of`, the count in its middle: the XP
/// bar's, in its colour and relief.
fn bar(buffer: &mut Buffer, at: (u16, u16), wide: u16, count: u64, of: u64) {
    let cells = usize::from(wide.saturating_sub(2));
    let (part, whole) = (count.min(of), of);
    let fit = |value: u64| u32::try_from(value).unwrap_or(u32::MAX);
    let mut spans = embossed(&bar_of(fit(part), fit(whole), cells), XP_COLOR);
    let counted = format!("{} / {}", thousands(part), thousands(whole));
    labelled(&mut spans, &counted, XP_COLOR);
    buffer.set_line(at.0, at.1, &Line::from(spans), wide);
}

/// A line of `area` with `text` in its middle.
pub(super) fn centred(buffer: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let width = u16::try_from(text.chars().count()).unwrap_or(area.width);
    buffer.set_string(
        area.x + area.width.saturating_sub(width) / 2,
        y,
        text,
        style,
    );
}

/// What a tile shows: a card's icon, or the reroll's.
fn icon(unlock: Unlock) -> SpriteId {
    match unlock {
        Unlock::Card(choice) => level_up::icon(choice),
        Unlock::Reroll => SpriteId::IconReroll,
        Unlock::Arcade => SpriteId::IconArcade,
    }
}

/// What kind a tile is: a weapon, a passive (the legendary too), an action; the reroll, a
/// bonus; the arcade, a mode.
fn kind(unlock: Unlock) -> String {
    match unlock {
        Unlock::Card(Choice::Weapon(_)) => fl!("card-weapon"),
        Unlock::Card(Choice::Passive(_) | Choice::Legendary(_)) => fl!("card-passive"),
        Unlock::Card(Choice::Action(_)) => fl!("card-action"),
        Unlock::Card(Choice::Heal) => fl!("heal"),
        Unlock::Reroll => fl!("collection-kind-bonus"),
        Unlock::Arcade => fl!("collection-kind-mode"),
    }
}

/// A card's name, or the reroll's.
fn name(unlock: Unlock) -> String {
    match unlock {
        Unlock::Card(choice) => level_up::name(choice),
        Unlock::Reroll => fl!("collection-reroll"),
        Unlock::Arcade => fl!("collection-arcade"),
    }
}

/// The colour of a tile's rarity.
fn colour(unlock: Unlock) -> Color {
    match unlock.rarity() {
        Rarity::Common => COMMON,
        Rarity::Rare => RARE,
        Rarity::Legendary => LEGENDARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_running_off_the_screen_is_cut_rather_than_failing() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
        let sprites = Sprites::load().unwrap();
        // Its lower rows under the screen's last.
        let off = Rect::new(4, 8, PICTURE.0, PICTURE.1);
        overlay(&mut buffer, off, true, |drawn, apart| {
            pixels::render_sprite(drawn, apart, &sprites, SpriteId::IconPistol);
        });
        assert_ne!(buffer[(5, 9)].symbol(), " ", "what fits is drawn");
    }

    #[test]
    fn up_and_down_go_to_the_tile_standing_there_on_a_shorter_centred_row() {
        // 22 tiles, 8 a row: the last row of 6 stands a tile in on each side.
        let beside = |at, down| beside_among(at, 8, 22, down);
        assert_eq!(
            beside(10, true),
            17,
            "the third tile over the second of the last row"
        );
        assert_eq!(
            beside(8, true),
            16,
            "none quite under the first: the nearest"
        );
        assert_eq!(beside(17, false), 10);
        assert_eq!(beside(2, true), 10, "full rows: straight down");
        assert_eq!(beside(17, true), 17, "the last row");
        assert_eq!(beside(3, false), 3, "the first row");
    }

    #[test]
    fn the_rows_fit_the_room_or_only_the_picked_one_shows() {
        assert_eq!(in_a_row(108), 10, "three rows of 10, 10 and 8");
        assert_eq!(in_a_row(63), 5);
        assert_eq!(height(108, 40), height_of(3));
        assert_eq!(height(108, 20), height_of(1));
        assert_eq!(height(108, 10), 0);
    }
}
