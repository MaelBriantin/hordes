//! The gear over the top left of the map: the cards the player has, as icons with their
//! levels and gauges, or as a list.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::super::level_up;
use super::super::sprites::{Look, SpriteId};
use super::{Canvas, GEAR_GAP, share_of};
use crate::game::{Action, Choice, MAX_LEVEL, Rarity, World, barricade_recharge};

/// The rows of veil above the icons and under their gauges, in pixels.
const GEAR_PAD: i32 = 1;
/// The room between two groups of cards, besides the gap between two icons, in pixels.
const GROUP_GAP: i32 = 4;
const GEAR_LEVEL: Color = Color::Rgb(235, 235, 240);
const GEAR_ROUNDS: Color = Color::Rgb(235, 210, 120);
const GEAR_RELOADING: Color = Color::Rgb(165, 165, 172);
const GEAR_EMPTY: Color = Color::Rgb(70, 70, 80);
const GEAR_MAX: Color = Color::Rgb(255, 200, 60);
const GEAR_BADGE: Color = Color::Rgb(35, 35, 45);
/// The gear as a list: the names' colour, and the gauges' width, in cells.
const GEAR_NAME: Color = Color::Rgb(200, 200, 210);
/// A spent card's red, crossing it out.
pub(super) const SPENT: Color = Color::Rgb(220, 50, 50);
/// The frame round a card's icon: its rarity's colour, this share of it over this dark; how far
/// out from the icon it is, in pixels, a pixel of room between them.
const FRAME_SHARE: f32 = 0.6;
const FRAME_UNDER: (u8, u8, u8) = (24, 26, 32);
pub(super) const FRAME_ROOM: i32 = 2;
const LIST_GAUGE: i32 = 6;

impl Canvas<'_> {
    /// A frame round a card's icon `size` pixels wide at `corner`, a pixel of room between: its
    /// rarity's colour, dimmed.
    pub(super) fn frame_card(&mut self, (x, y): (i32, i32), size: i32, choice: Choice) {
        let colour = match choice.rarity() {
            Rarity::Common => level_up::COMMON,
            Rarity::Rare => level_up::RARE,
            Rarity::Legendary => level_up::LEGENDARY,
        };
        let frame = match colour {
            Color::Rgb(red, green, blue) => {
                let mix = |channel: u8, under: u8| {
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "a channel, 0 to 255"
                    )]
                    let mixed = (f32::from(channel) * FRAME_SHARE
                        + f32::from(under) * (1.0 - FRAME_SHARE))
                        .round() as u8;
                    mixed
                };
                Color::Rgb(
                    mix(red, FRAME_UNDER.0),
                    mix(green, FRAME_UNDER.1),
                    mix(blue, FRAME_UNDER.2),
                )
            }
            other => other,
        };
        let (left, top) = (x - FRAME_ROOM, y - FRAME_ROOM);
        let (right, bottom) = (x + size - 1 + FRAME_ROOM, y + size - 1 + FRAME_ROOM);
        for along in left..=right {
            self.set((along, top), frame);
            self.set((along, bottom), frame);
        }
        for along in top..=bottom {
            self.set((left, along), frame);
            self.set((right, along), frame);
        }
    }

    /// A red line across an icon `size` pixels wide at `corner`, from its bottom left to its top
    /// right, two pixels thick: a spent card.
    pub(super) fn cross_out(&mut self, (x, y): (i32, i32), size: i32) {
        for step in 0..size {
            let (px, py) = (x + step, y + size - 1 - step);
            self.set((px, py), SPENT);
            self.set((px, py - 1), SPENT);
        }
    }

    /// The player's gear as icons, over the top left of what the canvas shows, on a dark veil:
    /// each card's icon, as on the level-up cards, its gauge right under it if it has one. Where
    /// each one's level goes, written over the icon's bottom right corner once the canvas is
    /// drawn.
    pub(super) fn paint_gear(&mut self, gear: &[GearSlot]) -> Vec<GearLevel> {
        if gear.is_empty() {
            return Vec::new();
        }
        let size = self.sprites.get(SpriteId::IconMachete).width();
        // Each icon in its frame, a pixel of room between them; the frames of a group side by
        // side.
        let cell = size + 2 * FRAME_ROOM;
        let step = cell;
        let groups =
            i32::try_from(gear.iter().filter(|card| card.group_start).count()).unwrap_or(0);
        let width =
            2 * GEAR_GAP + step * i32::try_from(gear.len()).unwrap_or(0) + GROUP_GAP * groups;
        // A row, the framed icon, its gauge under the frame, a row.
        self.veil(width, GEAR_PAD + cell + 1 + GEAR_PAD);
        let (left, top) = self.origin;
        let mut levels = Vec::new();
        let mut x = left + GEAR_GAP - step;
        for card in gear {
            // A group of cards apart from the one before.
            x += step + if card.group_start { GROUP_GAP } else { 0 };
            let (x, y) = (x + FRAME_ROOM, top + GEAR_PAD + FRAME_ROOM);
            let icon = level_up::icon(card.choice);
            self.frame_card((x, y), size, card.choice);
            if card.spent {
                self.paint_greyed(icon, (x, y));
                self.cross_out((x, y), size);
            } else {
                self.paint(Look::plain(icon), (x, y));
            }
            // Its gauge under its frame, as wide as it.
            if let Some((part, whole, colour)) = card.gauge {
                let lit = share_of(part, whole, cell, colour != GEAR_RELOADING);
                for pixel in 0..cell {
                    self.set(
                        (x - FRAME_ROOM + pixel, y + size + FRAME_ROOM),
                        if pixel < lit { colour } else { GEAR_EMPTY },
                    );
                }
            }
            if let Some(level) = card.level {
                levels.push(GearLevel {
                    // In the frame's bottom right corner, inside it, a pixel off its side:
                    // over the room under the icon and its last row.
                    right: x - left + size,
                    row: y - top + size,
                    level,
                });
            }
        }
        levels
    }
}

