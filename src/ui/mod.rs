//! What the terminal shows: the player's HP and XP bars and the time on top, the map around the
//! player in a frame, the keys below it; over the map, the title screen, the pause menu, the
//! level-up's cards or the game over.

mod font;
mod horde;
mod icons;
mod level_up;
mod menu;
mod night;
mod pixels;
mod sprites;
mod text;

use std::time::Duration;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Widget};

use crate::game::{Leaderboard, Vec2, World, xp_to_next};
use crate::game::{Weather, weather_progress};
pub use menu::PauseItem;
pub use night::{BloodLook, RainDisplay};
pub use sprites::Sprites;

/// Where the player is, outside the run itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The game waits for the player to start.
    Title,
    Playing,
    /// The run stands still, the pause menu open on its item `selected`.
    Paused {
        selected: usize,
    },
    /// The run is over with one of the best scores: three letters to type in, arcade style, the
    /// one at `cursor` being changed.
    NameEntry {
        letters: [u8; 3],
        cursor: usize,
    },
}
/// Below this share of their HP, the player's HP bar blinks.
const LOW_HP_PERCENT: u32 = 30;
/// The frames the low HP bar stays lit, then dimmed, as it blinks.
const LOW_HP_BLINK: u32 = 8;
/// The width of the HP bar's inside, in cells.
const BAR_CELLS: usize = 12;
/// The HP bar's red, the herbs': and dimmed, as it blinks.
const HP_COLOR: Color = Color::Rgb(220, 45, 65);
const HP_DIMMED: Color = Color::Rgb(90, 20, 28);
const XP_COLOR: Color = Color::Rgb(70, 150, 255);
const SCORE_COLOR: Color = Color::Rgb(240, 200, 60);
/// The width of the weather's gauge, in cells.
const WEATHER_GAUGE: u8 = 6;
/// The game's own background and text, whatever the terminal's theme: a light one would wash
/// the night out.
const SCREEN: Style = Style::new()
    .fg(Color::Rgb(225, 225, 230))
    .bg(Color::Rgb(12, 12, 18));
/// Greys for what is said aside, and for frames.
pub const GREY: Color = Color::Rgb(165, 165, 172);
pub const DARK_GREY: Color = Color::Rgb(105, 105, 115);

/// Empties an area in the game's own colours: under a box or a banner, or the whole screen.
pub struct Blank;

impl Widget for Blank {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Clear.render(area, buffer);
        buffer.set_style(area, SCREEN);
    }
}

/// The screen's parts: the line on top, the map in its frame, the keys below.
struct Areas {
    top: Rect,
    field: Rect,
    help: Rect,
}

fn areas(screen: Rect) -> Areas {
    let [top, field, help] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(screen);
    Areas { top, field, help }
}

/// Half of the map a screen of that size shows each way, in tiles.
pub fn view(screen: Rect) -> Vec2 {
    pixels::view(Block::bordered().inner(areas(screen).field))
}

/// What the screen needs beyond the run: the card selected on a level-up and how long the
/// cards have been shown, and the frames drawn so far, for what moves on its own.
#[derive(Debug, Clone, Copy)]
pub struct ViewState {
    pub selected: usize,
    pub shown: Duration,
    pub frame: u32,
    /// How blood shows, and how the rain shows.
    pub blood: BloodLook,
    pub rain: RainDisplay,
    /// The language the game speaks, by its own name: `Français`.
    pub language: &'static str,
    /// The leaderboard's place of the run just over, if it made it.
    pub highlight: Option<usize>,
}

