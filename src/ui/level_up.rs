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
use crate::game::{
    Action, Choice, HEAL, Legendary, MAX_LEVEL, MAX_PASSIVES, MAX_WEAPONS, Passive, Rarity, Skin,
    TICKS_PER_SECOND, Weapon, World, kit_at,
};

const CARD_WIDTH: u16 = 24;
const CARD_HEIGHT: u16 = 14;
/// Columns between two cards.
const GAP: u16 = 2;
/// The picture's size, in cells: 8 × 8 pixels.
const PICTURE: (u16, u16) = (8, 4);
const NEW: Color = Color::Rgb(80, 200, 90);
/// A rare card's blue and a legendary card's gold: their frame, and the word on it.
pub const RARE: Color = Color::Rgb(90, 150, 255);
pub const LEGENDARY: Color = Color::Rgb(255, 200, 60);
const BACK: Color = Color::Rgb(70, 80, 120);
/// How much of its colour a card's frame keeps when not selected: the selected one stands out,
/// bright, in the same colour.
const UNSELECTED: f32 = 0.55;
/// A common card's frame, selected.
pub const COMMON: Color = Color::Rgb(235, 235, 240);
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
    // Under the cards, the room left for weapons and passives; and the rerolls left, if any.
    let (weapons, passives) = world.player.build.counts();
    let room = fl!(
        "level-up-room",
        weapons = weapons,
        most_weapons = MAX_WEAPONS,
        passives = passives,
        most_passives = MAX_PASSIVES
    );
    let mut block = Block::bordered()
        .padding(Padding::new(GAP, GAP, 1, 1))
        .title(
            Line::from(format!(
                " {} ",
                fl!("level-up-title", level = world.player.level).to_uppercase()
            ))
            .bold()
            .centered(),
        )
        .title_bottom(Line::from(format!(" {room} ")).left_aligned());
    if world.rerolls > 0 {
        let reroll = fl!("level-up-reroll", left = world.rerolls);
        block = block.title_bottom(Line::from(format!(" {reroll} ")).right_aligned());
    }
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
            hero: world.player.build.hero(),
            selected: index == selected,
        };
        let started = shown.saturating_sub(DEAL * u32::try_from(index).unwrap_or(0));
        render_card(frame, *slot, sprites, &card, turned(started));
    }
}

/// The hero's own action, just come: a box over the run, its word on top, a line telling it is
/// the hero's own; its icon big on the left, its name, what it is and what it does on the right;
/// how to go on at its foot.
pub fn render_unlocked(frame: &mut Frame, map: Rect, world: &World, sprites: &Sprites) {
    let Some(action) = world.unlocked else {
        return;
    };
    let hero = world.player.build.hero();
    let choice = Choice::Action(action);
    let width = UNLOCKED_WIDTH.min(map.width);
    let text_width = width.saturating_sub(2 * (1 + GAP) + UNLOCKED_ICON.0 + GAP);
    let description = super::text::wrap(&action_description(action), usize::from(text_width));
    let effect = super::text::wrap(&gain_of(choice, 1, hero), usize::from(text_width));
    let words = 2 + description.len() + 1 + effect.len();
    let height = (u16::try_from(words).unwrap_or(0).max(UNLOCKED_ICON.1) + 4).min(map.height);
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(map);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Blank, area);
    let block = Block::bordered()
        .padding(Padding::new(GAP, GAP, 1, 1))
        .title(
            Line::from(format!(" {} ", fl!("unlocked-title").to_uppercase()))
                .bold()
                .centered(),
        )
        .title_bottom(Line::from(format!(" {} ", fl!("unlocked-prompt"))).centered());
    let inside = block.inner(area);
    frame.render_widget(block, area);
    let [icon_area, _, text_area] = Layout::horizontal([
        Constraint::Length(UNLOCKED_ICON.0),
        Constraint::Length(GAP),
        Constraint::Fill(1),
    ])
    .areas(inside);
    pixels::render_sprite_scaled(
        frame.buffer_mut(),
        Rect {
            height: UNLOCKED_ICON.1,
            ..icon_area
        },
        sprites,
        icon(choice),
        2,
    );
    let mut lines = vec![
        Line::styled(name(choice).to_uppercase(), Style::new().bold()),
        Line::default(),
    ];
    lines.extend(
        description
            .into_iter()
            .map(|line| Line::styled(line, Style::new().fg(Color::Gray))),
    );
    lines.push(Line::default());
    lines.extend(effect.into_iter().map(Line::from));
    frame.render_widget(Paragraph::new(lines), text_area);
}

