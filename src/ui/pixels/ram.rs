//! The shield ram's bracing: dust kicked up behind the player's feet as they brace, about to
//! charge. The agent's vault: their shadow under them, speed lines behind.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::World;

const DUST: Color = Color::Rgb(150, 120, 85);
/// A vault: the agent's shadow on the ground under them, and the speed lines behind.
const VAULT_SHADOW: Color = Color::Rgb(32, 44, 32);
const SPEED_LINES: Color = Color::Rgb(190, 190, 205);

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

    /// Vaulting, the agent's shadow on the ground under them, and speed lines behind them, a
    /// quick, low jump.
    pub(super) fn paint_vault(&mut self, world: &World) {
        let Some(vault) = world.player.vault else {
            return;
        };
        let (x, y) = pixel(world.player.position);
        self.set((x - 1, y + 2), VAULT_SHADOW);
        self.set((x, y + 2), VAULT_SHADOW);
        let way = vault.way();
        #[allow(clippy::cast_possible_truncation, reason = "a way: -1, 0 or 1")]
        let (dx, dy) = (way.x.round() as i32, way.y.round() as i32);
        for step in 0..3 {
            let (along, across) = (4 + step, step - 1);
            self.set(
                (x - dx * along - dy * across, y - dy * along + dx * across),
                SPEED_LINES,
            );
        }
    }
}