pub fn render(
    frame: &mut Frame,
    world: &World,
    sprites: &Sprites,
    leaderboard: &Leaderboard,
    screen: Screen,
    view: ViewState,
) {
    frame.render_widget(Blank, frame.area());
    let areas = areas(frame.area());
    let block = Block::bordered();
    let map = block.inner(areas.field);
    frame.render_widget(block, areas.field);
    // The run over, the death screen, but for the pause menu over the run.
    let dead = world.is_over() && !matches!(screen, Screen::Paused { .. });
    if screen == Screen::Title {
        menu::render_title(frame, map, sprites, view, leaderboard);
    } else if dead {
        let name = match screen {
            Screen::NameEntry { letters, cursor } => Some((letters, cursor)),
            _ => None,
        };
        menu::render_death(frame, map, sprites, view, world, leaderboard, name);
    } else {
        render_status(frame, areas.top, world, leaderboard, view.frame);
        pixels::render_map(
            frame.buffer_mut(),
            map,
            world,
            sprites,
            view.frame,
            view.blood,
            view.rain,
        );
        horde::shift(frame.buffer_mut(), map, horde::shake(world));
        if let Some(healing) = world.compass() {
            pixels::render_compass(
                frame.buffer_mut(),
                map,
                world.player.position,
                healing,
                view.frame,
            );
        }
    }
    // The keys below, but for the run going on: its XP bar, all along the screen.
    let help = match screen {
        Screen::Title => Some(fl!("help-title")),
        Screen::Paused { .. } => Some(fl!("help-paused")),
        Screen::NameEntry { .. } => Some(fl!("help-name")),
        Screen::Playing if world.is_over() => Some(fl!("help-over")),
        Screen::Playing if world.offer.is_some() => Some(fl!("help-level-up")),
        Screen::Playing => None,
    };
    match help {
        Some(help) => frame.render_widget(Line::from(help).fg(DARK_GREY), areas.help),
        None => frame.render_widget(experience(world, areas.help.width), areas.help),
    }
    match screen {
        Screen::Title | Screen::NameEntry { .. } => return,
        Screen::Paused { selected } => {
            menu::render_pause(frame, map, selected, view);
            return;
        }
        Screen::Playing => {}
    }
    if !dead {
        horde::render(frame, map, world, view.frame);
        level_up::render(frame, map, world, sprites, view.selected, view.shown);
    }
}

/// The status line, in two parts: on the left, what keeps the player alive (their HP and the
/// weather); on the right, how the run goes (the time and the score).
fn render_status(
    frame: &mut Frame,
    area: Rect,
    world: &World,
    leaderboard: &Leaderboard,
    frames: u32,
) {
    frame.render_widget(survival(world, frames), area);
    frame.render_widget(run(world, leaderboard, frames).right_aligned(), area);
}

/// The player's HP and the weather, as gauges.
fn survival(world: &World, frames: u32) -> Line<'static> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let player = &world.player;
    let (hp, max_hp) = (player.hp, player.max_hp());
    let low = hp * 100 < max_hp * LOW_HP_PERCENT;
    let hp_color = if low && (frames / LOW_HP_BLINK).is_multiple_of(2) {
        HP_DIMMED
    } else {
        HP_COLOR
    };
    let weather = world.weather();
    Line::from(vec![
        Span::styled(format!(" {} ", icons::HP), bold.fg(HP_COLOR)),
        Span::styled(bar(hp, max_hp), hp_color),
        Span::styled(format!(" {hp}"), bold),
        Span::styled(
            format!("   {} ", icons::weather(weather)),
            weather_style(weather),
        ),
        Span::styled(weather_name(weather), weather_style(weather)),
        Span::raw(" "),
        Span::styled(
            weather_gauge(weather, world.seconds()),
            weather_style(weather),
        ),
    ])
}

/// The player's level and their XP, in a bar as wide as `width` leaves.
fn experience(world: &World, width: u16) -> Line<'static> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let player = &world.player;
    let mut spans = vec![
        Span::styled(format!(" {} ", icons::LEVEL), bold.fg(XP_COLOR)),
        Span::styled(format!("{} ", player.level), bold),
    ];
    let taken: usize = spans.iter().map(Span::width).sum();
    // The bar's two ends take a cell each, and a cell is left after it.
    let cells = usize::from(width).saturating_sub(taken + 3);
    spans.push(Span::styled(
        bar_of(player.xp, xp_to_next(player.level), cells),
        XP_COLOR,
    ));
    Line::from(spans)
}