/// A card the player has, as the gear shows it: its level, none for a legendary; its gauge, if
/// it has one: a gun's rounds or reload, the Dash card's dashes.
pub(super) struct GearSlot {
    pub(super) choice: Choice,
    pub(super) level: Option<u8>,
    pub(super) gauge: Option<(u32, u32, Color)>,
    /// Whether it is spent, crossed out in red: Revival, used up.
    pub(super) spent: bool,
    /// Whether it starts a group of cards after another: some room before it.
    pub(super) group_start: bool,
}

/// An action's gauge: the Dash shows its dashes as a gun its rounds, full when all are ready,
/// filling back as they come back; the others fill back between two uses, full once ready.
fn action_gauge(world: &World, action: Action) -> (u32, u32, Color) {
    let (player, build) = (&world.player, &world.player.build);
    let level = build.action_level(action).max(1);
    let waiting = |left: u32, base: u32| {
        let whole = build.cooldown(base).max(1);
        if left > 0 {
            (whole.saturating_sub(left), whole, GEAR_RELOADING)
        } else {
            (1, 1, GEAR_ROUNDS)
        }
    };
    match action {
        Action::Dash => {
            let recharge = build.dash_recharge();
            let most = u32::from(build.dashes());
            if u32::from(player.dashes.ready) >= most {
                (1, 1, GEAR_ROUNDS)
            } else {
                (
                    u32::from(player.dashes.ready) * recharge + player.dashes.charging,
                    most * recharge,
                    GEAR_RELOADING,
                )
            }
        }
        Action::Barricade => waiting(world.barricade_wait, barricade_recharge(level)),
        Action::Ram => waiting(world.ram_wait, crate::game::ram_recharge(level)),
        Action::C4 => waiting(world.c4_wait, crate::game::c4_recharge(level)),
        Action::Kit => waiting(world.kit_wait, crate::game::kit_recharge(level)),
        Action::FireWall => waiting(world.fire_wall_wait, crate::game::fire_wall_recharge(level)),
        Action::Smoke => waiting(world.smoke_wait, crate::game::smoke_recharge(level)),
        Action::Gatling => waiting(world.gatling_wait, crate::game::gatling_recharge(level)),
        Action::Vault => waiting(world.vault_wait, crate::game::vault_recharge(level)),
    }
}

