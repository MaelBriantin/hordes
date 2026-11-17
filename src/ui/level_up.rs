//! The level-up: the choices on offer as cards over the map, dealt face down then turned over
//! one after the other, each with its picture; the selected one framed in gold.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph};

use super::pixels;
use super::sprites::{SpriteId, Sprites};
use crate::game::{Choice, MAX_LEVEL, Passive, Weapon, World};

const CARD_WIDTH: u16 = 24;
const CARD_HEIGHT: u16 = 13;
/// Columns between two cards.
const GAP: u16 = 2;
/// The picture's size, in cells: 8 × 8 pixels.
const PICTURE: (u16, u16) = (8, 4);
const SELECTED: Color = Color::Rgb(240, 200, 60);
const NEW: Color = Color::Rgb(80, 200, 90);
const BACK: Color = Color::Rgb(70, 80, 120);
/// How long a card takes to turn over.
const FLIP: Duration = Duration::from_millis(240);
/// How long after one card the next one starts to turn.
const DEAL: Duration = Duration::from_millis(120);

/// The cards on offer, if any; `shown`: how long they have been on screen.
pub fn render(
    frame: &mut Frame,
    map: Rect,
    world: &World,
    sprites: &Sprites,
    selected: usize,
    shown: Duration,
) {
    let Some(offer) = &world.offer else {
        return;
    };
    let cards = u16::try_from(offer.len()).unwrap_or(1);
    let width = cards * CARD_WIDTH + (cards - 1) * GAP + 2 * (1 + GAP);
    let [row] = Layout::vertical([Constraint::Length(CARD_HEIGHT + 4)])
        .flex(Flex::Center)
        .areas(map);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Clear, area);
    let block = Block::bordered()
        .padding(Padding::new(GAP, GAP, 1, 1))
        .title(
            Line::from(format!(" Level {} ", world.player.level))
                .bold()
                .centered(),
        );
    let inside = block.inner(area);
    frame.render_widget(block, area);

    let slots = Layout::horizontal(offer.iter().map(|_| Constraint::Length(CARD_WIDTH)))
        .spacing(GAP)
        .split(inside);
    for (index, (choice, slot)) in offer.iter().zip(slots.iter()).enumerate() {
        let card = Card {
            index,
            choice: *choice,
            level: world.player.build.next_level(*choice),
            selected: index == selected,
        };
        let started = shown.saturating_sub(DEAL * u32::try_from(index).unwrap_or(0));
        render_card(frame, *slot, sprites, &card, turned(started));
    }
}

struct Card {
    index: usize,
    choice: Choice,
    level: u8,
    selected: bool,
}

/// How far a card has turned over, from 0 (face down) to 1 (face up), `started` after it began.
fn turned(started: Duration) -> f32 {
    (started.as_secs_f32() / FLIP.as_secs_f32()).min(1.0)
}

/// A card, `turn` of the way turned over: it narrows face down, then widens face up.
fn render_card(frame: &mut Frame, slot: Rect, sprites: &Sprites, card: &Card, turn: f32) {
    if turn < 1.0 {
        let face_up = turn >= 0.5;
        let share = (1.0 - 2.0 * turn).abs();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a share of a card's width, between 0 and 24"
        )]
        let width = ((f32::from(CARD_WIDTH) * share).round() as u16).max(2);
        let [area] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(slot);
        let color = if face_up { Color::DarkGray } else { BACK };
        let fill = if face_up { " " } else { "░" };
        let rows = vec![Line::from(fill.repeat(usize::from(width))); usize::from(CARD_HEIGHT)];
        frame.render_widget(
            Paragraph::new(rows)
                .fg(color)
                .block(Block::bordered().border_style(Style::new().fg(color))),
            area,
        );
        return;
    }

    let border = if card.selected {
        Style::new().fg(SELECTED).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::DarkGray)
    };
    let block = Block::bordered()
        .border_style(border)
        .padding(Padding::new(2, 2, 1, 0))
        .title(Line::from(format!(" {} ", card.index + 1)).centered());
    let inside = block.inner(slot);
    frame.render_widget(block, slot);

    let [title, tag, _, picture, _, gain] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(PICTURE.1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inside);
    centred(frame, name(card.choice), Style::new().bold(), title);
    let (tag_text, tag_style) = match card.level {
        0 => (String::new(), Style::new()),
        1 => ("NEW".to_owned(), Style::new().fg(NEW).bold()),
        level => (format!("Lv {level}/{MAX_LEVEL}"), Style::new()),
    };
    centred(frame, &tag_text, tag_style, tag);
    let [picture] = Layout::horizontal([Constraint::Length(PICTURE.0)])
        .flex(Flex::Center)
        .areas(picture);
    pixels::render_sprite(frame.buffer_mut(), picture, sprites, icon(card.choice));
    centred(frame, gain_of(card.choice, card.level), Style::new(), gain);
}