/// What an action is, in a few words: the hero's own, told as it comes.
fn action_description(action: Action) -> String {
    match action {
        Action::Dash => fl!("action-dash-description"),
        Action::Barricade => fl!("action-barricade-description"),
        Action::Ram => fl!("action-ram-description"),
        Action::C4 => fl!("action-c4-description"),
        Action::Kit => fl!("action-kit-description"),
    }
}

struct Card {
    index: usize,
    choice: Choice,
    level: u8,
    selected: bool,
    /// The hero: their own HP and regeneration are what Vitality's and Regeneration's cards count
    /// from.
    hero: Skin,
}

/// How wide the box telling an action just come is; the room its icon takes, twice as big.
const UNLOCKED_WIDTH: u16 = 64;
const UNLOCKED_ICON: (u16, u16) = (16, 8);

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
    // Its number on its top border, and whether it is a weapon or a passive, as plainly.
    let kind = match card.choice {
        Choice::Weapon(_) => Some(fl!("card-weapon")),
        // A legendary is a passive to the player: one taking no room.
        Choice::Passive(_) | Choice::Legendary(_) => Some(fl!("card-passive")),
        Choice::Action(_) => Some(fl!("card-action")),
        Choice::Heal => None,
    };
    let title = match kind {
        Some(word) => Line::from(format!(" {} · {word} ", card.index + 1)),
        None => Line::from(format!(" {} ", card.index + 1)),
    };
    let block = Block::bordered()
        .border_style(border)
        .padding(Padding::new(2, 2, 1, 0))
        .title(title.centered());
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
    centred(
        frame,
        &gain_of(card.choice, card.level, card.hero),
        Style::new(),
        gain,
    );
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
            Weapon::Flamethrower => SpriteId::IconFlamethrower,
            Weapon::Hammer => SpriteId::IconHammer,
            Weapon::Baton => SpriteId::IconBaton,
            Weapon::Knife => SpriteId::IconKnife,
            Weapon::Katana => SpriteId::IconKatana,
            Weapon::Shuriken => SpriteId::IconShuriken,
            Weapon::Defibrillator => SpriteId::IconDefibrillator,
            Weapon::Chainsaw => SpriteId::IconChainsaw,
            Weapon::Grenade => SpriteId::IconGrenade,
            Weapon::Sniper => SpriteId::IconSniper,
            Weapon::Napalm => SpriteId::IconNapalm,
            Weapon::Fists => SpriteId::IconFists,
            Weapon::Truncheon => SpriteId::IconTruncheon,
        },
        Choice::Passive(passive) => match passive {
            Passive::Vitality => SpriteId::IconVitality,
            Passive::Swiftness => SpriteId::IconSwiftness,
            Passive::Magnet => SpriteId::IconMagnet,
            Passive::Might => SpriteId::IconMight,
            Passive::Haste => SpriteId::IconHaste,
            Passive::Flashlight => SpriteId::IconFlashlight,
            Passive::Wisdom => SpriteId::IconWisdom,
            Passive::Loot => SpriteId::IconLoot,
            Passive::Regeneration => SpriteId::IconRegeneration,
            Passive::Glare => SpriteId::IconGlare,
            Passive::Drain => SpriteId::IconDrain,
            Passive::HordeHunter => SpriteId::IconHordeHunter,
        },
        Choice::Action(Action::Dash) => SpriteId::IconDash,
        Choice::Action(Action::Barricade) => SpriteId::IconBarricade,
        Choice::Action(Action::Ram) => SpriteId::IconRam,
        Choice::Action(Action::C4) => SpriteId::IconC4,
        Choice::Action(Action::Kit) => SpriteId::IconKit,
        Choice::Legendary(Legendary::Illumination) => SpriteId::IconIllumination,
        Choice::Heal => SpriteId::IconHeal,
    }
}

