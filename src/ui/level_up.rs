//! The level-up: the choices on offer as cards over the map, dealt face down then turned over
//! one after the other, each with its picture; the selected one framed in red.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph};

use super::night;
use super::pixels;
use super::sprites::{SpriteId, Sprites};
use super::text::centred;
use super::{Blank, DARK_GREY};
use crate::game::{Choice, Legendary, MAX_LEVEL, Passive, Rarity, Weapon, World};

const CARD_WIDTH: u16 = 24;
const CARD_HEIGHT: u16 = 14;
/// Columns between two cards.
const GAP: u16 = 2;
/// The picture's size, in cells: 8 × 8 pixels.
const PICTURE: (u16, u16) = (8, 4);
const NEW: Color = Color::Rgb(80, 200, 90);
/// A rare card's blue and a legendary card's gold: their frame, and the word on it.
const RARE: Color = Color::Rgb(90, 150, 255);
const LEGENDARY: Color = Color::Rgb(255, 200, 60);
const BACK: Color = Color::Rgb(70, 80, 120);
/// How much of its colour a card's frame keeps when not selected: the selected one stands out,
/// bright, in the same colour.
const UNSELECTED: f32 = 0.55;
/// A common card's frame, selected.
const COMMON: Color = Color::Rgb(235, 235, 240);
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
    frame.render_widget(Blank, area);
    let block = Block::bordered()
        .padding(Padding::new(GAP, GAP, 1, 1))
        .title(
            Line::from(format!(
                " {} ",
                fl!("level-up-title", level = world.player.level).to_uppercase()
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
        let color = if face_up { DARK_GREY } else { BACK };
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

    // A rare or legendary card says so on its bottom border, in its colour, its frame too.
    // The frame in the card's rarity's colour: bright and bold when selected, dimmer when not.
    let (rarity, rarity_colour) = match card.choice.rarity() {
        Rarity::Common => (None, COMMON),
        Rarity::Rare => (Some(fl!("card-rare")), RARE),
        Rarity::Legendary => (Some(fl!("card-legendary")), LEGENDARY),
    };
    let rarity_colour = if card.selected {
        rarity_colour
    } else {
        dimmed(rarity_colour)
    };
    let border = if card.selected {
        Style::new().fg(rarity_colour).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(rarity_colour)
    };
    let block = Block::bordered()
        .border_style(border)
        .padding(Padding::new(2, 2, 1, 0))
        .title(Line::from(format!(" {} ", card.index + 1)).centered());
    let block = match rarity {
        Some(rarity) => block.title_bottom(
            Line::styled(
                format!(" {rarity} "),
                Style::new().fg(rarity_colour).add_modifier(Modifier::BOLD),
            )
            .centered(),
        ),
        None => block,
    };
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
    centred(
        frame,
        &name(card.choice).to_uppercase(),
        Style::new().bold(),
        title,
    );
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

/// A colour, dimmed: the frame of a card not selected.
fn dimmed(color: Color) -> Color {
    match color {
        Color::Rgb(red, green, blue) => Color::Rgb(
            night::scale_channel(red, UNSELECTED),
            night::scale_channel(green, UNSELECTED),
            night::scale_channel(blue, UNSELECTED),
        ),
        other => other,
    }
}

/// The picture of a weapon, a passive or the heal, on its card.
pub fn icon(choice: Choice) -> SpriteId {
    match choice {
        Choice::Weapon(weapon) => match weapon {
            Weapon::Crowbar => SpriteId::IconCrowbar,
            Weapon::Machete => SpriteId::IconMachete,
            Weapon::Pistol => SpriteId::IconPistol,
            Weapon::MachineGun => SpriteId::IconMachineGun,
            Weapon::Drones => SpriteId::IconDrones,
            Weapon::Shotgun => SpriteId::IconShotgun,
            Weapon::Molotov => SpriteId::IconMolotov,
        },
        Choice::Passive(passive) => match passive {
            Passive::Vitality => SpriteId::IconVitality,
            Passive::Swiftness => SpriteId::IconSwiftness,
            Passive::Magnet => SpriteId::IconMagnet,
            Passive::Might => SpriteId::IconMight,
            Passive::Haste => SpriteId::IconHaste,
            Passive::Flashlight => SpriteId::IconFlashlight,
            Passive::Compass => SpriteId::IconCompass,
            Passive::Wisdom => SpriteId::IconWisdom,
            Passive::Loot => SpriteId::IconLoot,
            Passive::Regeneration => SpriteId::IconRegeneration,
            Passive::Glare => SpriteId::IconGlare,
        },
        Choice::Legendary(Legendary::Illumination) => SpriteId::IconIllumination,
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
            Passive::Wisdom => fl!("passive-wisdom"),
            Passive::Loot => fl!("passive-loot"),
            Passive::Regeneration => fl!("passive-regeneration"),
            Passive::Glare => fl!("passive-glare"),
        },
        Choice::Legendary(Legendary::Illumination) => fl!("legendary-illumination"),
        Choice::Heal => fl!("heal"),
    }
}

pub fn weapon_name(weapon: Weapon) -> String {
    match weapon {
        Weapon::Crowbar => fl!("weapon-crowbar"),
        Weapon::Machete => fl!("weapon-machete"),
        Weapon::Pistol => fl!("weapon-pistol"),
        Weapon::MachineGun => fl!("weapon-machine-gun"),
        Weapon::Drones => fl!("weapon-drones"),
        Weapon::Shotgun => fl!("weapon-shotgun"),
        Weapon::Molotov => fl!("weapon-molotov"),
    }
}

/// What a choice brings, taken at `level`.
fn gain_of(choice: Choice, level: u8) -> String {
    match choice {
        Choice::Weapon(Weapon::Crowbar) => match level {
            1 => fl!("gain-crowbar-1"),
            2 => fl!("gain-crowbar-2"),
            3 => fl!("gain-crowbar-3"),
            4 => fl!("gain-crowbar-4"),
            _ => fl!("gain-crowbar-5"),
        },
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
        Choice::Weapon(Weapon::Shotgun) => match level {
            1 => fl!("gain-shotgun-1"),
            2 => fl!("gain-shotgun-2"),
            3 => fl!("gain-shotgun-3"),
            4 => fl!("gain-shotgun-4"),
            _ => fl!("gain-shotgun-5"),
        },
        Choice::Weapon(Weapon::Molotov) => match level {
            1 => fl!("gain-molotov-1"),
            2 => fl!("gain-molotov-2"),
            3 => fl!("gain-molotov-3"),
            4 => fl!("gain-molotov-4"),
            _ => fl!("gain-molotov-5"),
        },
        // A passive's gain, and the total it brings the player to at that level.
        Choice::Passive(passive) => {
            let level = u32::from(level);
            let total: u32 = match passive {
                Passive::Vitality => 100 + 20 * level,
                Passive::Swiftness | Passive::Haste => 8 * level,
                Passive::Magnet => 2 + level,
                Passive::Might | Passive::Wisdom | Passive::Loot | Passive::Glare => 10 * level,
                // In tenths of HP a second.
                Passive::Regeneration => 4 * level,
                Passive::Flashlight => 9 + 2 * level,
                Passive::Compass => 100 + 50 * level,
            };
            match passive {
                Passive::Vitality => fl!("gain-vitality", total = total),
                Passive::Swiftness => fl!("gain-swiftness", total = total),
                Passive::Magnet => fl!("gain-magnet", total = total),
                Passive::Might => fl!("gain-might", total = total),
                Passive::Haste => fl!("gain-haste", total = total),
                Passive::Flashlight => fl!("gain-flashlight", total = total),
                Passive::Compass if level <= 1 => fl!("gain-compass"),
                Passive::Compass => fl!("gain-compass-more", total = total),
                Passive::Wisdom => fl!("gain-wisdom", total = total),
                Passive::Loot => fl!("gain-loot", total = total),
                Passive::Regeneration => {
                    let (whole, tenth) = (total / 10, total % 10);
                    fl!("gain-regeneration", whole = whole, tenth = tenth)
                }
                Passive::Glare => fl!("gain-glare", total = total),
            }
        }
        Choice::Legendary(Legendary::Illumination) => fl!("gain-illumination"),
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
