//! The spitters' acid: a spitter pulling back to spit, its acid glinting at its mouth; its
//! globs lobbed through the air, where they will land blinking on the ground; their puddles.

use ratatui::style::Color;

use super::super::night;
use super::super::sprites;
use super::figures::ZOMBIE_LEGS;
use super::{Canvas, pixel};
use crate::game::{EnemyKind, Facing, Puddle, SPIT_WIND_TICKS, Vec2, World};

/// The acid, glinting, then its middle, then its darker edge: the spitter's own colours.
pub(super) const ACID: [Color; 3] = [
    Color::Rgb(255, 240, 120),
    Color::Rgb(225, 205, 40),
    Color::Rgb(150, 130, 20),
];
/// How high a glob flies at most, in pixels.
const GLOB_ARC: f32 = 8.0;
/// Where a glob will land, a ring of pixels round it, and the ticks it shows, then hides.
const TARGET_RING: [(i32, i32); 8] = [
    (-2, 0),
    (2, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];
const TARGET_BLINK: u32 = 5;
/// How far a spitter pulls back to spit, in tiles: a pixel.
pub(super) const PULL_BACK: f32 = 0.5;
/// A spitter about to spit, in the dark: its acid pulsing up to near white, how fast (radians
/// a tick); its light, in tiles, and its halo, as a lit kamikaze's.
const SPIT_BRIGHT: Color = Color::Rgb(255, 250, 200);
const SPIT_PACE: f32 = 0.8;
const SPIT_LIGHT: f32 = 2.0;
const SPIT_HALO: ((u8, u8, u8), f32, i32) = ((225, 205, 40), 0.3, 8);
/// A puddle of acid's glow in the dark: its light, in tiles, and its halo.
pub(super) const PUDDLE_LIGHT: f32 = 1.0;
pub(super) const PUDDLE_HALO: ((u8, u8, u8), f32, i32) = ((225, 205, 40), 0.25, 5);

/// Whether a spitter pulls back to spit: the last of its wait.
pub(super) fn pulling_back(enemy: &crate::game::Enemy) -> bool {
    enemy.kind == EnemyKind::Spitter && enemy.spit_wait <= SPIT_WIND_TICKS
}

impl Canvas<'_> {
    /// The spitters pulling back to spit, over the night, so that they show in the dark about
    /// to spit: their acid pulsing bright, their clothes as they are, not their legs; sideways,
    /// their acid glinting at their mouth, on and off.
    pub(super) fn paint_spit_glow(&mut self, world: &World) {
        let glint = (world.ticks() / 2).is_multiple_of(2);
        let time = f32::from(u16::try_from(world.ticks() % 10_000).unwrap_or(0));
        let bright =
            super::super::blend(ACID[0], SPIT_BRIGHT, 0.5 + 0.5 * (time * SPIT_PACE).sin());
        for enemy in world.enemies.iter().filter(|enemy| pulling_back(enemy)) {
            let look = sprites::enemy(enemy, world.ticks());
            let at = enemy.position - enemy.facing.way() * PULL_BACK;
            let (left, top) = self.corner_of(look, at);
            let pixels: Vec<_> = self
                .sprites
                .get(look.sprite)
                .pixels(look.mirrored)
                .filter(|&(_, _, colour)| colour != ZOMBIE_LEGS)
                .collect();
            for (x, y, colour) in pixels {
                let colour = if colour == ACID[0] || colour == ACID[1] {
                    bright
                } else {
                    colour
                };
                self.set((left + x, top + y), colour);
            }
            let ahead = match enemy.facing {
                Facing::Right => 4,
                Facing::Left => -1,
                Facing::Up | Facing::Down => continue,
            };
            if glint {
                self.set((left + ahead, top + 1), ACID[0]);
            }
        }
    }

    /// The globs in the air, over the night: where each will land blinking on the ground, and
    /// the glob itself, high on its arc.
    pub(super) fn paint_globs(&mut self, world: &World) {
        let blink = (world.ticks() / TARGET_BLINK).is_multiple_of(2);
        for glob in &world.globs {
            let (tx, ty) = pixel(glob.to);
            if blink {
                for (dx, dy) in TARGET_RING {
                    self.set((tx + dx, ty + dy), ACID[2]);
                }
            }
            let progress = glob.progress();
            let lift = (std::f32::consts::PI * progress).sin() * GLOB_ARC / 2.0;
            let along = glob.from + (glob.to - glob.from) * progress - Vec2::new(0.0, lift);
            let (x, y) = pixel(along);
            self.set((x, y), ACID[0]);
            self.set((x + 1, y), ACID[1]);
            self.set((x, y + 1), ACID[1]);
            self.set((x + 1, y + 1), ACID[2]);
        }
    }

    /// The puddles of acid on the ground, under the figures.
    pub(super) fn paint_acid_puddles(&mut self, world: &World, frame: u32) {
        for puddle in world.puddles.iter().filter(|puddle| puddle.acid) {
            self.paint_acid_puddle(puddle, frame);
        }
    }

    /// A puddle of acid on the ground: its edge darker, its middle glinting here and there, a
    /// bubble now and then; its last moment, breaking up.
    fn paint_acid_puddle(&mut self, puddle: &Puddle, frame: u32) {
        let (cx, cy) = pixel(puddle.centre);
        #[allow(clippy::cast_possible_truncation, reason = "a puddle of a few tiles")]
        let reach = (puddle.radius * 2.0).ceil() as i32;
        let going = puddle.left * 5 <= puddle.lasts;
        for dy in -reach..=reach {
            for dx in -reach - 1..=reach + 1 {
                // Wider than high, as a puddle seen from above at a slant.
                let distance = f32::from(i16::try_from(dx).unwrap_or(0))
                    .hypot(f32::from(i16::try_from(dy).unwrap_or(0)) * 1.5);
                if distance > puddle.radius * 2.0 {
                    continue;
                }
                let luck = (cx + dx).cast_unsigned().wrapping_mul(73_856_093)
                    ^ (cy + dy).cast_unsigned().wrapping_mul(19_349_663)
                    ^ (frame / 4).wrapping_mul(83_492_791);
                if going && luck.is_multiple_of(2) {
                    continue;
                }
                let colour = if luck.is_multiple_of(3) {
                    ACID[1]
                } else {
                    ACID[2]
                };
                self.set((cx + dx, cy + dy), colour);
            }
        }
        let bubble = (cx.cast_unsigned() ^ (frame / 3).wrapping_mul(2_654_435_761)) % 4;
        if bubble == 0 {
            let dx = i32::try_from(frame % 3).unwrap_or(0) - 1;
            self.set((cx + dx, cy - 1), ACID[0]);
        }
    }

    /// The puddles of acid glowing a little in the dark, so that they are seen; the spitters
    /// about to spit, more.
    pub(super) fn acid_lights(world: &World) -> (Vec<night::Light>, Vec<night::Halo>) {
        let spitting = world
            .enemies
            .iter()
            .filter(|enemy| pulling_back(enemy))
            .map(|enemy| {
                let centre = pixel(enemy.position);
                let (colour, strength, reach) = SPIT_HALO;
                (
                    night::Light::cold_fire(centre, SPIT_LIGHT),
                    night::Halo {
                        centre,
                        colour,
                        strength,
                        reach,
                    },
                )
            });
        let (colour, strength, reach) = PUDDLE_HALO;
        world
            .puddles
            .iter()
            .filter(|puddle| puddle.acid)
            .map(|puddle| {
                let centre = pixel(puddle.centre);
                (
                    night::Light::cold_fire(centre, PUDDLE_LIGHT),
                    night::Halo {
                        centre,
                        colour,
                        strength,
                        reach,
                    },
                )
            })
            .chain(spitting)
            .unzip()
    }
}