pub fn name(choice: Choice) -> String {
    match choice {
        Choice::Weapon(weapon) => weapon_name(weapon),
        Choice::Passive(passive) => match passive {
            Passive::Vitality => fl!("passive-vitality"),
            Passive::Swiftness => fl!("passive-swiftness"),
            Passive::Magnet => fl!("passive-magnet"),
            Passive::Might => fl!("passive-might"),
            Passive::Haste => fl!("passive-haste"),
            Passive::Flashlight => fl!("passive-flashlight"),
            Passive::Wisdom => fl!("passive-wisdom"),
            Passive::Loot => fl!("passive-loot"),
            Passive::Regeneration => fl!("passive-regeneration"),
            Passive::Glare => fl!("passive-glare"),
            Passive::Drain => fl!("passive-drain"),
            Passive::HordeHunter => fl!("passive-horde-hunter"),
        },
        Choice::Action(Action::Dash) => fl!("passive-dash"),
        Choice::Action(Action::Barricade) => fl!("action-barricade"),
        Choice::Action(Action::Ram) => fl!("action-ram"),
        Choice::Action(Action::C4) => fl!("action-c4"),
        Choice::Action(Action::Kit) => fl!("action-kit"),
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
        Weapon::Flamethrower => fl!("weapon-flamethrower"),
        Weapon::Hammer => fl!("weapon-hammer"),
        Weapon::Baton => fl!("weapon-baton"),
        Weapon::Knife => fl!("weapon-knife"),
        Weapon::Katana => fl!("weapon-katana"),
        Weapon::Shuriken => fl!("weapon-shuriken"),
        Weapon::Defibrillator => fl!("weapon-defibrillator"),
        Weapon::Chainsaw => fl!("weapon-chainsaw"),
        Weapon::Grenade => fl!("weapon-grenade"),
        Weapon::Sniper => fl!("weapon-sniper"),
        Weapon::Napalm => fl!("weapon-napalm"),
        Weapon::Fists => fl!("weapon-fists"),
        Weapon::Truncheon => fl!("weapon-truncheon"),
    }
}

/// What a choice brings, taken at `level`.
pub(super) fn gain_of(choice: Choice, level: u8, hero: Skin) -> String {
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
            3 => fl!("gain-drones-3"),
            4 => fl!("gain-drones-4"),
            _ => fl!("gain-drones-more"),
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
        // The weapons come later: the flamethrower, the heroes' own, the shurikens.
        Choice::Weapon(weapon) => later_weapon_gain(weapon, level),
        Choice::Passive(passive) => passive_gain(passive, level, hero),
        Choice::Action(action) => action_gain(action, level),
        Choice::Legendary(Legendary::Illumination) => fl!("gain-illumination"),
        Choice::Heal => {
            let hp: u32 = HEAL * hero.perk().heal_percent() / 100;
            fl!("gain-heal", hp = hp)
        }
    }
}

