//! The fists' punches: a fist out from the hero to the enemy struck, a small white burst on it;
//! a heavier one, a bigger, yellow burst.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::World;

const FIST: Color = Color::Rgb(240, 196, 150);
const BURST: Color = Color::Rgb(245, 245, 235);
const HEAVY: Color = Color::Rgb(255, 225, 100);
/// How long the fist shows out, in ticks.
const FIST_TICKS: u32 = 3;

impl Canvas<'_> {
    /// The last punch, while it shows: the fist out halfway to the enemy struck; a burst on it,
    /// bigger and yellow for a heavier one.
    pub(super) fn paint_punch(&mut self, world: &World) {
        let Some(punch) = world.punch else {
            return;
        };
        let (px, py) = pixel(world.player.position);
        let (ex, ey) = pixel(punch.at);
        if punch.age < FIST_TICKS {
            self.set((i32::midpoint(px, ex), i32::midpoint(py, ey)), FIST);
        }
        let colour = if punch.heavy { HEAVY } else { BURST };
        let reach = if punch.heavy { 2 } else { 1 };
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for step in 1..=reach {
                self.set((ex + dx * step, ey - 2 + dy * step), colour);
            }
        }
    }
}
