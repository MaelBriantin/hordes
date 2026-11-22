//! The electric strikes: the stun baton's thrust, the baton held out, its double prong ahead, a
//! jagged bolt running from it to the end of its reach, a spark on each enemy it paralyses; the
//! defibrillator's shock, a jagged bolt leaping from the hero through each enemy it goes
//! through.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::World;

/// How long the baton shows held out, and the bolt runs, in ticks.
const BATON_TICKS: u32 = 8;
const BOLT_TICKS: u32 = 5;
/// How far the bolt strays from its line, each side, in pixels.
const BOLT_STRAY: i32 = 2;
const STICK: Color = Color::Rgb(45, 45, 52);
const METAL: Color = Color::Rgb(205, 205, 215);
const WHITE: Color = Color::Rgb(235, 245, 255);
const CYAN: Color = Color::Rgb(120, 220, 255);

impl Canvas<'_> {
    /// The baton's last thrust, while it shows.
    pub(super) fn paint_thrust(&mut self, world: &World) {
        let Some(thrust) = world.thrust else {
            return;
        };
        let (x, y) = pixel(world.player.position);
        #[allow(clippy::cast_possible_truncation, reason = "a way: -1, 0 or 1")]
        let (dx, dy) = (thrust.way.x.round() as i32, thrust.way.y.round() as i32);
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
        if thrust.age < BATON_TICKS {
            for along in 0..3 {
                self.set(at(along, 0), if along < 2 { STICK } else { METAL });
            }
            // The double prong.
            self.set(at(3, -1), METAL);
            self.set(at(3, 1), METAL);
            self.set(at(4, -1), METAL);
            self.set(at(4, 1), METAL);
        }
        if thrust.age < BOLT_TICKS {
            let colour = if thrust.age < 2 { WHITE } else { CYAN };
            #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
            let reach = (thrust.reach * 2.0).round() as i32;
            let mut stray = 0;
            for along in 4..=reach {
                let luck = along.cast_unsigned().wrapping_mul(73_856_093)
                    ^ world.ticks().wrapping_mul(83_492_791);
                stray = (stray + i32::try_from(luck % 3).unwrap_or(1) - 1)
                    .clamp(-BOLT_STRAY, BOLT_STRAY);
                self.set(at(along, stray), colour);
            }
            // A spark on each enemy it paralyses.
            for enemy in world.enemies.iter().filter(|enemy| enemy.shocked > 0) {
                let (ex, ey) = pixel(enemy.position);
                self.set((ex + 1, ey - 3), WHITE);
                self.set((ex + 2, ey - 4), CYAN);
            }
        }
    }

    /// The defibrillator's last shock: a jagged bolt from the hero to the first enemy shocked,
    /// then on through each next one, white then cyan.
    pub(super) fn paint_shock(&mut self, world: &World) {
        let Some(shock) = &world.shock else {
            return;
        };
        let colour = if shock.age < 2 { WHITE } else { CYAN };
        for (leg, pair) in (0_u32..).zip(shock.path.windows(2)) {
            let (from, to) = (pixel(pair[0]), pixel(pair[1]));
            let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).max(1);
            let (across_x, across_y) = ((to.1 - from.1).signum(), -(to.0 - from.0).signum());
            let mut stray = 0;
            for step in 0..=steps {
                let luck = u32::try_from(step).unwrap_or(0).wrapping_mul(73_856_093)
                    ^ leg.wrapping_mul(19_349_663)
                    ^ world.ticks().wrapping_mul(83_492_791);
                stray = (stray + i32::try_from(luck % 3).unwrap_or(1) - 1)
                    .clamp(-BOLT_STRAY, BOLT_STRAY);
                // Back to the line at either end.
                let stray = if step == 0 || step == steps { 0 } else { stray };
                let point = (
                    from.0 + (to.0 - from.0) * step / steps + across_x * stray,
                    from.1 + (to.1 - from.1) * step / steps + across_y * stray,
                );
                self.set(point, colour);
            }
        }
    }
}
