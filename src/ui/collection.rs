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
const TILE_WIDTH: u16 = 10;
const TILE_HEIGHT: u16 = 6;
const PICTURE: (u16, u16) = (8, 4);
/// The cells an arrow beside a row takes: as tall as a picture, its pixels in its middle.
const ARROW: (u16, u16) = (3, 4);
/// The most tiles in a row, however wide the screen.
const MOST_IN_A_ROW: usize = 10;
/// The box telling the tile picked: its size, its ground, its words, its bar.
const BOX_WIDTH: u16 = 30;
const BOX_HEIGHT: u16 = 7;
const BOX: Color = Color::Rgb(28, 30, 38);
const WHITE: Color = Color::Rgb(225, 225, 230);
const GREY: Color = Color::Rgb(140, 145, 160);
/// The room on each side of the box's words and bar.
const BOX_PADDING: u16 = 3;
/// The word telling a card the player has.
const UNLOCKED: Color = Color::Rgb(110, 200, 110);
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

/// The rows the collection takes, `rows` rows of tiles shown: its title and a row of room, the
/// tiles, a row of room and the box.
fn height_of(rows: usize) -> u16 {
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

/// The collection in `area`, `picked` the tile the arrows are on, if they are.
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
    let bottom = y + PICTURE.1;
    if picked {
        let colour = Style::new().fg(if has { colour(unlock) } else { GREY });
        let line = "─".repeat(usize::from(PICTURE.0));
        buffer.set_string(at.0, at.1, format!("┌{line}┐"), colour);
        buffer.set_string(at.0, bottom, format!("└{line}┘"), colour);
        for row in y..bottom {
            buffer.set_string(at.0, row, "│", colour);
            buffer.set_string(x + PICTURE.0, row, "│", colour);
        }
    }
    picture(buffer, sprites, icon(unlock), inside, has);
}

/// A picture over what is under it, in grey if `coloured` isn't.
fn picture(buffer: &mut Buffer, sprites: &Sprites, sprite: SpriteId, area: Rect, coloured: bool) {
    // Drawn apart first: where it leaves its ground unset, what is under shows.
    let apart = Rect::new(0, 0, area.width, area.height);
    let mut drawn = Buffer::empty(apart);
    for cell in &mut drawn.content {
        cell.set_bg(Color::Reset);
    }
    pixels::render_sprite(&mut drawn, apart, sprites, sprite);
    let shade = |colour: Color| if coloured { colour } else { grey(colour) };
    for row in 0..area.height {
        for column in 0..area.width {
            let from = &drawn[(column, row)];
            if from.symbol() == " " {
                continue;
            }
            let (symbol, fg, bg) = (from.symbol().to_string(), from.fg, from.bg);
            let cell = &mut buffer[(area.x + column, area.y + row)];
            cell.set_symbol(&symbol).set_fg(shade(fg));
            if bg != Color::Reset {
                cell.set_bg(shade(bg));
            }
        }
    }
}

/// A colour gone grey and dark: what the player doesn't have yet.
fn grey(colour: Color) -> Color {
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
    // What it says, line by line: its name, then where it comes from.
    let lines = words(unlock);
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
    let (left, top) = (x + BOX_PADDING, y + 1);
    let name_colour = if has { colour } else { GREY };
    buffer.set_string(
        left,
        top,
        name(unlock),
        ground.fg(name_colour).add_modifier(Modifier::BOLD),
    );
    // Its kind on the same row, on the right.
    let kind = kind(unlock);
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
fn centred(buffer: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
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
    fn the_rows_fit_the_room_or_only_the_picked_one_shows() {
        assert_eq!(in_a_row(108), 7, "three even rows rather than 10, 10 and 1");
        assert_eq!(in_a_row(63), 5);
        assert_eq!(height(108, 40), height_of(3));
        assert_eq!(height(108, 20), height_of(1));
        assert_eq!(height(108, 10), 0);
    }
}
