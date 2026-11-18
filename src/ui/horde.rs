//! The top row of the map: a new weather told quietly. And a horde as the screen tells it, once announced in big letters (see the announcements): a
//! blinking band of arrows on the edge it comes from (every edge for a ring).

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;

use crate::game::{Facing, News, TICKS_PER_SECOND, WeatherNews, World};

const DANGER: Color = Color::Rgb(230, 50, 50);
const DANGER_DIM: Color = Color::Rgb(120, 22, 26);
/// The band of arrows: so many cells across a top or bottom edge, so many rows down a side
/// (two cells deep); the frames it stays bright, then dark.
const ARROWS_ACROSS: u16 = 9;
const ARROWS_DOWN: u16 = 5;
const ARROWS_BLINK: u32 = 6;
/// How long news told on a banner stays on the top row, in seconds.
const BANNER_SECONDS: u32 = 3;

/// The horde's banners and arrows over the map.
pub fn render(frame: &mut Frame, map: Rect, world: &World, frames: u32) {
    let left = world.horde_left();
    if left > 0 && world.announcement.is_none() {
        count(frame, map, left);
    }
    // News told on a banner: a new weather's name on the top row a while, the horde's count
    // over it; a horde on its way, in red.
    if let Some((news, at)) = world.told
        && world.weather_news == WeatherNews::Banner
        && world.ticks().saturating_sub(at) < BANNER_SECONDS * u32::from(TICKS_PER_SECOND)
    {
        match news {
            News::Weather(weather) if left == 0 => banner(
                frame,
                map,
                &super::weather_name(weather).to_uppercase(),
                super::weather_colour(weather),
            ),
            News::Weather(_) => {}
            News::Horde { everywhere } => banner(
                frame,
                map,
                &format!(
                    "{} · {}",
                    fl!("horde-title"),
                    super::announce::horde_line(everywhere)
                ),
                DANGER,
            ),
        }
    }
    let Some(warning) = world.horde_warning else {
        return;
    };
    let sides = match warning.side {
        Some(side) => vec![side],
        None => vec![Facing::Up, Facing::Down, Facing::Left, Facing::Right],
    };
    let lit = (frames / ARROWS_BLINK).is_multiple_of(2);
    for side in sides {
        arrows(frame.buffer_mut(), map, side, lit);
    }
}

/// A few words on the top row of the map, dark on a colour.
fn banner(frame: &mut Frame, map: Rect, words: &str, colour: Color) {
    let words = format!(" {words} ");
    let width = u16::try_from(words.chars().count()).unwrap_or(map.width);
    let [row] = Layout::vertical([Constraint::Length(1)]).areas(map);
    let [ground] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(
        Line::styled(
            words,
            Style::new()
                .fg(Color::Rgb(20, 20, 26))
                .bg(colour)
                .add_modifier(Modifier::BOLD),
        ),
        ground,
    );
}

/// How many of the hordes still stand, on the top row of the map, white on red: they are told
/// from the other enemies at a glance.
fn count(frame: &mut Frame, map: Rect, left: u32) {
    let words = format!(" {} ", fl!("horde-left", left = left));
    let width = u16::try_from(words.chars().count()).unwrap_or(map.width);
    let [row] = Layout::vertical([Constraint::Length(1)]).areas(map);
    let [ground] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(
        Line::styled(
            words,
            Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(DANGER)
                .add_modifier(Modifier::BOLD),
        ),
        ground,
    );
}

/// A band of arrows on the middle of an edge of the map, white on red, pointing in from where
/// the horde comes: brighter, then darker, as it blinks.
fn arrows(buffer: &mut Buffer, map: Rect, side: Facing, lit: bool) {
    let style = Style::new()
        .fg(Color::Rgb(255, 255, 255))
        .bg(if lit { DANGER } else { DANGER_DIM })
        .add_modifier(Modifier::BOLD);
    let (middle_x, middle_y) = (map.x + map.width / 2, map.y + map.height / 2);
    let across = ARROWS_ACROSS;
    let down = ARROWS_DOWN;
    let cells: Vec<(u16, u16)> = match side {
        Facing::Right => (0..down)
            .flat_map(|i| (0..2).map(move |d| (map.right() - 1 - d, middle_y + i - down / 2)))
            .collect(),
        Facing::Left => (0..down)
            .flat_map(|i| (0..2).map(move |d| (map.left() + d, middle_y + i - down / 2)))
            .collect(),
        Facing::Up => (0..across)
            .map(|i| (middle_x + i - across / 2, map.top()))
            .collect(),
        Facing::Down => (0..across)
            .map(|i| (middle_x + i - across / 2, map.bottom() - 1))
            .collect(),
    };
    let arrow = match side {
        Facing::Right => "◀",
        Facing::Left => "▶",
        Facing::Up => "▼",
        Facing::Down => "▲",
    };
    for (x, y) in cells {
        buffer.set_string(x, y, arrow, style);
    }
}
