//! A horde as the screen tells it: announced by a blinking banner on top of the map, arrows on
//! the edge it comes from (every edge for a ring) and the map shaking; wiped out, a banner in
//! gold with its bonus.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};

use super::Blank;
use super::text::centred_as_is;
use crate::game::{Facing, TICKS_PER_SECOND, World};

const DANGER: Color = Color::Rgb(230, 50, 50);
const GOLD: Color = Color::Rgb(240, 200, 60);
/// The frames the warning stays lit, then out, as it blinks.
const BLINK: u32 = 6;
/// How long the banner of a horde wiped out stays, in seconds.
const CLEARED_SECONDS: u32 = 2;

/// The horde's banners and arrows over the map.
pub fn render(frame: &mut Frame, map: Rect, world: &World, frames: u32) {
    if let Some(warning) = world.horde_warning {
        if (frames / BLINK).is_multiple_of(2) {
            let words = match warning.side {
                Some(_) => fl!("horde-coming"),
                None => fl!("horde-everywhere"),
            };
            banner(frame, map, &format!("════  {words}  ════"), DANGER);
        }
        let sides = match warning.side {
            Some(side) => vec![side],
            None => vec![Facing::Up, Facing::Down, Facing::Left, Facing::Right],
        };
        for side in sides {
            arrows(frame.buffer_mut(), map, side);
        }
    }
    let ticks_shown = CLEARED_SECONDS * u32::from(TICKS_PER_SECOND);
    if let Some((at, bonus)) = world.horde_cleared
        && world.ticks().saturating_sub(at) < ticks_shown
    {
        banner(frame, map, &fl!("horde-cleared", points = bonus), GOLD);
    }
}

/// How far the map shakes, in columns, while a horde is announced: twice, right then left.
pub fn shake(world: &World) -> i16 {
    let Some(warning) = world.horde_warning else {
        return 0;
    };
    let left = warning.arrives_at.saturating_sub(world.ticks());
    match left {
        80..=85 | 44..=49 => 1,
        74..=79 | 38..=43 => -1,
        _ => 0,
    }
}

/// Moves what the map shows by `by` columns, right or left: the column that comes in stays as
/// it was, the one that goes is lost.
pub fn shift(buffer: &mut Buffer, map: Rect, by: i16) {
    if by == 0 {
        return;
    }
    for y in map.top()..map.bottom() {
        let row: Vec<_> = (map.left()..map.right())
            .map(|x| buffer[(x, y)].clone())
            .collect();
        for (offset, cell) in row.into_iter().enumerate() {
            let Ok(offset) = i32::try_from(offset) else {
                continue;
            };
            let x = i32::from(map.left()) + offset + i32::from(by);
            if let Ok(x) = u16::try_from(x)
                && (map.left()..map.right()).contains(&x)
            {
                buffer[(x, y)] = cell;
            }
        }
    }
}

/// A line of words centred on the top row of the map, on its own dark ground.
fn banner(frame: &mut Frame, map: Rect, words: &str, color: Color) {
    let width = u16::try_from(words.chars().count() + 2).unwrap_or(map.width);
    let [row] = Layout::vertical([Constraint::Length(1)]).areas(map);
    let [ground] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Blank, ground);
    centred_as_is(
        frame,
        words,
        Style::new().fg(color).add_modifier(Modifier::BOLD),
        ground,
    );
}

/// Three arrows on the middle of an edge of the map, pointing in from where the horde comes.
fn arrows(buffer: &mut Buffer, map: Rect, side: Facing) {
    let style = Style::new().fg(DANGER).add_modifier(Modifier::BOLD);
    let (middle_x, middle_y) = (map.x + map.width / 2, map.y + map.height / 2);
    let cells: [(u16, u16); 3] = match side {
        Facing::Right => [0, 1, 2].map(|i| (map.right() - 1, middle_y + i - 1)),
        Facing::Left => [0, 1, 2].map(|i| (map.left(), middle_y + i - 1)),
        // Below the banner, on the top edge.
        Facing::Up => [0, 1, 2].map(|i| (middle_x + i - 1, map.top() + 1)),
        Facing::Down => [0, 1, 2].map(|i| (middle_x + i - 1, map.bottom() - 1)),
    };
    let arrow = match side {
        Facing::Right => "▶",
        Facing::Left => "◀",
        Facing::Up => "▲",
        Facing::Down => "▼",
    };
    for (x, y) in cells {
        buffer.set_string(x, y, arrow, style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shift_moves_the_map_a_column_and_keeps_the_rest() {
        let map = Rect::new(0, 0, 4, 1);
        let mut buffer = Buffer::empty(map);
        buffer.set_string(0, 0, "abcd", Style::new());
        shift(&mut buffer, map, 1);
        let row: String = (0..4).map(|x| buffer[(x, 0)].symbol().to_owned()).collect();
        assert_eq!(row, "aabc");
    }
}
