//! The C4: its charges on the ground, a small dark block, their red light blinking and glowing
//! in the dark; their blasts, a white flash over the whole blast, then a fireball shrinking,
//! then smoke rising.

use ratatui::style::Color;

use super::super::night;
use super::{Canvas, pixel};
use crate::game::{Fireball, World};

const BODY: Color = Color::Rgb(90, 95, 105);
const LIGHT_ON: Color = Color::Rgb(255, 60, 60);
const LIGHT_OFF: Color = Color::Rgb(140, 20, 25);
/// The light's glow around it, lit.
const GLOW: (u8, u8, u8) = (255, 60, 60);
const FLASH: Color = Color::Rgb(255, 250, 225);
/// The fireball, from its edge to its heart.
const FIREBALL: [Color; 3] = [
    Color::Rgb(210, 40, 40),
    Color::Rgb(245, 130, 40),
    Color::Rgb(255, 225, 100),
];
const SMOKE: Color = Color::Rgb(85, 85, 88);
/// A blast's ticks: its flash, then its fireball until it is gone; its smoke from when, to the
/// end of the blast; how many puffs.
const FLASH_TICKS: u32 = 2;
const FIREBALL_TICKS: u32 = 12;
const SMOKE_FROM: u32 = 6;
const PUFFS: u32 = 14;
/// A blast's light and halo while its fireball burns: its colour, how strong at its middle,
/// how far it reaches in pixels.
const BLAST_HALO: ((u8, u8, u8), f32, i32) = ((255, 170, 60), 0.6, 18);
/// A kamikaze's blast, its fire green: its fireball, from its edge to its heart, and its halo.
const KAMIKAZE_FIREBALL: [Color; 3] = [
    Color::Rgb(50, 150, 40),
    Color::Rgb(120, 220, 70),
    Color::Rgb(210, 255, 150),
];
const KAMIKAZE_BLAST_HALO: ((u8, u8, u8), f32, i32) = ((120, 220, 70), 0.6, 18);

impl Canvas<'_> {
    /// The charges on the ground, under the figures: a dark block, its light on top, dark red
    /// while off.
    pub(super) fn paint_charges(&mut self, world: &World) {
        for charge in &world.charges {
            let (x, y) = pixel(charge.at);
            for dx in -1..=1 {
                self.set((x + dx, y + 2), BODY);
            }
            self.set((x, y + 1), LIGHT_OFF);
        }
    }

    /// The charges' lights on, each glowing red around it.
    pub(super) fn charge_glows(world: &World) -> Vec<night::Glow> {
        world
            .charges
            .iter()
            .filter(|charge| charge.lit())
            .map(|charge| {
                let (x, y) = pixel(charge.at);
                ((x, y + 1), GLOW)
            })
            .collect()
    }

    /// The charges' lights on, over the night and their glow.
    pub(super) fn paint_charge_lights(&mut self, world: &World) {
        for (at, _) in Self::charge_glows(world) {
            self.set(at, LIGHT_ON);
        }
    }

    /// The blasts lighting around them while their fireball burns, warm; the kamikazes', green.
    pub(super) fn blast_lights(world: &World) -> (Vec<night::Light>, Vec<night::Halo>) {
        world
            .fireballs()
            .map(|blast| (blast, BLAST_HALO))
            .chain(
                world
                    .kamikaze_blasts
                    .iter()
                    .map(|blast| (blast, KAMIKAZE_BLAST_HALO)),
            )
            .filter(|(blast, _)| blast.age < FIREBALL_TICKS)
            .map(|(blast, (colour, strength, reach))| {
                let centre = pixel(blast.at);
                let left = 1.0 - night::share(blast.age, FIREBALL_TICKS);
                (
                    night::Light::burst(centre, left),
                    night::Halo {
                        centre,
                        colour,
                        strength: strength * left,
                        reach,
                    },
                )
            })
            .unzip()
    }

    /// The blasts: a white flash over the whole blast, then a fireball shrinking, red, orange,
    /// a yellow heart (the kamikazes', green); smoke rising from it, thinning out.
    pub(super) fn paint_fireballs(&mut self, world: &World) {
        for blast in world.fireballs() {
            self.paint_fireball(blast, FIREBALL);
        }
        for blast in &world.kamikaze_blasts {
            self.paint_fireball(blast, KAMIKAZE_FIREBALL);
        }
    }

    /// A blast, its fireball in `colours`, from its edge to its heart.
    fn paint_fireball(&mut self, blast: &Fireball, colours: [Color; 3]) {
        let centre = pixel(blast.at);
        let reach = blast.radius * 2.0;
        if blast.age < FLASH_TICKS {
            self.disc(centre, reach, FLASH);
        } else if blast.age < FIREBALL_TICKS {
            let size = reach
                * (1.0 - night::share(blast.age - FLASH_TICKS, FIREBALL_TICKS - FLASH_TICKS + 2));
            for (colour, share) in colours.into_iter().zip([1.0, 0.75, 0.4]) {
                self.disc(centre, size * share, colour);
            }
        }
        if blast.age >= SMOKE_FROM {
            self.smoke(centre, blast.age - SMOKE_FROM);
        }
    }

    /// A disc of `colour`, `radius` pixels around `centre`.
    fn disc(&mut self, (cx, cy): (i32, i32), radius: f32, colour: Color) {
        #[allow(clippy::cast_possible_truncation, reason = "a blast of a few tiles")]
        let reach = radius.ceil() as i32;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let distance = f32::from(i16::try_from(dx).unwrap_or(i16::MAX))
                    .hypot(f32::from(i16::try_from(dy).unwrap_or(i16::MAX)));
                if distance <= radius {
                    self.set((cx + dx, cy + dy), colour);
                }
            }
        }
    }

    /// Puffs of smoke rising from `centre`, `age` ticks after they started, fewer and fewer.
    fn smoke(&mut self, (cx, cy): (i32, i32), age: u32) {
        for puff in 0..PUFFS {
            let luck = puff.wrapping_mul(2_654_435_761);
            // Gone one by one as the smoke thins out.
            if (luck >> 8) % 12 < age / 3 {
                continue;
            }
            let angle = f32::from(u16::try_from(luck % 628).unwrap_or(0)) / 100.0;
            let away = f32::from(u16::try_from((luck >> 4) % 6).unwrap_or(0));
            let rise = f32::from(u16::try_from(age).unwrap_or(0))
                * (0.2 + f32::from(u16::try_from((luck >> 12) % 10).unwrap_or(0)) / 40.0);
            #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
            let (dx, dy) = (
                (away * angle.cos()).round() as i32,
                (away * angle.sin() * 0.6 - rise).round() as i32,
            );
            self.set((cx + dx, cy + dy), SMOKE);
        }
    }
}
