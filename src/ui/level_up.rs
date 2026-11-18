//! The level-up: the choices on offer as cards over the map, dealt face down then turned over
//! one after the other, each with its picture; the selected one framed in red.

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
const SELECTED: Color = Color::Rgb(230, 55, 55);
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
            Line::from(format!(
                " {} ",
                fl!("level-up-title", level = world.player.level)
            ))
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
    centred(frame, &name(card.choice), Style::new().bold(), title);
    let (tag_text, tag_style) = match card.level {
        0 => (String::new(), Style::new()),
        1 => (fl!("card-new"), Style::new().fg(NEW).bold()),
        level => (
            fl!("card-level", level = level, max = MAX_LEVEL),
            Style::new(),
        ),
    };
    centred(frame, &tag_text, tag_style, tag);
    let [picture] = Layout::horizontal([Constraint::Length(PICTURE.0)])
        .flex(Flex::Center)
        .areas(picture);
    pixels::render_sprite(frame.buffer_mut(), picture, sprites, icon(card.choice));
    centred(frame, &gain_of(card.choice, card.level), Style::new(), gain);
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
            Passive::Compass => SpriteId::IconCompass,
        },
        Choice::Heal => SpriteId::IconHeal,
    }
}

fn name(choice: Choice) -> String {
    match choice {
        Choice::Weapon(weapon) => weapon_name(weapon),
        Choice::Passive(passive) => match passive {
            Passive::Vitality => fl!("passive-vitality"),
            Passive::Swiftness => fl!("passive-swiftness"),
            Passive::Magnet => fl!("passive-magnet"),
            Passive::Might => fl!("passive-might"),
            Passive::Haste => fl!("passive-haste"),
            Passive::Flashlight => fl!("passive-flashlight"),
            Passive::Compass => fl!("passive-compass"),
        },
        Choice::Heal => fl!("heal"),
    }
}

pub fn weapon_name(weapon: Weapon) -> String {
    match weapon {
        Weapon::Machete => fl!("weapon-machete"),
        Weapon::Pistol => fl!("weapon-pistol"),
        Weapon::MachineGun => fl!("weapon-machine-gun"),
        Weapon::Drones => fl!("weapon-drones"),
    }
}

/// What a choice brings, taken at `level`.
fn gain_of(choice: Choice, level: u8) -> String {
    match choice {
        Choice::Weapon(Weapon::Machete) => match level {
            1 => fl!("gain-machete-1"),
            2 => fl!("gain-machete-2"),
            3 => fl!("gain-machete-3"),
            4 => fl!("gain-machete-4"),
            _ => fl!("gain-machete-5"),
        },
        Choice::Weapon(Weapon::Pistol) => match level {
            1 => fl!("gain-pistol-1"),
            2 => fl!("gain-pistol-2"),
            3 => fl!("gain-pistol-3"),
            4 => fl!("gain-pistol-4"),
            _ => fl!("gain-pistol-5"),
        },
        Choice::Weapon(Weapon::MachineGun) => match level {
            1 => fl!("gain-machine-gun-1"),
            2 => fl!("gain-machine-gun-2"),
            3 => fl!("gain-machine-gun-3"),
            4 => fl!("gain-machine-gun-4"),
            _ => fl!("gain-machine-gun-5"),
        },
        Choice::Weapon(Weapon::Drones) => match level {
            1 => fl!("gain-drones-1"),
            2 | 4 => fl!("gain-drones-more"),
            3 => fl!("gain-drones-3"),
            _ => fl!("gain-drones-5"),
        },
        Choice::Passive(passive) => match passive {
            Passive::Vitality => fl!("gain-vitality"),
            Passive::Swiftness => fl!("gain-swiftness"),
            Passive::Magnet => fl!("gain-magnet"),
            Passive::Might => fl!("gain-might"),
            Passive::Haste => fl!("gain-haste"),
            Passive::Flashlight => fl!("gain-flashlight"),
            Passive::Compass if level <= 1 => fl!("gain-compass"),
            Passive::Compass => fl!("gain-compass-more"),
        },
        Choice::Heal => fl!("gain-heal"),
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
