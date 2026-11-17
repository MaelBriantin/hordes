//! The title screen and the pause menu, over the map.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph};

use super::night;
use super::pixels;
use super::sprites::{SpriteId, Sprites};
use super::text::centred;
use super::{RainDisplay, ViewState};

const SELECTED: Color = Color::Rgb(240, 200, 60);

/// What the pause menu offers, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseItem {
    Resume,
    /// Blood shown or not.
    Blood,
    /// How the rain shows.
    Rain,
    /// The language the game speaks.
    Language,
    NewRun,
    Quit,
}

impl PauseItem {
    pub const ALL: [PauseItem; 6] = [
        PauseItem::Resume,
        PauseItem::Blood,
        PauseItem::Rain,
        PauseItem::Language,
        PauseItem::NewRun,
        PauseItem::Quit,
    ];

    fn label(self, view: ViewState) -> String {
        match self {
            PauseItem::Resume => fl!("pause-resume"),
            PauseItem::Blood => fl!("pause-blood", value = on_off(view.blood)),
            PauseItem::Rain => fl!("pause-rain", value = rain_name(view.rain)),
            PauseItem::Language => fl!("pause-language", value = view.language),
            PauseItem::NewRun => fl!("pause-new-run"),
            PauseItem::Quit => fl!("pause-quit"),
        }
    }
}

fn on_off(on: bool) -> String {
    if on {
        fl!("setting-on")
    } else {
        fl!("setting-off")
    }
}

fn rain_name(rain: RainDisplay) -> String {
    match rain {
        RainDisplay::Full => fl!("rain-full"),
        RainDisplay::Reduced => fl!("rain-reduced"),
        RainDisplay::Off => fl!("rain-off"),
    }
}

/// A box of `width` × `height` in the middle of `map`, cleared, framed.
fn boxed(frame: &mut Frame, map: Rect, width: u16, height: u16, title: &str) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(map);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Clear, area);
    let block = Block::bordered().padding(Padding::new(2, 2, 1, 1));
    let block = if title.is_empty() {
        block
    } else {
        block.title(Line::from(format!(" {title} ")).centered())
    };
    let inside = block.inner(area);
    frame.render_widget(block, area);
    inside
}

/// The title screen, `frames` frames in: the game's name, big, over a stormy night; the game
/// waits for the player.
pub fn render_title(frame: &mut Frame, area: Rect, sprites: &Sprites, view: ViewState) {
    let frames = view.frame;
    night::render_storm(frame.buffer_mut(), area, sprites, frames);
    // Twice as big where it fits: 70 columns, 7 rows.
    let scale = if area.width >= 74 && area.height >= 14 {
        2
    } else {
        1
    };
    let name_rows = u16::try_from(4 * scale).unwrap_or(4);
    let [_, name, _, line, _, start, _, option, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(name_rows),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Fill(2),
    ])
    .areas(area);
    pixels::render_sprite_scaled(frame.buffer_mut(), name, sprites, SpriteId::Title, scale);
    centred(
        frame,
        &fl!("title-tagline"),
        Style::new().fg(Color::Gray),
        line,
    );
    // Blinking, as on an arcade cabinet: shown two thirds of each second.
    if frames % 30 < 20 {
        centred(
            frame,
            &fl!("title-start"),
            Style::new().add_modifier(Modifier::BOLD),
            start,
        );
    }
    centred(
        frame,
        &fl!(
            "title-options",
            blood = on_off(view.blood),
            rain = rain_name(view.rain),
            language = view.language
        ),
        Style::new().fg(Color::DarkGray),
        option,
    );
}

/// The pause menu, `selected` its highlighted item, the settings as `view` holds them.
pub fn render_pause(frame: &mut Frame, map: Rect, selected: usize, view: ViewState) {
    let height = u16::try_from(PauseItem::ALL.len()).unwrap_or(3) + 4;
    let inside = boxed(frame, map, 28, height, &fl!("pause-title"));
    let lines: Vec<Line> = PauseItem::ALL
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == selected {
                Line::styled(
                    format!("▶ {}", item.label(view)),
                    Style::new().fg(SELECTED).add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(format!("  {}", item.label(view)))
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inside);
}
