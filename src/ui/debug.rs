//! A debug tool, in debug builds only: the console over the map, to try at once what comes late
//! in a run.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph};

use super::Blank;

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
    Heal,
    Campfire,
    Unhurt,
}

impl DebugAction {
    pub const ALL: [DebugAction; 7] = [
        DebugAction::NextWeather,
        DebugAction::Horde,
        DebugAction::Dawn,
        DebugAction::LevelUp,
        DebugAction::Heal,
        DebugAction::Campfire,
        DebugAction::Unhurt,
    ];

    /// Its line in the console, in English: a tool, not part of the game.
    fn label(self, unhurt: bool) -> String {
        match self {
            DebugAction::NextWeather => "Next weather (then sunrise)".into(),
            DebugAction::Horde => "Horde now".into(),
            DebugAction::Dawn => "Dawn in 10 s".into(),
            DebugAction::LevelUp => "Level up".into(),
            DebugAction::Heal => "Full heal".into(),
            DebugAction::Campfire => "Campfire here".into(),
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
