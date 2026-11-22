//! The demolition hammer's blow: the hammer raised over the hero's head, then down on the ground a
//! moment, the wave of its shock going out to its reach as it throws the enemies out; the waves
//! of the shocks following on.

use ratatui::style::Color;

use super::{Canvas, pixel};
use crate::game::{Blow, Facing, HAMMER_WAVE_TICKS as WAVE_TICKS, World};

/// How long the hammer shows down on the ground, in ticks.
const HAMMER_TICKS: u32 = 6;
/// The wave's band, its front first: white, pale, grey; and the ticks it fades once it has
/// stopped. It goes out over the hammer's wave ticks.
const WAVE: [Color; 3] = [
    Color::Rgb(245, 245, 235),
    Color::Rgb(190, 185, 170),
    Color::Rgb(120, 118, 112),
];
const FADE_TICKS: u32 = 4;
const STEEL: Color = Color::Rgb(205, 205, 215);
const DARK_STEEL: Color = Color::Rgb(90, 95, 105);
const WOOD: Color = Color::Rgb(150, 100, 60);

impl Canvas<'_> {
    /// The hammer raised over the player's head, about to fall; its last blow, while it shows.
    pub(super) fn paint_blow(&mut self, world: &World) {
        let (px, py) = pixel(world.player.position);
        if world.hammer_raised > 0 {
            // The handle up from the hero's head, the head on top of it.
            let column = if world.player.facing == Facing::Left {
                px - 1
            } else {
                px
            };
            for up in 3..=5 {
                self.set((column, py - up), WOOD);
            }
            self.hammer_head((column, py - 6));
        }
        for wave in &world.quake_waves {
            self.paint_wave(*wave);
        }
        let Some(blow) = world.blow else {
            return;
        };
        let (x, y) = pixel(blow.at);
        if blow.age < HAMMER_TICKS {
            // The handle from the player's hands to the head, down on the ground.
            for step in 1_u8..=4 {
                let share = f32::from(step) / 5.0;
                let along = |a: i32, b: i32| {
                    let apart = f32::from(i16::try_from(b - a).unwrap_or(0));
                    #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
                    let moved = (apart * share).round() as i32;
                    a + moved
                };
                self.set((along(px, x), along(py, y)), WOOD);
            }
            self.hammer_head((x, y));
        }
        self.paint_wave(blow);
    }

    /// A shock's wave: a thick band going out from where it landed to its reach, as fast as it
    /// throws the enemies out, white at its front; then fading where it stops.
    fn paint_wave(&mut self, blow: Blow) {
        let (x, y) = pixel(blow.at);
        let age = blow.age;
        let tick = |value: u32| f32::from(u16::try_from(value).unwrap_or(u16::MAX));
        let reach = blow.radius * 2.0;
        if age < WAVE_TICKS + FADE_TICKS {
            let out = tick((age + 1).min(WAVE_TICKS)) / tick(WAVE_TICKS);
            let front = reach * out;
            let shades: &[Color] = if age < WAVE_TICKS { &WAVE } else { &WAVE[1..] };
            for (behind, colour) in (0_u8..).zip(shades) {
                let radius = front - f32::from(behind);
                if radius < 0.5 {
                    continue;
                }
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a few dozen points"
                )]
                let points = (radius * 8.0).ceil() as u16;
                for point in 0..points {
                    let angle = std::f32::consts::TAU * f32::from(point) / f32::from(points);
                    #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
                    let at = (
                        x + (radius * angle.cos()).round() as i32,
                        y + (radius * angle.sin()).round() as i32,
                    );
                    self.set(at, *colour);
                }
            }
        }
    }

    /// The hammer's head, three pixels wide and two high, its bottom middle at `at`.
    fn hammer_head(&mut self, (x, y): (i32, i32)) {
        for dx in -1..=1 {
            self.set((x + dx, y - 1), STEEL);
            self.set((x + dx, y), DARK_STEEL);
        }
    }
}
