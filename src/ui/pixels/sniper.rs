//! The sniper rifle: its red dot on the enemy it aims at, glowing a little; its bullet, a short
//! bright tracer; where it strikes, a white star and blood thrown out behind, the way it went.
//! All over the night: it shows in the dark.

use ratatui::style::Color;

use super::super::night::{self, BloodLook};
use super::{Canvas, pixel};
use crate::game::{SNIPER_IMPACT_TICKS, World};

const DOT: Color = Color::Rgb(255, 50, 50);
const GLOW: (u8, u8, u8) = (255, 50, 50);
const TRACER: Color = Color::Rgb(255, 235, 150);
const TRACER_TAIL: Color = Color::Rgb(255, 250, 225);
const STAR: Color = Color::Rgb(255, 250, 225);
/// The impact's white star, its ticks; its drops of blood, how many, from when.
const STAR_TICKS: u32 = 3;
const DROPS: u32 = 10;
const DROPS_FROM: u32 = 1;

impl Canvas<'_> {
    /// The red dot's glow, on the enemy aimed at, at its middle.
    pub(super) fn sniper_glows(world: &World) -> Vec<night::Glow> {
        world
            .sniper_dot()
            .map(|at| (pixel(at), GLOW))
            .into_iter()
            .collect()
    }

    /// The red dot itself, over the night and its glow.
    pub(super) fn paint_sniper_dot(&mut self, world: &World) {
        if let Some(at) = world.sniper_dot() {
            self.set(pixel(at), DOT);
        }
    }

    /// The bullet in the air: a short tracer, its head bright, a white pixel behind.
    pub(super) fn paint_sniper_shot(&mut self, world: &World) {
        let Some(shot) = world.shot else {
            return;
        };
        let at = shot.from + (shot.to - shot.from) * shot.progress();
        let way = (shot.to - shot.from).normalized();
        self.set(pixel(at), TRACER);
        self.set(pixel(at - way * 0.5), TRACER_TAIL);
    }

    /// Where the bullets struck: a white star a moment, then drops of blood thrown out behind,
    /// the way the bullet went, a little scattered, as the blood setting has it.
    pub(super) fn paint_sniper_impacts(&mut self, world: &World, blood: BloodLook) {
        for impact in &world.impacts {
            let (x, y) = pixel(impact.at);
            if impact.age < STAR_TICKS {
                for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.set((x + dx, y + dy), STAR);
                }
            }
            let Some(colours) = blood.colours() else {
                continue;
            };
            if impact.age < DROPS_FROM || impact.age >= SNIPER_IMPACT_TICKS {
                continue;
            }
            let age = f32::from(u16::try_from(impact.age).unwrap_or(0));
            let across = (-impact.way.y, impact.way.x);
            for drop in 0..DROPS {
                let luck = drop.wrapping_mul(2_654_435_761);
                let far =
                    (0.5 + f32::from(u16::try_from(luck % 10).unwrap_or(0)) / 10.0) * age * 0.6;
                let far = far.min(5.0);
                let spread = (f32::from(u16::try_from((luck >> 8) % 7).unwrap_or(3)) - 3.0)
                    * 0.25
                    * far
                    * 0.4;
                #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
                let point = (
                    x + (impact.way.x * far + across.0 * spread).round() as i32,
                    y + (impact.way.y * far + across.1 * spread).round() as i32,
                );
                self.set(point, colours[usize::try_from(drop % 2).unwrap_or(0)]);
            }
        }
    }
}