/// What a passive brings to `hero`, taken at `level`: its share, and its total from the hero's own.
fn passive_gain(passive: Passive, level: u8, hero: Skin) -> String {
    let level = u32::from(level);
    let total: u32 = match passive {
        // From the hero's own HP: the demolisher's 140, the agent's 80.
        Passive::Vitality => hero.hp() + 20 * level,
        Passive::Swiftness | Passive::Haste => 8 * level,
        Passive::Magnet => 2 + level,
        // The hero's own counted: the veteran's Commando.
        Passive::Might => (100 + 10 * level) * hero.perk().damage_percent() / 100 - 100,
        Passive::Wisdom | Passive::Loot | Passive::Glare | Passive::HordeHunter => 10 * level,
        // In tenths of HP a second.
        Passive::Regeneration => 4 * level,
        Passive::Drain => 5 + 5 * level,
        Passive::Flashlight => 9 + 2 * level,
    };
    match passive {
        Passive::Vitality => fl!("gain-vitality", total = total),
        // Against the survivor's pace, the hero's own counted: the demolisher slower.
        Passive::Swiftness => {
            let total = i32::from(hero.speed_percent()) + i32::try_from(total).unwrap_or(0);
            let total = if total < 0 {
                format!("−{}", total.unsigned_abs())
            } else {
                format!("+{total}")
            };
            fl!("gain-swiftness", total = total)
        }
        Passive::Magnet => fl!("gain-magnet", total = total),
        Passive::Might => fl!("gain-might", total = total),
        Passive::Haste => fl!("gain-haste", total = total),
        Passive::Flashlight => fl!("gain-flashlight", total = total),
        Passive::Wisdom => fl!("gain-wisdom", total = total),
        Passive::Loot => fl!("gain-loot", total = total),
        Passive::Regeneration => {
            let (whole, tenth) = (total / 10, total % 10);
            fl!("gain-regeneration", whole = whole, tenth = tenth)
        }
        Passive::Glare => fl!("gain-glare", total = total),
        Passive::Drain => fl!("gain-drain", total = total),
        Passive::HordeHunter => match level {
            1 => fl!("gain-horde-hunter-1"),
            2 => fl!("gain-horde-hunter-2", total = total),
            3 => fl!("gain-horde-hunter-3", total = total),
            4 => fl!("gain-horde-hunter-4", total = total),
            _ => fl!("gain-horde-hunter-5", total = total),
        },
    }
}

/// What a weapon come later brings, taken at `level`: the flamethrower, the heroes' own, the
/// shurikens.
fn later_weapon_gain(weapon: Weapon, level: u8) -> String {
    match weapon {
        Weapon::Flamethrower => match level {
            1 => fl!("gain-flamethrower-1"),
            2 => fl!("gain-flamethrower-2"),
            3 => fl!("gain-flamethrower-3"),
            4 => fl!("gain-flamethrower-4"),
            _ => fl!("gain-flamethrower-5"),
        },
        Weapon::Hammer => hammer_gain(level),
        Weapon::Baton => baton_gain(level),
        Weapon::Knife => knife_gain(level),
        Weapon::Katana => katana_gain(level),
        Weapon::Defibrillator => match level {
            1 => fl!("gain-defibrillator-1"),
            2 => fl!("gain-defibrillator-2"),
            3 => fl!("gain-defibrillator-3"),
            4 => fl!("gain-defibrillator-4"),
            _ => fl!("gain-defibrillator-5"),
        },
        Weapon::Chainsaw => match level {
            1 => fl!("gain-chainsaw-1"),
            2 => fl!("gain-chainsaw-2"),
            3 => fl!("gain-chainsaw-3"),
            4 => fl!("gain-chainsaw-4"),
            _ => fl!("gain-chainsaw-5"),
        },
        Weapon::Sniper => match level {
            1 => fl!("gain-sniper-1"),
            2 => fl!("gain-sniper-2"),
            3 => fl!("gain-sniper-3"),
            4 => fl!("gain-sniper-4"),
            _ => fl!("gain-sniper-5"),
        },
        Weapon::Grenade => match level {
            1 => fl!("gain-grenade-1"),
            2 => fl!("gain-grenade-2"),
            3 => fl!("gain-grenade-3"),
            4 => fl!("gain-grenade-4"),
            _ => fl!("gain-grenade-5"),
        },
        Weapon::Napalm => match level {
            1 => fl!("gain-napalm-1"),
            2 => fl!("gain-napalm-2"),
            3 => fl!("gain-napalm-3"),
            4 => fl!("gain-napalm-4"),
            _ => fl!("gain-napalm-5"),
        },
        Weapon::Fists => match level {
            1 => fl!("gain-fists-1"),
            2 => fl!("gain-fists-2"),
            3 => fl!("gain-fists-3"),
            4 => fl!("gain-fists-4"),
            _ => fl!("gain-fists-5"),
        },
        Weapon::Truncheon => match level {
            1 => fl!("gain-truncheon-1"),
            2 => fl!("gain-truncheon-2"),
            3 => fl!("gain-truncheon-3"),
            4 => fl!("gain-truncheon-4"),
            _ => fl!("gain-truncheon-5"),
        },
        _ => shuriken_gain(level),
    }
}