/// Text centred in `area`, wrapped at its width. A line that can't sit exactly in the middle
/// leaves its spare column on the right, as the card's number does in its border: everything on
/// a card leans the same way.
fn centred(frame: &mut Frame, text: &str, style: Style, area: Rect) {
    let width = usize::from(area.width);
    let lines: Vec<Line> = wrap(text, width)
        .into_iter()
        .map(|line| {
            let left = width.saturating_sub(line.chars().count()) / 2;
            Line::styled(format!("{}{line}", " ".repeat(left)), style)
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

/// Words gathered into lines no wider than `width`; a word wider alone gets a line of its own.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

fn icon(choice: Choice) -> SpriteId {
    match choice {
        Choice::Weapon(Weapon::Bolt) => SpriteId::IconBolt,
        Choice::Passive(passive) => match passive {
            Passive::Vitality => SpriteId::IconVitality,
            Passive::Swiftness => SpriteId::IconSwiftness,
            Passive::Magnet => SpriteId::IconMagnet,
            Passive::Might => SpriteId::IconMight,
            Passive::Haste => SpriteId::IconHaste,
        },
        Choice::Heal => SpriteId::IconHeal,
    }
}

fn name(choice: Choice) -> &'static str {
    match choice {
        Choice::Weapon(Weapon::Bolt) => "Bolt",
        Choice::Passive(passive) => match passive {
            Passive::Vitality => "Vitality",
            Passive::Swiftness => "Swiftness",
            Passive::Magnet => "Magnet",
            Passive::Might => "Might",
            Passive::Haste => "Haste",
        },
        Choice::Heal => "Heal",
    }
}

/// What a choice brings, taken at `level`.
fn gain_of(choice: Choice, level: u8) -> &'static str {
    match choice {
        Choice::Weapon(Weapon::Bolt) => match level {
            1 => "A spark at the nearest enemy",
            2 | 4 => "+1 spark per shot",
            3 => "Shoots every 1.5 s",
            _ => "Damage 10 → 20",
        },
        Choice::Passive(passive) => match passive {
            Passive::Vitality => "+20 max HP, healed",
            Passive::Swiftness => "+8% speed",
            Passive::Magnet => "Gems fly in from 1 tile further",
            Passive::Might => "+10% damage",
            Passive::Haste => "−8% weapon cooldowns",
        },
        Choice::Heal => "Heal 20 HP",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_wrap_at_the_width() {
        assert_eq!(
            wrap("Gems fly in from 1 tile further", 18),
            ["Gems fly in from 1", "tile further"]
        );
        assert_eq!(wrap("Heal 20 HP", 18), ["Heal 20 HP"]);
    }

    #[test]
    fn a_card_turns_over_in_its_time() {
        assert!(turned(Duration::ZERO) < f32::EPSILON);
        assert!((turned(FLIP / 2) - 0.5).abs() < 1e-3);
        assert!((turned(FLIP * 3) - 1.0).abs() < f32::EPSILON);
    }
}
