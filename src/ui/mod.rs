//! What the terminal shows: the player's HP and XP bars and the time on top, the map around the player in a
//! frame, the keys below it; when the run is over, a word over the map.

mod level_up;
mod pixels;
mod sprites;

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::game::{Vec2, World, xp_to_next};
pub use sprites::Sprites;

const HELP: &str = "←↑↓→ / WASD / ZQSD move · Esc quit";
const HELP_OVER: &str = "R new run · Esc quit";
const HELP_LEVEL_UP: &str = "←→ / 1 2 3 choose · Enter take · Esc quit";
/// Below this share of their HP, the player's HP bar shows in red.
const LOW_HP_PERCENT: u32 = 30;
/// The width of the HP and XP bars' insides, in cells.
const BAR: u32 = 12;
const HP_COLOR: Color = Color::Rgb(80, 200, 90);
const LOW_HP_COLOR: Color = Color::Rgb(230, 50, 50);
const XP_COLOR: Color = Color::Rgb(70, 150, 255);

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

/// A level-up as the player sees it: the card selected, and how long the cards have been shown.
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelUpView {
    pub selected: usize,
    pub shown: Duration,
}

pub fn render(frame: &mut Frame, world: &World, sprites: &Sprites, level_up: LevelUpView) {
    let areas = areas(frame.area());
    frame.render_widget(status(world), areas.top);
    let block = Block::bordered();
    let map = block.inner(areas.field);
    frame.render_widget(block, areas.field);
    pixels::render_map(frame.buffer_mut(), map, world, sprites);
    let help = if world.is_over() {
        HELP_OVER
    } else if world.offer.is_some() {
        HELP_LEVEL_UP
    } else {
        HELP
    };
    frame.render_widget(Line::from(help).dark_gray(), areas.help);
    if world.is_over() {
        render_game_over(frame, map, world);
    } else {
        level_up::render(
            frame,
            map,
            world,
            sprites,
            level_up.selected,
            level_up.shown,
        );
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
    Line::from(vec![
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
        Span::raw(format!("   Kills {}", world.kills)),
    ])
}

/// A share of a whole as a bar: `▕████░░░░░░░░▏`. Rounded down, so it only shows full when it
/// is.
fn bar(part: u32, whole: u32) -> String {
    let filled = (part * BAR / whole.max(1)).min(BAR);
    let filled = usize::try_from(filled).unwrap_or(0);
    let empty = usize::try_from(BAR).unwrap_or(0) - filled;
    format!("▕{}{}▏", "█".repeat(filled), "░".repeat(empty))
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
