//! The Eye-gouger: stars turning over the head of every enemy it has dazed; an enemy it has
//! felled bursting, a spray of blood and chunks thrown out the way the shot went.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::{FLASH_BURST_TICKS, FlashBurst, TICKS_PER_SECOND, Vec2, World};

/// The ball in flight, black, a grey trail behind it.
pub(super) const BALL: Color = Color::Rgb(35, 35, 40);
pub(super) const BALL_TRAIL: Color = Color::Rgb(120, 120, 130);
/// The stars over a dazed enemy; how fast they turn, in radians a tick.
const STAR: Color = Color::Rgb(255, 225, 100);
const STAR_TURN: f32 = 0.27;
/// A burst: blood, darker blood, chunks; the white flash it starts with, in ticks.
const BLOOD: Color = Color::Rgb(190, 30, 30);
const DARK_BLOOD: Color = Color::Rgb(120, 15, 20);
const CHUNK: Color = Color::Rgb(110, 170, 90);
const FLASH: Color = Color::Rgb(255, 250, 225);
const FLASH_TICKS: u32 = 2;
/// How many drops a burst throws, how long they fly, in ticks, how far they fall after.
const DROPS: u32 = 18;
const FLY_TICKS: u32 = 10;

impl Canvas<'_> {
    /// The flashball's marks: stars over the dazed, the felled bursting.
    pub(super) fn paint_flashball(&mut self, world: &World) {
        let ticks = world.ticks();
        for enemy in world.enemies.iter().filter(|enemy| enemy.dazed > 0) {
            let (x, y) = pixel(enemy.position);
            let turn = f32::from(u16::try_from(ticks % 10_000).unwrap_or(0)) * STAR_TURN;
            for star in 0..3_u8 {
                let angle = turn + f32::from(star) * 2.1;
                #[allow(clippy::cast_possible_truncation, reason = "a pixel or two")]
                let (dx, dy) = (
                    (2.0 * angle.cos()).round() as i32,
                    (0.6 * angle.sin()).round() as i32,
                );
                self.set((x + dx, y - 4 + dy), STAR);
            }
        }
        for burst in &world.flash_bursts {
            self.paint_burst(burst);
        }
    }

    /// An enemy felled by the flashball: a white flash, then drops of blood and chunks thrown
    /// out the way the shot went, fanned out, falling to the ground.
    fn paint_burst(&mut self, burst: &FlashBurst) {
        if burst.age >= FLASH_BURST_TICKS {
            return;
        }
        let (x, y) = pixel(burst.at);
        if burst.age < FLASH_TICKS {
            self.set((x, y), FLASH);
        }
        let second = f32::from(TICKS_PER_SECOND);
        let age = f32::from(u16::try_from(burst.age).unwrap_or(0)) / second;
        let flown = f32::from(u16::try_from(burst.age.min(FLY_TICKS)).unwrap_or(0)) / second;
        let base = burst.way.y.atan2(burst.way.x);
        for drop in 0..DROPS {
            let luck = drop.wrapping_mul(2_654_435_761) ^ 0x9e37;
            let spread = f32::from(u16::try_from(luck % 140).unwrap_or(0)) - 70.0;
            let angle = base + spread.to_radians();
            let speed = 10.0 + f32::from(u16::try_from((luck >> 8) % 14).unwrap_or(0));
            let along = Vec2::new(angle.cos(), angle.sin()) * (flown * speed);
            let fall = ((age - 0.2).max(0.0) * 6.0).min(3.0);
            #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
            let (dx, dy) = (along.x.round() as i32, (along.y + fall).round() as i32);
            let colour = if drop % 4 == 0 {
                CHUNK
            } else if drop % 3 == 0 {
                DARK_BLOOD
            } else {
                BLOOD
            };
            self.set((x + dx, y + dy.min(3)), colour);
        }
    }
}
