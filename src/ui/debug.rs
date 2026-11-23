//! A debug tool, in debug builds only: the console over the map, to try at once what comes late
//! in a run.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph};

use super::{Blank, level_up};
use crate::game::{Choice, World};

/// The console's colour, apart from the game's own.
const ACCENT: Color = Color::Rgb(90, 220, 200);
/// Its width: its longest line is the hint on its bottom border.
const WIDTH: u16 = 40;

/// What the console can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    NextWeather,
    Horde,
    Dawn,
    LevelUp,
    GiveCard,
    ActionUp,
    Heal,
    OneHp,
    Campfire,
    Fire,
    UnlockAll,
    Unhurt,
}

impl DebugAction {
    pub const ALL: [DebugAction; 12] = [
        DebugAction::NextWeather,
        DebugAction::Horde,
        DebugAction::Dawn,
        DebugAction::LevelUp,
        DebugAction::GiveCard,
        DebugAction::ActionUp,
        DebugAction::Heal,
        DebugAction::OneHp,
        DebugAction::Campfire,
        DebugAction::Fire,
        DebugAction::UnlockAll,
        DebugAction::Unhurt,
    ];

    /// Its line in the console, in English: a tool, not part of the game.
    fn label(self, unhurt: bool) -> String {
        match self {
            DebugAction::NextWeather => "Next weather (then sunrise)".into(),
            DebugAction::Horde => "Horde now".into(),
            DebugAction::Dawn => "Dawn in 10 s".into(),
            DebugAction::LevelUp => "Level up".into(),
            DebugAction::GiveCard => "Give a card…".into(),
            DebugAction::ActionUp => "Hero's action +1".into(),
            DebugAction::Heal => "Full heal".into(),
            DebugAction::OneHp => "Down to 1 HP".into(),
            DebugAction::Campfire => "Campfire here".into(),
            DebugAction::Fire => "Set fire around".into(),
            DebugAction::UnlockAll => "Unlock everything".into(),
            DebugAction::Unhurt => format!("Unhurt: {}", if unhurt { "on" } else { "off" }),
        }
    }
}

/// The console over the map, the action `selected` marked.
pub fn render(frame: &mut Frame, map: Rect, selected: usize, unhurt: bool) {
    let lines: Vec<Line> = DebugAction::ALL
        .iter()
        .enumerate()
        .map(|(index, action)| {
            if index == selected {
                Line::styled(
                    format!("▶ {}", action.label(unhurt)),
                    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(format!("  {}", action.label(unhurt)))
            }
        })
        .collect();
    let height = u16::try_from(lines.len()).unwrap_or(u16::MAX) + 2;
    let area = map.centered(Constraint::Length(WIDTH), Constraint::Length(height));
    let block = Block::bordered()
        .padding(Padding::horizontal(1))
        .border_style(Style::new().fg(ACCENT))
        .title(" Debug ")
        .title_bottom(Line::from(" ↑↓ choose · Enter do · Esc back ").right_aligned());
    frame.render_widget(Blank, area);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// The cards the console can give, the one `selected` marked, each with its level: as many as
/// the map has room for, around the one marked.
pub fn render_cards(frame: &mut Frame, map: Rect, world: &World, selected: usize) {
    let cards = world.debug_cards();
    let build = &world.player.build;
    let room = usize::from(map.height.saturating_sub(4)).max(1);
    let first = selected
        .saturating_sub(room / 2)
        .min(cards.len().saturating_sub(room));
    let lines: Vec<Line> = cards
        .iter()
        .enumerate()
        .skip(first)
        .take(room)
        .map(|(index, &choice)| {
            let level = match choice {
                Choice::Weapon(weapon) => build.weapon_level(weapon),
                Choice::Passive(passive) => build.passive_level(passive),
                Choice::Legendary(legendary) => u8::from(build.has(legendary)),
                Choice::Action(action) => build.action_level(action),
                Choice::Heal => 0,
            };
            let full = !build.can_take(choice);
            let text = format!(
                "{:<26}{}",
                level_up::name(choice),
                if level > 0 {
                    format!("lv {level}")
                } else {
                    String::new()
                }
            );
            if index == selected {
                Line::styled(
                    format!("▶ {text}"),
                    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
                )
            } else if full {
                Line::styled(format!("  {text}"), Style::new().fg(Color::DarkGray))
            } else {
                Line::from(format!("  {text}"))
            }
        })
        .collect();
    let height = u16::try_from(lines.len()).unwrap_or(u16::MAX) + 2;
    let area = map.centered(Constraint::Length(WIDTH), Constraint::Length(height));
    let block = Block::bordered()
        .padding(Padding::horizontal(1))
        .border_style(Style::new().fg(ACCENT))
        .title(" Give a card ")
        .title_bottom(Line::from(" Enter +1 level · Esc back ").right_aligned());
    frame.render_widget(Blank, area);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
