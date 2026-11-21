//! The gear over the top left of the map: the cards the player has, as icons with their
//! levels and gauges, or as a list.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::super::level_up;
use super::super::sprites::{Look, SpriteId};
use super::{Canvas, GEAR_GAP, share_of};
use crate::game::{Action, Choice, MAX_LEVEL, World};

/// The rows of veil above the icons and under their gauges, in pixels.
const GEAR_PAD: i32 = 1;
const GEAR_LEVEL: Color = Color::Rgb(235, 235, 240);
const GEAR_ROUNDS: Color = Color::Rgb(235, 210, 120);
const GEAR_RELOADING: Color = Color::Rgb(165, 165, 172);
const GEAR_EMPTY: Color = Color::Rgb(70, 70, 80);
const GEAR_MAX: Color = Color::Rgb(255, 200, 60);
const GEAR_BADGE: Color = Color::Rgb(35, 35, 45);
/// The gear as a list: the names' colour, and the gauges' width, in cells.
const GEAR_NAME: Color = Color::Rgb(200, 200, 210);
const LIST_GAUGE: i32 = 6;

impl Canvas<'_> {
    /// The player's gear as icons, over the top left of what the canvas shows, on a dark veil:
    /// each card's icon, as on the level-up cards, its gauge right under it if it has one. Where
    /// each one's level goes, written over the icon's bottom right corner once the canvas is
    /// drawn.
    pub(super) fn paint_gear(&mut self, gear: &[GearSlot]) -> Vec<GearLevel> {
        if gear.is_empty() {
            return Vec::new();
        }
        let size = self.sprites.get(SpriteId::IconMachete).width();
        let step = size + GEAR_GAP;
        let width = GEAR_GAP + step * i32::try_from(gear.len()).unwrap_or(0);
        // A row, the icon, its gauge, a row.
        self.veil(width, GEAR_PAD + size + 1 + GEAR_PAD);
        let (left, top) = self.origin;
        let mut levels = Vec::new();
        for (slot, card) in (0..).zip(gear) {
            let x = left + GEAR_GAP + step * slot;
            let y = top + GEAR_PAD;
            self.paint(Look::plain(level_up::icon(card.choice)), (x, y));
            if let Some((part, whole, colour)) = card.gauge {
                let lit = share_of(part, whole, size, colour != GEAR_RELOADING);
                for pixel in 0..size {
                    self.set(
                        (x + pixel, y + size),
                        if pixel < lit { colour } else { GEAR_EMPTY },
                    );
                }
            }
            if let Some(level) = card.level {
                levels.push(GearLevel {
                    right: x - left + size,
                    // The cell over the icon's last two rows but one: the gauge left clear.
                    row: y - top + size - 2,
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
}

/// The cards the player has, in the order first picked.
pub(super) fn gear_slots(world: &World) -> Vec<GearSlot> {
    let player = &world.player;
    let build = &player.build;
    let ammo = world.ammo();
    let waits = world.waits();
    // The Dash card shows its dashes as a gun its rounds: full when all are ready, filling back
    // as they come back.
    let recharge = build.dash_recharge();
    let most = u32::from(build.dashes());
    let dashes = if u32::from(player.dashes.ready) >= most {
        (1, 1, GEAR_ROUNDS)
    } else {
        (
            u32::from(player.dashes.ready) * recharge + player.dashes.charging,
            most * recharge,
            GEAR_RELOADING,
        )
    };
    build
        .picked()
        .iter()
        .map(|&choice| match choice {
            Choice::Weapon(weapon) => GearSlot {
                choice,
                level: Some(build.weapon_level(weapon)),
                gauge: ammo
                    .iter()
                    .find(|gun| gun.0 == weapon)
                    .map(|gun| {
                        let (_, magazine, full, reload) = *gun;
                        if magazine.reloading > 0 {
                            (reload - magazine.reloading, reload, GEAR_RELOADING)
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
                                    (whole - left, whole, GEAR_RELOADING)
                                } else {
                                    (1, 1, GEAR_ROUNDS)
                                }
                            })
                    }),
            },
            Choice::Passive(passive) => GearSlot {
                choice,
                level: Some(build.passive_level(passive)),
                gauge: None,
            },
            Choice::Action(action) => GearSlot {
                choice,
                level: Some(build.action_level(action)),
                gauge: (action == Action::Dash).then_some(dashes),
            },
            Choice::Legendary(_) | Choice::Heal => GearSlot {
                choice,
                level: None,
                gauge: None,
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
        u16::try_from(gear.len() + 2).unwrap_or(u16::MAX),
    )
}

/// The gear as a list over the top left of the map, dimmed under it, a line a card: its name,
/// its level, its gauge if it has one.
pub(super) fn draw_gear_list(buffer: &mut Buffer, area: Rect, gear: &[GearSlot]) {
    let name_width = longest_name(gear);
    let lines = list_size(gear).1.min(area.height);
    let ground = Style::new();
    for (row, card) in (1_u16..).zip(gear) {
        if row + 1 >= lines {
            break;
        }
        let y = area.y + row;
        let name = level_up::name(card.choice).to_uppercase();
        let mut x = area.x + 1;
        buffer.set_string(x, y, &name, ground.fg(GEAR_NAME));
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

/// Each card's level in the gear, over its icon's bottom right corner, on a dark badge: its
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
