//! The top row of the map: a new weather told quietly. And a horde as the screen tells it, once
//! announced in big letters (see the announcements): a red glow on the edge it comes from,
//! pulsing (every edge for a ring).

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;

use crate::game::{Facing, News, TICKS_PER_SECOND, WeatherNews, World};

const DANGER: Color = Color::Rgb(230, 50, 50);
/// The glow on the edge a horde comes from: its red; how deep it reaches in, in cells across
/// (a row down counting as two); how far along the edge, as a share of its half; how red at
/// most, and at least as it pulses; the frames a pulse takes.
const GLOW_RED: (u8, u8, u8) = (230, 40, 40);
const GLOW_DEPTH: f32 = 10.0;
const GLOW_ALONG: f32 = 0.7;
const GLOW_MOST: f32 = 0.75;
const GLOW_LEAST: f32 = 0.35;
const GLOW_PULSE: u32 = 45;
/// The cells between two hordes' counts, the map showing there.
const COUNT_GAP: u16 = 1;
/// How long news told on a banner stays on the top row, in seconds.
const BANNER_SECONDS: u32 = 3;

/// The horde's banners and arrows over the map.
pub fn render(frame: &mut Frame, map: Rect, world: &World, frames: u32) {
    let left = world.horde_left();
    if left > 0 && world.announcement.is_none() {
        count(frame, map, &world.hordes_left());
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
    glow(frame.buffer_mut(), map, &sides, frames);
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
/// from the other enemies at a glance. Each horde its own count, by its number, while several
/// stand.
fn count(frame: &mut Frame, map: Rect, hordes: &[(u32, u32)]) {
    let counts: Vec<String> = match hordes {
        &[(_, left)] => vec![fl!("horde-left", left = left)],
        _ => hordes
            .iter()
            .map(|&(number, left)| {
                let shown = number + 1;
                fl!("horde-left-of", number = shown, left = left)
            })
            .collect(),
    };
    // Each count a label of its own, the map showing between them.
    let labels: Vec<String> = counts.iter().map(|count| format!(" {count} ")).collect();
    let widths: Vec<u16> = labels
        .iter()
        .map(|label| u16::try_from(label.chars().count()).unwrap_or(map.width))
        .collect();
    let gaps = u16::try_from(labels.len().saturating_sub(1)).unwrap_or(0) * COUNT_GAP;
    let width = widths.iter().sum::<u16>() + gaps;
    let [row] = Layout::vertical([Constraint::Length(1)]).areas(map);
    let [ground] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    let mut x = ground.x;
    for (label, width) in labels.into_iter().zip(widths) {
        let area = Rect::new(x, ground.y, width, 1).intersection(ground);
        frame.render_widget(
            Line::styled(
                label,
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(DANGER)
                    .add_modifier(Modifier::BOLD),
            ),
            area,
        );
        x = x.saturating_add(width + COUNT_GAP);
    }
}

/// A red glow on the middle of the `sides` of the map a horde comes from, reddest on the edge,
/// fading inwards and along it, pulsing slowly.
fn glow(buffer: &mut Buffer, map: Rect, sides: &[Facing], frames: u32) {
    let turn = super::night::share(frames % GLOW_PULSE, GLOW_PULSE) * std::f32::consts::TAU;
    let strength = GLOW_MOST * (GLOW_LEAST + (1.0 - GLOW_LEAST) * (0.5 - 0.5 * turn.cos()));
    let cells = |count: u16| f32::from(count);
    let (width, height) = (cells(map.width), cells(map.height));
    for y in map.top()..map.bottom() {
        for x in map.left()..map.right() {
            let (across, down) = (cells(x - map.left()), cells(y - map.top()));
            // How far in from the side, in cells across, and how far along it from its middle,
            // as a share of its half.
            let reach = sides.iter().fold(0.0_f32, |most, side| {
                let (inward, along) = match side {
                    Facing::Right => (width - 1.0 - across, (down - height / 2.0) / (height / 2.0)),
                    Facing::Left => (across, (down - height / 2.0) / (height / 2.0)),
                    Facing::Up => (down * 2.0, (across - width / 2.0) / (width / 2.0)),
                    Facing::Down => (
                        (height - 1.0 - down) * 2.0,
                        (across - width / 2.0) / (width / 2.0),
                    ),
                };
                let deep = (1.0 - inward / GLOW_DEPTH).max(0.0).powf(1.6);
                let wide = (1.0 - (along / GLOW_ALONG).powi(2)).max(0.0);
                most.max(deep * wide)
            });
            if reach <= 0.0 {
                continue;
            }
            let cell = &mut buffer[(x, y)];
            cell.fg = super::redden(cell.fg, GLOW_RED, strength * reach);
            cell.bg = super::redden(cell.bg, GLOW_RED, strength * reach);
        }
    }
}
