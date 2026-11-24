//! The chargers about to strike, over the night: scraping the ground or charging, their eyes
//! glowing red, their dust kicked up.

use ratatui::style::Color;

use super::super::sprites;
use super::Canvas;
use crate::game::{CHARGER_SCRAPE_TICKS, Charge, Enemy, EnemyKind, Facing, World};

/// Its eyes: red as any, then glowing, two reds it blinks between as it scrapes, every so many
/// ticks; the brighter as it charges.
const EYES: Color = Color::Rgb(230, 40, 40);
const EYE_GLOW: [Color; 2] = [Color::Rgb(255, 90, 70), Color::Rgb(255, 170, 140)];
const EYE_BLINK: u32 = 3;
/// Its dust, fading to the dark ground; the ticks a puff of it lasts, scraping and charging,
/// and every how many ticks a puff is kicked up.
const DUST: Color = Color::Rgb(200, 190, 165);
const DUST_GONE: Color = Color::Rgb(60, 58, 50);
const SCRAPE_DUST_TICKS: u32 = 15;
const SCRAPE_PUFF: u32 = 4;
const CHARGE_DUST_TICKS: u32 = 14;
const CHARGE_PUFF: u32 = 2;
/// How far it goes a tick charging, in pixels: 12 tiles a second.
const CHARGE_PIXELS: f32 = 0.8;
/// How many ticks each of its feet scrapes, the dust going out on that side, up or down.
const SCRAPE_STEP: u32 = 5;

impl Canvas<'_> {
    /// The chargers scraping or charging, over the night: their eyes glowing, their dust.
    pub(super) fn paint_charger_glow(&mut self, world: &World) {
        let ticks = world.ticks();
        for enemy in world.enemies.iter().filter(|enemy| showing(enemy)) {
            let look = sprites::enemy(enemy, ticks);
            let (left, top) = self.corner_of(look, enemy.position);
            let sprite = self.sprites.get(look.sprite);
            let (width, height) = (sprite.width(), sprite.height());
            let glow = match enemy.charge {
                Charge::Scraping(_) => {
                    EYE_GLOW[usize::from(!(ticks / EYE_BLINK).is_multiple_of(2))]
                }
                _ => EYE_GLOW[1],
            };
            let eyes: Vec<_> = sprite
                .pixels(look.mirrored)
                .filter(|&(_, _, colour)| colour == EYES)
                .collect();
            for (x, y, _) in eyes {
                self.set((left + x, top + y), glow);
            }
            let feet = (left + width / 2, top + height - 1);
            match enemy.charge {
                Charge::Scraping(left_ticks) => self.paint_scrape_dust(enemy, feet, left_ticks),
                Charge::Charging {
                    left: left_ticks, ..
                } => {
                    self.paint_charge_dust(enemy, feet, left_ticks);
                }
                Charge::Ready | Charge::Resting(_) => {}
            }
        }
    }

    /// Puffs kicked back from its back foot, sideways, or out to the side of the foot scraping,
    /// up or down: going up, spreading, fading.
    fn paint_scrape_dust(&mut self, enemy: &Enemy, (x, y): (i32, i32), left: u32) {
        let scraped = CHARGER_SCRAPE_TICKS.saturating_sub(left);
        for age in 0..SCRAPE_DUST_TICKS.min(scraped + 1) {
            let born = scraped - age;
            if !born.is_multiple_of(SCRAPE_PUFF) {
                continue;
            }
            let colour = fade(age, SCRAPE_DUST_TICKS);
            let age_f = f32::from(u16::try_from(age).unwrap_or(0));
            for grain in 0..3_u32 {
                let luck = luck(enemy.id, born, grain);
                let spread = f32::from(u16::try_from(grain).unwrap_or(0)) - 1.0;
                let rise =
                    age_f * (0.1 + f32::from(u16::try_from((luck >> 3) % 4).unwrap_or(0)) * 0.06);
                let out = age_f * (0.25 + f32::from(u16::try_from(luck % 5).unwrap_or(0)) * 0.06);
                let (dx, dy) = match enemy.facing {
                    Facing::Left => (3.0 + out, -rise + spread * 0.6),
                    Facing::Right => (-3.0 - out, -rise + spread * 0.6),
                    Facing::Up | Facing::Down => {
                        let side = if (born / SCRAPE_STEP).is_multiple_of(2) {
                            -1.0
                        } else {
                            1.0
                        };
                        (side * (3.0 + out * 0.6) + spread * 0.5, -1.0 - rise * 2.0)
                    }
                };
                self.set((x + round(dx), y + round(dy)), colour);
            }
        }
    }

    /// Puffs left where its feet went, charging: rising a little, fading.
    fn paint_charge_dust(&mut self, enemy: &Enemy, (x, y): (i32, i32), left: u32) {
        let Charge::Charging { way, .. } = enemy.charge else {
            return;
        };
        let charged = crate::game::CHARGER_CHARGE_TICKS.saturating_sub(left);
        for age in 0..CHARGE_DUST_TICKS.min(charged + 1) {
            let born = charged - age;
            if !born.is_multiple_of(CHARGE_PUFF) {
                continue;
            }
            let luck = luck(enemy.id, born, 5);
            let age_f = f32::from(u16::try_from(age).unwrap_or(0));
            let jitter = f32::from(u16::try_from(luck % 3).unwrap_or(0)) - 1.0;
            let back = age_f * CHARGE_PIXELS;
            let (dx, dy) = (
                -way.x * back + way.y * jitter,
                -way.y * back
                    - way.x * jitter
                    - age_f * 0.15
                    - f32::from(u16::try_from((luck >> 4) % 2).unwrap_or(0)),
            );
            self.set((x + round(dx), y + round(dy)), fade(age, CHARGE_DUST_TICKS));
        }
    }
}

/// Whether a charger shows in the dark: scraping or charging, not stunned.
fn showing(enemy: &Enemy) -> bool {
    enemy.kind == EnemyKind::Charger && enemy.hp > 0 && enemy.stunned == 0 && enemy.charge.showing()
}

/// A puff of dust `age` ticks old, of `lasts`.
fn fade(age: u32, lasts: u32) -> Color {
    let share =
        f32::from(u16::try_from(age).unwrap_or(0)) / f32::from(u16::try_from(lasts).unwrap_or(1));
    super::super::blend(DUST, DUST_GONE, share)
}

/// A number of its own for a grain of dust.
fn luck(id: u32, born: u32, grain: u32) -> u32 {
    (id.wrapping_mul(2_654_435_761) ^ born.wrapping_mul(40_503) ^ grain.wrapping_mul(97)) >> 5
}

#[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
fn round(value: f32) -> i32 {
    value.round() as i32
}
