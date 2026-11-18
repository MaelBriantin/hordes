//! News told over the map in big letters, while the world stands still: the weather the run
//! comes into, a horde on its way.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};

use super::font;
use super::menu::DAWN_COLORS;
use super::night;
use super::sprites::Sprites;
use super::text::centred_as_is;
use super::{weather_colour, weather_name};
use crate::game::{News, Weather, World};

/// The share of its light the map keeps under the news.
const DIM: f32 = 0.45;
/// A horde's word: red, darker below.
const HORDE_COLORS: (Color, Color) = (Color::Rgb(230, 50, 50), Color::Rgb(150, 25, 30));
/// How much darker a word's bottom is than its top.
const SHADE: f32 = 0.65;

/// The news over the map, if the world stands still for some: a word in big letters, twice as
/// big where it fits, and a line under it, the map dimmed. At daybreak, the dawn's word, the
/// world going on under it.
pub fn render(frame: &mut Frame, map: Rect, world: &World, sprites: &Sprites) {
    let Some(announcement) = world.announcement else {
        if world.daybreak() && !world.is_over() {
            tell(frame, map, sprites, &fl!("dawn-title"), DAWN_COLORS, None);
        }
        return;
    };
    dim(frame.buffer_mut(), map);
    let (title, colors, line) = match announcement.news {
        News::Weather(weather) => {
            let top = weather_colour(weather);
            let line = match weather {
                Weather::Dusk => fl!("announce-dusk"),
                Weather::Clear => fl!("announce-night"),
                Weather::Drizzle | Weather::Rain | Weather::Storm => fl!("announce-worse"),
                Weather::Tempest => fl!("announce-last"),
            };
            (
                weather_name(weather).to_uppercase(),
                (top, shade(top)),
                Some(line),
            )
        }
        News::Horde { everywhere } => {
            let line = if everywhere {
                fl!("horde-everywhere")
            } else {
                fl!("horde-coming")
            };
            (fl!("horde-title"), HORDE_COLORS, Some(line))
        }
    };
    tell(frame, map, sprites, &title, colors, line.as_deref());
}

/// A word in big letters in the middle of the map, twice as big where it fits, and a line under
/// it.
fn tell(
    frame: &mut Frame,
    map: Rect,
    sprites: &Sprites,
    title: &str,
    colors: (Color, Color),
    line: Option<&str>,
) {
    let wide = font::width(title) * 2 + 4 <= i32::from(map.width);
    let scale = if wide && map.height >= 20 { 2 } else { 1 };
    let rows = u16::try_from(4 * scale).unwrap_or(4);
    let [_, big, _, under, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(rows),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(map);
    font::render(frame.buffer_mut(), big, sprites, title, colors, scale);
    if let Some(line) = line {
        centred_as_is(
            frame,
            line,
            Style::new().fg(colors.0).add_modifier(Modifier::BOLD),
            under,
        );
    }
}

/// A colour, darker: a word's bottom.
fn shade(color: Color) -> Color {
    match color {
        Color::Rgb(red, green, blue) => Color::Rgb(
            night::scale_channel(red, SHADE),
            night::scale_channel(green, SHADE),
            night::scale_channel(blue, SHADE),
        ),
        other => other,
    }
}

/// What the area shows, dimmed: the news stands out over it.
fn dim(buffer: &mut Buffer, area: Rect) {
    let darker = |color: Option<Color>| match color {
        Some(Color::Rgb(red, green, blue)) => Some(Color::Rgb(
            night::scale_channel(red, DIM),
            night::scale_channel(green, DIM),
            night::scale_channel(blue, DIM),
        )),
        _ => None,
    };
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buffer[(x, y)];
            if let Some(fg) = darker(Some(cell.fg)) {
                cell.fg = fg;
            }
            if let Some(bg) = darker(Some(cell.bg)) {
                cell.bg = bg;
            }
        }
    }
}
