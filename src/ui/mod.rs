//! What the terminal shows: the player's HP and XP bars and the time on top, the map around the
//! player in a frame, the keys below it; over the map, the title screen, the pause menu, the
//! level-up's cards or the game over.

mod level_up;
mod menu;
mod night;
mod pixels;
mod sprites;
mod text;

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::game::{Vec2, World, xp_to_next};
use crate::game::{Weather, weather_progress};
pub use menu::PauseItem;
pub use night::RainDisplay;
pub use sprites::Sprites;

const HELP: &str = "←↑↓→ / WASD / ZQSD move · P / Esc pause";
const HELP_TITLE: &str = "Enter start · B blood · R rain · Esc quit";
const HELP_PAUSED: &str = "↑↓ choose · Enter confirm · ←→ change · P / Esc resume";
const HELP_OVER: &str = "R new run · Esc quit";
const HELP_LEVEL_UP: &str = "←→ / 1 2 3 choose · Enter take · Esc pause";

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
}
/// Below this share of their HP, the player's HP bar shows in red.
const LOW_HP_PERCENT: u32 = 30;
/// The width of the HP and XP bars' insides, in cells.
const BAR: u32 = 12;
const HP_COLOR: Color = Color::Rgb(80, 200, 90);
const LOW_HP_COLOR: Color = Color::Rgb(230, 50, 50);
const XP_COLOR: Color = Color::Rgb(70, 150, 255);
/// The width of the weather's gauge, in cells.
const WEATHER_GAUGE: u8 = 6;

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
    /// Whether blood shows, and how the rain shows.
    pub blood: bool,
    pub rain: RainDisplay,
}

pub fn render(
    frame: &mut Frame,
    world: &World,
    sprites: &Sprites,
    screen: Screen,
    view: ViewState,
) {
    let areas = areas(frame.area());
    let block = Block::bordered();
    let map = block.inner(areas.field);
    frame.render_widget(block, areas.field);
    if screen == Screen::Title {
        menu::render_title(frame, map, sprites, view);
    } else {
        frame.render_widget(status(world), areas.top);
        pixels::render_map(
            frame.buffer_mut(),
            map,
            world,
            sprites,
            view.frame,
            view.blood,
            view.rain,
        );
    }
    let help = match screen {
        Screen::Title => HELP_TITLE,
        Screen::Paused { .. } => HELP_PAUSED,
        Screen::Playing if world.is_over() => HELP_OVER,
        Screen::Playing if world.offer.is_some() => HELP_LEVEL_UP,
        Screen::Playing => HELP,
    };
    frame.render_widget(Line::from(help).dark_gray(), areas.help);
    match screen {
        Screen::Title => return,
        Screen::Paused { selected } => {
            menu::render_pause(frame, map, selected, view);
            return;
        }
        Screen::Playing => {}
    }
    if world.is_over() {
        render_game_over(frame, map, world);
    } else {
        level_up::render(frame, map, world, sprites, view.selected, view.shown);
    }
}

/// The player's HP, their level and XP, the time survived and the kills.
fn status(world: &World) -> Line<'static> {
    let player = &world.player;
    let (hp, max_hp) = (player.hp, player.max_hp());
    let hp_color = if hp * 100 < max_hp * LOW_HP_PERCENT {
        LOW_HP_COLOR
    } else {
        HP_COLOR
    };
    let mut spans = vec![
        Span::raw(" HP "),
        Span::styled(bar(hp, max_hp), hp_color),
        Span::styled(format!(" {hp}"), Style::new().add_modifier(Modifier::BOLD)),
        Span::raw("   Lv "),
        Span::styled(
            player.level.to_string(),
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(bar(player.xp, xp_to_next(player.level)), XP_COLOR),
        Span::raw("   "),
        Span::raw(clock(world.seconds())),
        Span::raw("  "),
        Span::styled(
            weather_name(world.weather()),
            weather_style(world.weather()),
        ),
        Span::raw(" "),
        Span::styled(
            weather_gauge(world.weather(), world.seconds()),
            weather_style(world.weather()),
        ),
        Span::raw(format!("   Kills {}", world.kills)),
    ];
    for (weapon, magazine, full) in world.ammo() {
        let rounds = if magazine.reloading > 0 {
            "reloading".to_owned()
        } else {
            format!("{}/{full}", magazine.rounds)
        };
        spans.push(Span::raw(format!("   {} ", level_up::weapon_name(weapon))));
        spans.push(Span::styled(
            rounds,
            Style::new().add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

/// A share of a whole as a bar: `▕████░░░░░░░░▏`. Rounded down, so it only shows full when it
/// is.
fn bar(part: u32, whole: u32) -> String {
    let filled = (part * BAR / whole.max(1)).min(BAR);
    let filled = usize::try_from(filled).unwrap_or(0);
    let empty = usize::try_from(BAR).unwrap_or(0) - filled;
    format!("▕{}{}▏", "█".repeat(filled), "░".repeat(empty))
}

fn weather_name(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "Clear",
        Weather::Drizzle => "Drizzle",
        Weather::Rain => "Rain",
        Weather::Storm => "Storm",
        Weather::Tempest => "Tempest",
    }
}

/// The weather's colour: grey for a clear night, darkening through blues to red.
fn weather_style(weather: Weather) -> Style {
    let color = match weather {
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

/// Seconds as minutes and seconds: `03:42`.
fn clock(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn render_game_over(frame: &mut Frame, map: Rect, world: &World) {
    let lines = vec![
        Line::from("Game over".bold().red()),
        Line::from(""),
        Line::from(format!("You survived {}", clock(world.seconds()))),
        Line::from(format!(
            "Level {} · {} kills",
            world.player.level, world.kills
        )),
    ];
    let [row] = Layout::vertical([Constraint::Length(6)])
        .flex(Flex::Center)
        .areas(map);
    let [area] = Layout::horizontal([Constraint::Length(28)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).centered().block(Block::bordered()),
        area,
    );
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