/// What the hammer brings, taken at `level`.
fn hammer_gain(level: u8) -> String {
    match level {
        1 => fl!("gain-hammer-1"),
        2 => fl!("gain-hammer-2"),
        3 => fl!("gain-hammer-3"),
        4 => fl!("gain-hammer-4"),
        _ => fl!("gain-hammer-5"),
    }
}

/// What the stun baton brings, taken at `level`.
fn baton_gain(level: u8) -> String {
    match level {
        1 => fl!("gain-baton-1"),
        2 => fl!("gain-baton-2"),
        3 => fl!("gain-baton-3"),
        4 => fl!("gain-baton-4"),
        _ => fl!("gain-baton-5"),
    }
}

/// What the combat knife brings, taken at `level`.
fn knife_gain(level: u8) -> String {
    match level {
        1 => fl!("gain-knife-1"),
        2 => fl!("gain-knife-2"),
        3 => fl!("gain-knife-3"),
        4 => fl!("gain-knife-4"),
        _ => fl!("gain-knife-5"),
    }
}

/// What the katana brings, taken at `level`.
fn katana_gain(level: u8) -> String {
    match level {
        1 => fl!("gain-katana-1"),
        2 => fl!("gain-katana-2"),
        3 => fl!("gain-katana-3"),
        4 => fl!("gain-katana-4"),
        _ => fl!("gain-katana-5"),
    }
}

/// What the shurikens bring, taken at `level`.
fn shuriken_gain(level: u8) -> String {
    match level {
        1 => fl!("gain-shuriken-1"),
        2 => fl!("gain-shuriken-2"),
        3 => fl!("gain-shuriken-3"),
        4 => fl!("gain-shuriken-4"),
        _ => fl!("gain-shuriken-5"),
    }
}

