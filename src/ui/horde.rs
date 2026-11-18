//! A horde as the screen tells it, once announced in big letters (see the announcements): arrows
//! on the edge it comes from (every edge for a ring); wiped out, a banner in
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
/// How long the banner of a horde wiped out stays, in seconds.
const CLEARED_SECONDS: u32 = 2;

/// The horde's banners and arrows over the map.
pub fn render(frame: &mut Frame, map: Rect, world: &World) {
    if let Some(warning) = world.horde_warning {
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
