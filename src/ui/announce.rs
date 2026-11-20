//! News told over the map in big letters, while the world stands still: the weather the run
//! comes into, a horde on its way. A horde wiped out and the dawn are told the same way, the
//! world going on.

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
use crate::game::{News, TICKS_PER_SECOND, Weather, WeatherNews, World};

/// The share of its light the map keeps under the news.
const DIM: f32 = 0.45;
/// A horde's word: red, darker below.
const HORDE_COLORS: (Color, Color) = (Color::Rgb(230, 50, 50), Color::Rgb(150, 25, 30));
/// A horde wiped out: its words in gold, as long as they stay, in seconds, the run going on.
const CLEARED_COLORS: (Color, Color) = (Color::Rgb(240, 200, 60), Color::Rgb(190, 130, 30));
const CLEARED_SECONDS: u32 = 2;
/// How long a new weather stays in big letters, the run going on, in seconds.
const BIG_NEWS_SECONDS: u32 = 2;
/// How much darker a word's bottom is than its top.
const SHADE: f32 = 0.65;

/// The news over the map, if the world stands still for some: a word in big letters, twice as
/// big where it fits, and a line under it, the map dimmed. At daybreak, the dawn's word, the
/// world going on under it.
pub fn render(frame: &mut Frame, map: Rect, world: &World, sprites: &Sprites) {
    let Some(Title {
        text,
        colors,
        line,
        dimmed,
    }) = title(world)
    else {
        return;
    };
    if dimmed {
        dim(frame.buffer_mut(), map);
    }
    tell(frame, map, sprites, &text, colors, line.as_deref());
}

/// Whether a word in big letters shows over the map.
pub fn showing(world: &World) -> bool {
    title(world).is_some()
}

/// A word in big letters over the map: its colours, the line under it, and whether the map
/// dims under it, the world standing still.
struct Title {
    text: String,
    colors: (Color, Color),
    line: Option<String>,
    dimmed: bool,
}

/// The word in big letters the map shows now, if any.
fn title(world: &World) -> Option<Title> {
    let going_on = |(text, colors, line): (String, (Color, Color), Option<String>)| Title {
        text,
        colors,
        line,
        dimmed: false,
    };
    if let Some(announcement) = world.announcement {
        let (text, colors, line) = words(announcement.news);
        return Some(Title {
            text,
            colors,
            line,
            dimmed: true,
        });
    }
    if world.daybreak() && !world.is_over() {
        return Some(going_on((fl!("dawn-title"), DAWN_COLORS, None)));
    }
    if world.weather_news == WeatherNews::Big
        && let Some((news, at)) = world.told
        && world.ticks().saturating_sub(at) < BIG_NEWS_SECONDS * u32::from(TICKS_PER_SECOND)
    {
        // News in big letters, the run going on.
        return Some(going_on(words(news)));
    }
    if let Some((at, bonus, number)) = world.horde_cleared
        && world.ticks().saturating_sub(at) < CLEARED_SECONDS * u32::from(TICKS_PER_SECOND)
    {
        // A level, and in arcade the points too; which horde, while another still stands.
        let mut line = if bonus > 0 {
            fl!("horde-cleared-points", points = bonus)
        } else {
            fl!("horde-cleared-level")
        };
        if let Some(number) = number {
            let shown = number + 1;
            line = format!("{} · {line}", fl!("horde-number", number = shown));
        }
        return Some(going_on((fl!("horde-cleared"), CLEARED_COLORS, Some(line))));
    }
    None
}

/// The line under the big letters: white, the word above carrying the colour.
const LINE: Color = Color::Rgb(235, 235, 240);

/// A word in big letters in the middle of the map, twice as big where it fits, and a line in
/// capitals under it, in white.
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
            &line.to_uppercase(),
            Style::new().fg(LINE).add_modifier(Modifier::BOLD),
            under,
        );
    }
}

/// News as told in big letters: its word, its colours, and its line under it.
fn words(news: News) -> (String, (Color, Color), Option<String>) {
    match news {
        News::Weather(weather) => {
            let top = weather_colour(weather);
            (
                weather_name(weather).to_uppercase(),
                (top, shade(top)),
                Some(weather_line(weather)),
            )
        }
        News::Horde { everywhere } => (
            fl!("horde-title"),
            HORDE_COLORS,
            Some(horde_line(everywhere)),
        ),
    }
}

/// The line under a horde's word: where it comes from.
pub fn horde_line(everywhere: bool) -> String {
    if everywhere {
        fl!("horde-everywhere")
    } else {
        fl!("horde-coming")
    }
}

/// The line under a new weather's name.
fn weather_line(weather: Weather) -> String {
    match weather {
        Weather::Dusk => fl!("announce-dusk"),
        Weather::Clear => fl!("announce-night"),
        Weather::Drizzle | Weather::Rain | Weather::Storm => fl!("announce-worse"),
        Weather::Tempest => fl!("announce-last"),
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

/// What the area shows, dimmed: the news, or a menu, stands out over it.
pub fn dim(buffer: &mut Buffer, area: Rect) {
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