/// The cards the player has, in the order first picked.
pub(super) fn gear_slots(world: &World) -> Vec<GearSlot> {
    let player = &world.player;
    let build = &player.build;
    let ammo = world.ammo();
    let waits = world.waits();
    let groups = build.groups();
    groups
        .iter()
        .enumerate()
        .flat_map(|(group, (_, cards))| {
            cards
                .iter()
                .enumerate()
                .map(move |(index, &choice)| (group > 0 && index == 0, choice))
        })
        .map(|(group_start, choice)| match choice {
            Choice::Weapon(weapon) => GearSlot {
                choice,
                level: Some(build.weapon_level(weapon)),
                gauge: ammo
                    .iter()
                    .find(|gun| gun.0 == weapon)
                    .map(|gun| {
                        let (_, magazine, full, reload) = *gun;
                        if magazine.reloading > 0 {
                            (
                                reload.saturating_sub(magazine.reloading),
                                reload,
                                GEAR_RELOADING,
                            )
                        } else {
                            (magazine.rounds, full, GEAR_ROUNDS)
                        }
                    })
                    // The other weapons fill between two blows, full once ready.
                    .or_else(|| {
                        waits
                            .iter()
                            .find(|wait| wait.0 == weapon)
                            .map(|&(_, left, whole)| {
                                if left > 0 {
                                    (whole.saturating_sub(left), whole, GEAR_RELOADING)
                                } else {
                                    (1, 1, GEAR_ROUNDS)
                                }
                            })
                    }),
                spent: false,
                group_start,
            },
            Choice::Passive(passive) => GearSlot {
                choice,
                level: Some(build.passive_level(passive)),
                gauge: None,
                spent: false,
                group_start,
            },
            Choice::Action(action) => GearSlot {
                choice,
                level: Some(build.action_level(action)),
                gauge: Some(action_gauge(world, action)),
                spent: false,
                group_start,
            },
            Choice::Legendary(_) | Choice::Heal => GearSlot {
                choice,
                level: None,
                gauge: None,
                spent: world.spent(choice),
                group_start,
            },
        })
        .collect()
}

/// A card's level as the gear writes it: its number, or MAX in gold at the last.
pub(super) fn level_text(level: u8) -> (String, Color) {
    if level >= MAX_LEVEL {
        ("MAX".to_string(), GEAR_MAX)
    } else {
        (level.to_string(), GEAR_LEVEL)
    }
}

/// The longest name in the gear, in cells.
pub(super) fn longest_name(gear: &[GearSlot]) -> usize {
    gear.iter()
        .map(|card| level_up::name(card.choice).chars().count())
        .max()
        .unwrap_or(0)
}

/// How much of the map the gear's list takes, in cells: a space, the longest name, a space, the
/// level, two spaces, the gauge, a space across; a line a card, and one above and below.
pub(super) fn list_size(gear: &[GearSlot]) -> (u16, u16) {
    let width = longest_name(gear) + 1 + 1 + 3 + 2 + 6 + 1;
    (
        u16::try_from(width).unwrap_or(u16::MAX),
        u16::try_from(gear.len() + 2 + gear.iter().filter(|card| card.group_start).count())
            .unwrap_or(u16::MAX),
    )
}

