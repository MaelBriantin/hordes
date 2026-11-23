//! The veteran's gatling burst: the gun in their hands while it fires, the way they face, a flash
//! at its muzzle; the spent cases falling at their feet, lying there a moment.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::{CASE_TICKS, Vec2, World};

/// The gun's body and barrels; its muzzle's flash, white then yellow round it.
const BODY: Color = Color::Rgb(90, 95, 105);
const BARRELS: Color = Color::Rgb(205, 205, 215);
const FLASH: Color = Color::Rgb(255, 250, 225);
const FLASH_ROUND: Color = Color::Rgb(255, 225, 100);
/// How far the muzzle is from the veteran's middle, in pixels; where the barrels start.
const MUZZLE: i32 = 5;
const BARRELS_FROM: i32 = 3;
/// A spent case: brass, then dulled once it has lain a while; the ticks its fall takes, and how
/// high it flies first, in tiles.
const BRASS: Color = Color::Rgb(230, 180, 60);
const OLD_BRASS: Color = Color::Rgb(150, 115, 40);
const FALL_TICKS: u32 = 9;
const CASE_LIFT: f32 = 0.75;

impl Canvas<'_> {
    /// While the gatling fires, the gun the way the veteran faces, its muzzle flashing every
    /// other frame; and every spent case, falling, then lying at their feet.
    pub(super) fn paint_gatling(&mut self, world: &World, frame: u32) {
        for case in &world.cases {
            let t = (f32::from(u16::try_from(case.age.min(FALL_TICKS)).unwrap_or(0))
                / f32::from(u16::try_from(FALL_TICKS).unwrap_or(1)))
            .min(1.0);
            let lift = CASE_LIFT * 4.0 * t * (1.0 - t);
            let at = Vec2::new(
                case.from.x + (case.to.x - case.from.x) * t,
                case.from.y + (case.to.y - case.from.y) * t * t - lift,
            );
            let colour = if case.age < CASE_TICKS * 2 / 3 {
                BRASS
            } else {
                OLD_BRASS
            };
            self.set(pixel(at), colour);
        }
        if !world.firing_gatling() {
            return;
        }
        let (x, y) = pixel(world.player.position);
        let way = world.player.facing.way();
        #[allow(clippy::cast_possible_truncation, reason = "a way: -1, 0 or 1")]
        let (dx, dy) = (way.x.round() as i32, way.y.round() as i32);
        for along in 1..MUZZLE {
            let colour = if along < BARRELS_FROM { BODY } else { BARRELS };
            self.set((x + dx * along, y + dy * along), colour);
        }
        if frame.is_multiple_of(2) {
            let (mx, my) = (x + dx * MUZZLE, y + dy * MUZZLE);
            self.set((mx, my), FLASH);
            for (sx, sy) in [(dx, dy), (dy, dx), (-dy, -dx)] {
                self.set((mx + sx, my + sy), FLASH_ROUND);
            }
        }
    }
}
