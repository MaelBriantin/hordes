//! The napalm's bombs as they fall: a streak dropping from the sky onto where each lands, a
//! white head, grey behind; where they land, a fireball as the C4's, the ground burning as a
//! Molotov's puddle.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::World;

const HEAD: Color = Color::Rgb(255, 250, 225);
const TAIL: Color = Color::Rgb(120, 118, 112);
/// How many pixels a falling bomb drops a tick, and how long its tail is.
const DROP: u32 = 2;
const TAIL_PIXELS: i32 = 2;

impl Canvas<'_> {
    /// The bombs falling, the last of their fall: each a streak dropping onto where it lands.
    pub(super) fn paint_bombs(&mut self, world: &World) {
        for bomb in world.falling_bombs() {
            let (x, y) = pixel(bomb.at);
            let top = y - i32::try_from(bomb.left * DROP).unwrap_or(i32::MAX);
            self.set((x, top), HEAD);
            for behind in 1..=TAIL_PIXELS {
                self.set((x, top - behind), TAIL);
            }
        }
    }
}
