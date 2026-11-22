//! The shield ram's bracing: dust kicked up behind the player's feet as they brace, about to
//! charge.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::World;

const DUST: Color = Color::Rgb(150, 120, 85);

impl Canvas<'_> {
    /// Braced before a charge, dust behind the player's feet, the other way from where they face,
    /// stirring.
    pub(super) fn paint_brace(&mut self, world: &World) {
        if !world.bracing() {
            return;
        }
        let (x, y) = pixel(world.player.position);
        let way = world.player.facing.way();
        #[allow(clippy::cast_possible_truncation, reason = "a way: -1, 0 or 1")]
        let (dx, dy) = (way.x.round() as i32, way.y.round() as i32);
        let stir = i32::try_from(world.ticks() % 2).unwrap_or(0);
        for (along, across) in [(3, -1), (3, 1), (4, stir)] {
            self.set(
                (
                    x - dx * along - dy * across,
                    y + 1 - dy * along + dx * across,
                ),
                DUST,
            );
        }
    }
}
