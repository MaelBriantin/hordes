//! The combat knife's stabs: the blade flicking out from the hero to the enemy it stabs, and a
//! drop of blood falling from it, as the blood setting has it.

use ratatui::style::Color;

use super::super::night::BloodLook;
use super::{Canvas, pixel};
use crate::game::World;

/// How long the blade shows, in ticks; when the drop starts to fall, and how many ticks it takes
/// a pixel.
const BLADE_TICKS: u32 = 3;
const DROP_FROM: u32 = 1;
const DROP_FALL: u32 = 2;
/// How many pixels of the blade show, out from the hero: one less on a diagonal, longer to the
/// eye.
const BLADE: i32 = 3;
const STEEL: Color = Color::Rgb(205, 205, 215);

impl Canvas<'_> {
    /// The knife's last stabs, while they show.
    pub(super) fn paint_stabs(&mut self, world: &World, blood: BloodLook) {
        let (px, py) = pixel(world.player.position);
        for stab in &world.stabs {
            let (ex, ey) = pixel(stab.at);
            if stab.age < BLADE_TICKS {
                // Out from the hero's edge, the way the enemy is.
                let (dx, dy) = ((ex - px).signum(), (ey - py).signum());
                let (start_x, start_y) = (
                    px + if dx > 0 { 1 } else { dx * 2 },
                    py + if dy > 0 { 2 } else { dy * 3 },
                );
                let length = if dx != 0 && dy != 0 { BLADE - 1 } else { BLADE };
                for along in 0..length {
                    self.set((start_x + dx * along, start_y + dy * along), STEEL);
                }
            }
            if stab.age >= DROP_FROM
                && let Some([_, bright]) = blood.colours()
            {
                let fallen = i32::try_from((stab.age - DROP_FROM) / DROP_FALL).unwrap_or(0);
                self.set((ex + 1, ey + 1 + fallen), bright);
            }
        }
    }
}