/// What an action card brings, taken at `level`.
fn action_gain(action: Action, level: u8) -> String {
    match action {
        // The dash: level 3 brings the second; the others, sooner back, in tenths of a second.
        Action::Dash => {
            let tenths: u32 = [80, 60, 60, 45, 30][usize::from(level.clamp(1, 5) - 1)];
            match level {
                0 | 1 => fl!("gain-dash"),
                3 => fl!("gain-dash-second"),
                _ => {
                    let (whole, tenth) = (tenths / 10, tenths % 10);
                    fl!("gain-dash-sooner", whole = whole, tenth = tenth)
                }
            }
        }
        // The barricade: more HP at levels 2 and 4, a third and sooner at 3, metal at 5.
        Action::Barricade => match level {
            0 | 1 => fl!("gain-barricade"),
            2 => fl!("gain-barricade-hp", from = 80, to = 120),
            3 => fl!("gain-barricade-third"),
            4 => fl!("gain-barricade-hp", from = 120, to = 180),
            _ => fl!("gain-barricade-metal"),
        },
        // The C4: wider at 2, harder at 3, sooner back at 4, two charges at 5.
        Action::C4 => match level {
            0 | 1 => fl!("gain-c4-1"),
            2 => fl!("gain-c4-2"),
            3 => fl!("gain-c4-3"),
            4 => fl!("gain-c4-4"),
            _ => fl!("gain-c4-5"),
        },
        // The first-aid kit: more HP at 2 and 4, sooner back at 3 and 5.
        Action::Kit => {
            let (now, before) = (kit_at(level), kit_at(level.saturating_sub(1)));
            let second = u32::from(TICKS_PER_SECOND);
            let (wait, wait_before) = (now.recharge / second, before.recharge / second);
            match level {
                0 | 1 => fl!("gain-kit-1", hp = now.heal, seconds = wait),
                2 | 4 => fl!("gain-kit-hp", from = before.heal, to = now.heal),
                _ => fl!("gain-kit-recharge", from = wait_before, to = wait),
            }
        }
        // The ram: sooner back at 2, harder at 3, further at 4, a longer stun at 5.
        Action::Ram => match level {
            0 | 1 => fl!("gain-ram-1"),
            2 => fl!("gain-ram-2"),
            3 => fl!("gain-ram-3"),
            4 => fl!("gain-ram-4"),
            _ => fl!("gain-ram-5"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Action;

    #[test]
    fn the_vitality_card_counts_from_the_hero_s_own_hp() {
        let vitality = Choice::Passive(Passive::Vitality);
        assert!(
            gain_of(vitality, 1, Skin::Demolisher).contains("160"),
            "140 HP and 20 more: 160"
        );
        assert!(gain_of(vitality, 1, Skin::Survivor).contains("120"));
    }

    #[test]
    fn the_heal_might_and_swiftness_cards_count_the_hero_s_own_perks_and_pace() {
        assert!(
            gain_of(Choice::Heal, 1, Skin::Medic).contains("30"),
            "half again"
        );
        assert!(gain_of(Choice::Heal, 1, Skin::Survivor).contains("20"));
        let might = gain_of(Choice::Passive(Passive::Might), 1, Skin::Classic);
        assert!(might.contains("21"), "Commando on top: {might}");
        let swiftness = gain_of(Choice::Passive(Passive::Swiftness), 1, Skin::Demolisher);
        assert!(swiftness.contains("−7"), "still slower: {swiftness}");
    }

    #[test]
    fn the_regeneration_card_counts_no_own_regeneration_and_the_kit_tells_the_medic_s_hp() {
        let regeneration = Choice::Passive(Passive::Regeneration);
        let medic = gain_of(regeneration, 1, Skin::Medic);
        assert!(medic.contains("0,4") || medic.contains("0.4"), "{medic}");
        let kit = Choice::Action(Action::Kit);
        assert!(gain_of(kit, 1, Skin::Medic).contains("15"));
        assert!(gain_of(kit, 2, Skin::Medic).contains("20"));
        assert!(!gain_of(kit, 1, Skin::Medic).contains("Espace"));
    }

    #[test]
    fn every_gain_fits_its_card() {
        let choices = Weapon::ALL
            .into_iter()
            .map(Choice::Weapon)
            .chain(Passive::ALL.into_iter().map(Choice::Passive))
            .chain(Action::ALL.into_iter().map(Choice::Action))
            .chain(Legendary::ALL.into_iter().map(Choice::Legendary))
            .chain([Choice::Heal]);
        // Inside its frame and its room on each side, three rows under the picture.
        let width = usize::from(CARD_WIDTH - 2 - 4);
        for choice in choices {
            for level in 1..=MAX_LEVEL {
                let gain = gain_of(choice, level, Skin::Survivor);
                let rows = super::super::text::wrap(&gain, width).len();
                assert!(rows <= 3, "{choice:?} at {level}: {gain}");
            }
        }
    }

    #[test]
    fn a_card_turns_over_in_its_time() {
        assert!(turned(Duration::ZERO) < f32::EPSILON);
        assert!((turned(FLIP / 2) - 0.5).abs() < 1e-3);
        assert!((turned(FLIP * 3) - 1.0).abs() < f32::EPSILON);
    }
}