/// The gear as a list over the top left of the map, dimmed under it, a line a card: its name,
/// its level, its gauge if it has one.
pub(super) fn draw_gear_list(buffer: &mut Buffer, area: Rect, gear: &[GearSlot]) {
    let name_width = longest_name(gear);
    let lines = list_size(gear).1.min(area.height);
    let ground = Style::new();
    let mut row = 0;
    for card in gear {
        // A blank line between two groups of cards.
        row += if card.group_start { 2 } else { 1 };
        if row + 1 >= lines {
            break;
        }
        let y = area.y + row;
        let name = level_up::name(card.choice).to_uppercase();
        let mut x = area.x + 1;
        // Spent, its name crossed out in red.
        let name_style = if card.spent {
            ground.fg(SPENT).add_modifier(Modifier::CROSSED_OUT)
        } else {
            ground.fg(GEAR_NAME)
        };
        buffer.set_string(x, y, &name, name_style);
        x += u16::try_from(name_width).unwrap_or(0) + 1;
        if let Some(level) = card.level {
            let (text, colour) = level_text(level);
            buffer.set_string(
                x,
                y,
                format!("{text:>3}"),
                ground.fg(colour).add_modifier(Modifier::BOLD),
            );
        }
        x += 3 + 2;
        if let Some((part, whole, colour)) = card.gauge {
            let lit = share_of(part, whole, LIST_GAUGE, colour != GEAR_RELOADING);
            for cell in 0..LIST_GAUGE {
                let shade = if cell < lit { colour } else { GEAR_EMPTY };
                buffer.set_string(
                    x + u16::try_from(cell).unwrap_or(0),
                    y,
                    "━",
                    ground.fg(shade),
                );
            }
        }
    }
}

/// Where a card's level is written in the gear: the pixel column just right of its icon, and a
/// pixel row of the cell it goes in, from the canvas' top left; and the level.
pub(super) struct GearLevel {
    pub(super) right: i32,
    pub(super) row: i32,
    pub(super) level: u8,
}

/// Each card's level in the gear, in its frame's bottom right corner, on a dark badge: its
/// number, or MAX in gold once at the last.
pub(super) fn draw_gear_levels(buffer: &mut Buffer, area: Rect, levels: &[GearLevel]) {
    for gear in levels {
        let (text, colour) = level_text(gear.level);
        let width = i32::try_from(text.len()).unwrap_or(0);
        let (Ok(x), Ok(y)) = (
            u16::try_from(gear.right - width),
            u16::try_from(gear.row / 2),
        ) else {
            continue;
        };
        if x + 3 > area.width || y >= area.height {
            continue;
        }
        let style = Style::new()
            .fg(colour)
            .bg(GEAR_BADGE)
            .add_modifier(Modifier::BOLD);
        buffer.set_string(area.x + x, area.y + y, text, style);
    }
}

/// A card's level on a dark badge, ending just before `right` on row `y`.
pub(super) fn draw_level_badge(buffer: &mut Buffer, (right, y): (u16, u16), level: u8) {
    let (text, colour) = level_text(level);
    let width = u16::try_from(text.len()).unwrap_or(0);
    let style = Style::new()
        .fg(colour)
        .bg(GEAR_BADGE)
        .add_modifier(Modifier::BOLD);
    buffer.set_string(right.saturating_sub(width), y, text, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Input, Passive, Weapon};

    /// Every gauge of the gear within its whole: filled no more than full.
    fn assert_gauges_fit(world: &World) {
        for slot in gear_slots(world) {
            if let Some((filled, whole, _)) = slot.gauge {
                assert!(filled <= whole, "{:?}: {filled} / {whole}", slot.choice);
            }
        }
    }

    #[test]
    fn haste_taken_while_a_gun_reloads_leaves_its_gauge_whole() {
        let mut world = World::new(1);
        world.announcement = None;
        world.player.build.take(Choice::Weapon(Weapon::Pistol));
        world.update(Input::default());
        let reload = world.ammo()[0].3;
        world
            .magazines
            .get_mut(&Weapon::Pistol)
            .expect("a magazine once the pistol is held")
            .reloading = reload;
        for _ in 0..3 {
            world.player.build.take(Choice::Passive(Passive::Haste));
        }
        assert_gauges_fit(&world);
    }

    #[test]
    fn the_barricade_sooner_back_while_one_comes_leaves_its_gauge_whole() {
        let mut world = World::new(1);
        for _ in 0..2 {
            world.player.build.take(Choice::Action(Action::Barricade));
        }
        world.barricade_wait = barricade_recharge(2);
        world.player.build.take(Choice::Action(Action::Barricade));
        assert_gauges_fit(&world);
    }
}