/// The time survived and the score, with its multiplier: the score gleams while ahead of the
/// best run kept, as a new high score does.
fn run(world: &World, leaderboard: &Leaderboard, frames: u32) -> Line<'static> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let mut spans = vec![
        Span::styled(format!("{} ", icons::TIME), bold),
        Span::styled(clock(world.seconds()), bold),
        Span::styled(format!("   {} ", icons::SCORE), bold.fg(SCORE_COLOR)),
    ];
    let score = world.score.to_string();
    let best = leaderboard.entries().first().map(|entry| entry.score);
    if best.is_some_and(|best| world.score > best) {
        spans.extend(text::shimmer(&score, frames));
    } else {
        spans.push(Span::styled(score, bold.fg(SCORE_COLOR)));
    }
    spans.extend([
        Span::raw(" "),
        Span::styled(multiplier(world.multiplier_tenths()), SCORE_COLOR),
        Span::raw(" "),
    ]);
    Line::from(spans)
}

/// A share of a whole as a bar: `▕████░░░░░░░░▏`. Rounded down, so it only shows full when it
/// is.
fn bar(part: u32, whole: u32) -> String {
    bar_of(part, whole, BAR_CELLS)
}

/// The same bar, `cells` wide inside its ends.
fn bar_of(part: u32, whole: u32, cells: usize) -> String {
    let cells_u64 = u64::try_from(cells).unwrap_or(0);
    let filled = (u64::from(part) * cells_u64 / u64::from(whole.max(1))).min(cells_u64);
    let filled = usize::try_from(filled).unwrap_or(0);
    format!("▕{}{}▏", "█".repeat(filled), "░".repeat(cells - filled))
}

fn weather_name(weather: Weather) -> String {
    match weather {
        Weather::Dusk => fl!("weather-dusk"),
        Weather::Clear => fl!("weather-clear"),
        Weather::Drizzle => fl!("weather-drizzle"),
        Weather::Rain => fl!("weather-rain"),
        Weather::Storm => fl!("weather-storm"),
        Weather::Tempest => fl!("weather-tempest"),
    }
}

/// The weather's colour: orange at dusk, grey for a clear night, darkening through blues to red.
fn weather_style(weather: Weather) -> Style {
    let color = match weather {
        Weather::Dusk => Color::Rgb(240, 150, 70),
        Weather::Clear => Color::Rgb(160, 160, 170),
        Weather::Drizzle => Color::Rgb(130, 170, 220),
        Weather::Rain => Color::Rgb(80, 120, 230),
        Weather::Storm => Color::Rgb(170, 90, 230),
        Weather::Tempest => Color::Rgb(230, 50, 50),
    };
    Style::new().fg(color).add_modifier(Modifier::BOLD)
}

/// How far the run is to the next weather, as a short bar; full in the tempest, the last.
fn weather_gauge(weather: Weather, seconds: u32) -> String {
    let share = if weather.next().is_some() {
        weather_progress(seconds)
    } else {
        1.0
    };
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a share of 6 cells"
    )]
    let filled =
        ((share * f32::from(WEATHER_GAUGE)).floor() as usize).min(usize::from(WEATHER_GAUGE));
    format!(
        "▕{}{}▏",
        "█".repeat(filled),
        "░".repeat(usize::from(WEATHER_GAUGE) - filled)
    )
}

/// A multiplier given in tenths: `×2.4`.
fn multiplier(tenths: u64) -> String {
    let (whole, tenth) = (tenths / 10, tenths % 10);
    fl!("multiplier", whole = whole, tenth = tenth)
}

/// Seconds as minutes and seconds: `03:42`.
pub fn clock(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clock_shows_minutes_and_seconds() {
        assert_eq!(clock(0), "00:00");
        assert_eq!(clock(222), "03:42");
        assert_eq!(clock(600), "10:00");
    }

    #[test]
    fn a_bar_fills_with_its_share() {
        assert_eq!(bar(0, 10), "▕░░░░░░░░░░░░▏");
        assert_eq!(bar(5, 10), "▕██████░░░░░░▏");
        assert_eq!(bar(10, 10), "▕████████████▏");
        assert_eq!(bar(99, 100), "▕███████████░▏");
    }
}
