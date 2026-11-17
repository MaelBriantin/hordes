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
use super::text::centred;
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

fn icon(choice: Choice) -> SpriteId {
    match choice {
        Choice::Weapon(weapon) => match weapon {
            Weapon::Machete => SpriteId::IconMachete,
            Weapon::Pistol => SpriteId::IconPistol,
            Weapon::MachineGun => SpriteId::IconMachineGun,
            Weapon::Drones => SpriteId::IconDrones,
        },
        Choice::Passive(passive) => match passive {
            Passive::Vitality => SpriteId::IconVitality,
            Passive::Swiftness => SpriteId::IconSwiftness,
            Passive::Magnet => SpriteId::IconMagnet,
            Passive::Might => SpriteId::IconMight,
            Passive::Haste => SpriteId::IconHaste,
            Passive::Flashlight => SpriteId::IconFlashlight,
        },
        Choice::Heal => SpriteId::IconHeal,
    }
}

fn name(choice: Choice) -> &'static str {
    match choice {
        Choice::Weapon(weapon) => weapon_name(weapon),
        Choice::Passive(passive) => match passive {
            Passive::Vitality => "Vitality",
            Passive::Swiftness => "Swiftness",
            Passive::Magnet => "Magnet",
            Passive::Might => "Might",
            Passive::Haste => "Haste",
            Passive::Flashlight => "Flashlight",
        },
        Choice::Heal => "Heal",
    }
}

pub fn weapon_name(weapon: Weapon) -> &'static str {
    match weapon {
        Weapon::Machete => "Machete",
        Weapon::Pistol => "Pistol",
        Weapon::MachineGun => "Machine gun",
        Weapon::Drones => "Drones",
    }
}

/// What a choice brings, taken at `level`.
fn gain_of(choice: Choice, level: u8) -> &'static str {
    match choice {
        Choice::Weapon(Weapon::Machete) => match level {
            1 => "Slashes enemies in front",
            2 => "Reach 2.5 → 3.5",
            3 => "A second slash, behind",
            4 => "Wider swing",
            _ => "A third slash, ahead",
        },
        Choice::Weapon(Weapon::Pistol) => match level {
            1 => "6 rounds at the nearest enemies",
            2 => "Magazine 6 → 9",
            3 => "Reloads in 1.5 s",
            4 => "+1 bullet per shot",
            _ => "Magazine 12, damage 15",
        },
        Choice::Weapon(Weapon::MachineGun) => match level {
            1 => "Bursts of 20 rounds",
            2 => "Magazine 20 → 30",
            3 => "Damage 4 → 6",
            4 => "Reloads in 2 s",
            _ => "Bullets go through an enemy",
        },
        Choice::Weapon(Weapon::Drones) => match level {
            1 => "A drone circling you",
            2 | 4 => "+1 drone",
            3 => "Damage 8 → 12",
            _ => "Wider and faster",
        },
        Choice::Passive(passive) => match passive {
            Passive::Vitality => "+20 max HP, healed",
            Passive::Swiftness => "+8% speed",
            Passive::Magnet => "Gems fly in from 1 tile further",
            Passive::Might => "+10% damage",
            Passive::Haste => "−8% weapon cooldowns",
            Passive::Flashlight => "+2 tiles of light",
        },
        Choice::Heal => "Heal 20 HP",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_turns_over_in_its_time() {
        assert!(turned(Duration::ZERO) < f32::EPSILON);
        assert!((turned(FLIP / 2) - 0.5).abs() < 1e-3);
        assert!((turned(FLIP * 3) - 1.0).abs() < f32::EPSILON);
    }
}
