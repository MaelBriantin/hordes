//! Revival: the hero's soul out of their body, up, then back into it, the body grey meanwhile;
//! then a white ring spreading from them (the green crosses are the heal's).

use ratatui::style::Color;

use super::super::sprites::{self, Look, Outfit};
use super::{Canvas, pixel};
use crate::game::{REVIVAL_SOUL_TICKS, World};

/// The soul, pale blue; how high it rises at its height, in tiles.
const SOUL: Color = Color::Rgb(200, 220, 255);
const SOUL_LIFT: f32 = 3.0;
/// The body while the soul is out of it.
pub(super) const LIFELESS: Color = Color::Rgb(90, 90, 95);
/// The ring: white, how fast it spreads, in pixels a tick, and its points.
const RING: Color = Color::Rgb(255, 250, 225);
const RING_SPEED: f32 = 0.67;
const RING_POINTS: u16 = 28;
/// How much flatter than round the ring is.
const RING_FLAT: f32 = 0.8;

impl Canvas<'_> {
    /// Come back from 0 HP: the soul up and back down into the body, then the ring.
    pub(super) fn paint_revival(&mut self, world: &World, outfit: Outfit) {
        let Some(age) = world.revived else {
            return;
        };
        if age < REVIVAL_SOUL_TICKS {
            let share = f32::from(u16::try_from(age).unwrap_or(0))
                / f32::from(u16::try_from(REVIVAL_SOUL_TICKS).unwrap_or(1));
            let lift = SOUL_LIFT * (std::f32::consts::PI * share).sin();
            let look =
                sprites::hero_in(world.hero_facing(), outfit, world.torch_on() == Some(false));
            self.paint_at(
                Look {
                    tint: Some(SOUL),
                    ..look
                },
                world.player.position - crate::game::Vec2::new(0.0, lift),
            );
            return;
        }
        let (x, y) = pixel(world.player.position);
        let radius =
            1.0 + f32::from(u16::try_from(age - REVIVAL_SOUL_TICKS).unwrap_or(0)) * RING_SPEED;
        for point in 0..RING_POINTS {
            let angle = std::f32::consts::TAU * f32::from(point) / f32::from(RING_POINTS);
            #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
            let (dx, dy) = (
                (radius * angle.cos()).round() as i32,
                (radius * angle.sin() * RING_FLAT).round() as i32,
            );
            self.set((x + dx, y + dy), RING);
        }
    }
}
