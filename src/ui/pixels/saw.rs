//! The chainsaw, while it grinds: held out ahead of the hero, the way they face, its motor then
//! its blade, the teeth running along its edges, blood on its end and its edge as the blood
//! setting has it; the hero's corners gleaming, nothing hurting them meanwhile.

use ratatui::style::Color;

use super::super::night::BloodLook;
use super::{Canvas, pixel};
use crate::game::World;

/// How long the blade is, out from the motor, in pixels.
const BLADE: i32 = 4;
const MOTOR: Color = Color::Rgb(230, 120, 40);
const STEEL: Color = Color::Rgb(205, 205, 215);
const TEETH: Color = Color::Rgb(90, 95, 105);
/// Around the hero while it grinds, nothing hurting them: a pale gleam at their corners,
/// blinking.
const GUARD: Color = Color::Rgb(200, 225, 255);
const GUARD_CORNERS: [(i32, i32); 4] = [(-3, -3), (2, -3), (-3, 2), (2, 2)];

impl Canvas<'_> {
    /// The chainsaw held out, grinding: its motor at the hero's edge, its blade ahead, the teeth
    /// running on either edge, blood flicking on its end and along its edge.
    pub(super) fn paint_saw(&mut self, world: &World, blood: BloodLook) {
        if !world.sawing {
            return;
        }
        let (x, y) = pixel(world.player.position);
        let way = world.player.facing.way();
        #[allow(clippy::cast_possible_truncation, reason = "a way: -1, 0 or 1")]
        let (dx, dy) = (way.x.round() as i32, way.y.round() as i32);
        let (across_x, across_y) = (-dy, dx);
        // From just past the hero's sprite, the way they face: two pixels wide, four tall.
        let edge = match (dx, dy) {
            (1, _) => (x + 1, y),
            (-1, _) => (x - 2, y),
            (_, 1) => (x, y + 2),
            _ => (x, y - 3),
        };
        let at = |along: i32, across: i32| {
            (
                edge.0 + dx * along + across_x * across,
                edge.1 + dy * along + across_y * across,
            )
        };
        let frame = world.ticks();
        // Nothing hurts the hero meanwhile: their corners gleam, two at a time.
        for (corner, (cx, cy)) in (0_u32..).zip(GUARD_CORNERS) {
            if (corner + frame / 3).is_multiple_of(2) {
                self.set((x + cx, y + cy), GUARD);
            }
        }
        self.set(at(0, 0), MOTOR);
        self.set(at(0, 1), MOTOR);
        for along in 1..=BLADE {
            self.set(at(along, 0), STEEL);
            // The teeth running: on one edge then the other, along the blade.
            let side = if (along + i32::try_from(frame % 2).unwrap_or(0)) % 2 == 0 {
                -1
            } else {
                1
            };
            self.set(at(along, side), TEETH);
        }
        if let Some([dark, bright]) = blood.colours() {
            self.set(at(BLADE + 1, 0), bright);
            let along = 2 + i32::try_from(frame / 2 % 3).unwrap_or(0);
            self.set(at(along, -1), dark);
        }
    }
}
