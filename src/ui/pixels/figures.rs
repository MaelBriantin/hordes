//! The figures on the map and what marks them: the player's arrow and HP bar, the flames of
//! the enemies on fire, the Shadows' eyes, the hordes' veil, an abomination's burst.

use ratatui::style::Color;

use super::super::sprites::{self, Look, Outfit};
use super::super::{ViewState, night};
use super::{Arrow, Canvas, FLAME_COLOURS, FLAMES, burst_pixels, pixel};
use crate::game::{BURST_REACH, Blast, EnemyKind, REVIVAL_SOUL_TICKS, Vec2, Veil, World};

/// The Shadows' eyes, the red of the sprites' palette, and the darker red they shine with in the
/// dark.
const SHADOW_EYES: Color = Color::Rgb(230, 40, 40);
const SHADOW_EYES_GLOW: Color = Color::Rgb(160, 22, 26);
/// The hordes' veiled, as Horde Hunter has it: the red, at most how much on the very last ones,
/// pulsing, the ticks it takes to pulse once; how much, steady, on the others.
const HUNTER_VEIL: (u8, u8, u8) = (230, 40, 40);
const VEIL_MOST: f32 = 0.5;
const VEIL_PULSE: u32 = 30;
const VEIL_STEADY: f32 = 0.2;
/// An abomination's burst: the bile its disc tints the ground with, more towards its edge.
const BILE: (u8, u8, u8) = (150, 205, 60);
const BILE_MIDDLE: f32 = 0.35;
const BILE_EDGE: f32 = 0.85;
/// The silhouettes a dashing player leaves behind: how far behind, in tiles, and their colour,
/// fading.
const AFTERIMAGES: [(f32, Color); 2] = [
    (1.6, Color::Rgb(40, 44, 62)),
    (0.8, Color::Rgb(70, 78, 108)),
];
/// The arrow over the player's head, pointing down at them, as wide as they are; the pixels
/// between it and their head, and the ticks it spends up, then down, bouncing.
const ARROW: [&str; 2] = ["####", ".##."];
const ARROW_GAP: i32 = 2;
const ARROW_BOUNCE: u32 = 15;
/// The small HP bar over the player's head: its width, the pixels between it and the head, or
/// the arrow when it shows.
const MINI_BAR: i32 = 6;
const MINI_BAR_GAP: i32 = 1;

/// The ticks the mini bar shows, then hides, low on HP.
const MINI_BAR_BLINK: u32 = 6;
/// How far a kamikaze about to blow swells each side, in tiles: a pixel.
const SWELL: f32 = 0.5;
/// A kamikaze's flame, from its tip to its root.
pub(super) const KAMIKAZE_FLAME: [Color; 3] = [
    Color::Rgb(210, 255, 150),
    Color::Rgb(120, 220, 70),
    Color::Rgb(50, 150, 40),
];
/// Its pixels, facing right, from the sprite's top left: its head and the two rows over it;
/// whether each is a tongue, there or not.
const KAMIKAZE_TONGUES: [(i32, i32, bool); 5] = [
    (2, -2, true),
    (1, -1, true),
    (2, -1, false),
    (1, 0, false),
    (2, 0, false),
];

impl Canvas<'_> {
    /// The player picked out of the crowd as `view` says: an arrow over their head, always or a
    /// moment once pinged; and their HP in a small bar over it all, if it shows.
    pub(super) fn mark(&mut self, world: &World, view: ViewState) {
        let player = &world.player;
        let look = sprites::hero(world.hero_facing(), view.outfit());
        let arrow = view.arrow != Arrow::Off || view.pinged;
        if arrow {
            self.arrow(look, player.position, world.ticks(), view.arrow.colour());
        }
        // Low on HP, it blinks.
        let blinks_out = super::super::low_hp(world).is_some()
            && (world.ticks() / MINI_BAR_BLINK).is_multiple_of(2);
        if view.mini_bars && !blinks_out {
            self.mini_bars(world, look, arrow);
        }
    }

    /// The player's HP in a small bar over the head of their sprite as `paint_at` would place
    /// it; over the arrow if it shows, out of its way.
    pub(super) fn mini_bars(&mut self, world: &World, look: Look, over_arrow: bool) {
        let player = &world.player;
        let centre = pixel(player.position);
        let head = centre.1 - self.sprites.get(look.sprite).height() / 2;
        let arrow = if over_arrow {
            ARROW_GAP + i32::try_from(ARROW.len()).unwrap_or(0) + 1
        } else {
            0
        };
        let row = head - MINI_BAR_GAP - 1 - arrow;
        let hp = f32::from(u16::try_from(player.hp).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(player.max_hp().max(1)).unwrap_or(u16::MAX));
        let left = centre.0 - MINI_BAR / 2;
        self.mini_bar((left, row), MINI_BAR, hp, super::super::HP_COLOR);
    }

    /// A small bar of `width` pixels from `start`, `filled` of it (from 0 to 1) in `colour`, the
    /// rest faded.
    pub(super) fn mini_bar(&mut self, start: (i32, i32), width: i32, filled: f32, colour: Color) {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a share of a few pixels, clamped"
        )]
        let full =
            (filled.clamp(0.0, 1.0) * f32::from(u8::try_from(width).unwrap_or(0))).round() as i32;
        for x in 0..width {
            let shade = if x < full {
                colour
            } else {
                super::super::faded(colour, super::super::BAR_EMPTY)
            };
            self.set((start.0 + x, start.1), shade);
        }
    }

    /// An arrow over the head of a sprite as `paint_at` would place it, pointing down at it,
    /// bouncing a pixel up and down, `now` the run's tick, in `colour`.
    pub(super) fn arrow(&mut self, look: Look, position: Vec2, now: u32, colour: Color) {
        let centre = pixel(position);
        let sprite = self.sprites.get(look.sprite);
        let width = i32::try_from(ARROW[0].len()).unwrap_or(0);
        let height = i32::try_from(ARROW.len()).unwrap_or(0);
        let up = i32::from((now / ARROW_BOUNCE).is_multiple_of(2));
        let left = centre.0 - width / 2;
        let top = centre.1 - sprite.height() / 2 - ARROW_GAP - height - up;
        for (y, row) in (0..).zip(ARROW) {
            for (x, key) in (0..).zip(row.chars()) {
                if key == '#' {
                    self.set((left + x, top + y), colour);
                }
            }
        }
    }

    /// Flames over each enemy on fire: a few pixels above its head, flickering; the kamikazes'
    /// heads.
    pub(super) fn paint_flames(&mut self, world: &World, frame: u32) {
        self.paint_kamikaze_flames(world, frame);
        for enemy in world.enemies.iter().filter(|enemy| enemy.burning > 0) {
            let look = sprites::enemy(enemy, world.ticks());
            let height = self.sprites.get(look.sprite).height();
            let (x, y) =
                pixel(enemy.position - Vec2::new(0.0, sprites::lift(enemy, world.ticks())));
            let top = y - height / 2;
            // Its own flicker, never quite the same as a neighbour's.
            let luck = (frame / 3).wrapping_add(enemy.id.wrapping_mul(7));
            for (index, &(dx, dy)) in (0_u32..).zip(FLAMES.iter()) {
                if index > 0 && (luck >> index) & 1 == 0 {
                    continue;
                }
                let shade = usize::try_from((index + luck) % 3).unwrap_or(0);
                self.set((x + dx, top + dy), FLAME_COLOURS[shade]);
            }
        }
    }

    /// The kamikazes' heads once their fuse is lit, a green flame, over the night: its root on
    /// the head, its tongues flickering over it; in the colour the kamikaze blinks in, if it
    /// does.
    fn paint_kamikaze_flames(&mut self, world: &World, frame: u32) {
        let [tip, middle, root] = KAMIKAZE_FLAME;
        for enemy in world
            .enemies
            .iter()
            .filter(|enemy| enemy.kind == EnemyKind::Kamikaze && enemy.fuse > 0)
        {
            let look = sprites::enemy(enemy, world.ticks());
            let (left, top) = self.corner_of(look, enemy.position);
            for (index, &(x, y, tongue)) in (0_u32..).zip(KAMIKAZE_TONGUES.iter()) {
                // Its own flicker, never quite the same as a neighbour's.
                let luck = (frame / 2)
                    .wrapping_add(enemy.id.wrapping_mul(13))
                    .wrapping_mul(2_654_435_761)
                    >> (index * 3 + 7);
                if tongue && luck & 1 == 0 {
                    continue;
                }
                let colour = look.tint.unwrap_or(if tongue || y < -1 {
                    tip
                } else {
                    [tip, middle, middle, root][usize::try_from(luck >> 1 & 3).unwrap_or(0)]
                });
                let x = if look.mirrored { 3 - x } else { x };
                self.set((left + x, top + y), colour);
            }
        }
    }

    /// An abomination's burst: a disc of bile spreading over the ground, then fading.
    pub(super) fn paint_blast(&mut self, blast: &Blast) {
        let centre = pixel(blast.centre);
        let reach = burst_pixels(BURST_REACH);
        let radius = burst_pixels(blast.radius());
        let strength = blast.strength();
        for y in -radius..=radius {
            for x in -radius..=radius {
                let apart = (x * x + y * y).isqrt();
                if apart > radius {
                    continue;
                }
                let Some(index) = self.index((centre.0 + x, centre.1 + y)) else {
                    continue;
                };
                let edge = night::share(apart.unsigned_abs(), reach.unsigned_abs());
                let by = strength * (BILE_MIDDLE + (BILE_EDGE - BILE_MIDDLE) * edge);
                if let Some(colour) = self.pixels[index] {
                    self.pixels[index] = Some(super::super::redden(colour, BILE, by));
                }
            }
        }
    }

    /// At night, the Shadows' eyes, shining red in the dark.
    pub(super) fn paint_shadow_eyes(&mut self, world: &World) {
        if world.torch_on().is_none() {
            return;
        }
        for enemy in world
            .enemies
            .iter()
            .filter(|enemy| enemy.kind == EnemyKind::Shadow)
        {
            self.repaint(
                sprites::enemy(enemy, world.ticks()),
                enemy.position,
                SHADOW_EYES,
                SHADOW_EYES_GLOW,
            );
        }
    }

    /// Only the pixels of a sprite in one `colour`, placed as `paint_at` would, in another one,
    /// `shown`: its eyes, say, shining in the dark.
    pub(super) fn repaint(&mut self, look: Look, position: Vec2, colour: Color, shown: Color) {
        let centre = pixel(position);
        let sprite = self.sprites.get(look.sprite);
        let (left, top) = (
            centre.0 - sprite.width() / 2,
            centre.1 - sprite.height() / 2,
        );
        for (x, y, pixel) in sprite.pixels(look.mirrored) {
            if pixel == colour {
                self.set((left + x, top + y), shown);
            }
        }
    }

    /// The player, the enemies and the barricades, the lowest on the map last: in front of those
    /// above them.
    /// While lightning is about to strike, which pixels they cover, for its ring to show on the
    /// ground under them; none otherwise.
    pub(super) fn paint_figures(&mut self, world: &World, outfit: Outfit) -> Vec<bool> {
        let player = &world.player;
        let mut figures: Vec<(Vec2, Look)> = world
            .enemies
            .iter()
            .flat_map(|enemy| {
                let lift = sprites::lift(enemy, world.ticks());
                let at = enemy.position - Vec2::new(0.0, lift);
                let look = sprites::enemy(enemy, world.ticks());
                // A kamikaze about to blow swells a pixel each side.
                let swell = if sprites::fuse_last(enemy) {
                    vec![
                        (at - Vec2::new(SWELL, 0.0), look),
                        (at + Vec2::new(SWELL, 0.0), look),
                    ]
                } else {
                    Vec::new()
                };
                swell.into_iter().chain([(at, look)])
            })
            .chain(
                world
                    .swellings
                    .iter()
                    .map(|swelling| (swelling.position, sprites::swelling(swelling))),
            )
            .collect();
        // Blinking while nothing can hurt them.
        let blinks_out = player.invincible > 0 && (world.ticks() / 3).is_multiple_of(2);
        if player.dash.is_some() || !blinks_out {
            // Vaulting, a little off the ground; their soul out of them, grey.
            let lift = player.vault.map_or(0.0, |vault| vault.lift());
            let look =
                sprites::hero_in(world.hero_facing(), outfit, world.torch_on() == Some(false));
            let lifeless = world.revived.is_some_and(|age| age < REVIVAL_SOUL_TICKS);
            figures.push((
                player.position - Vec2::new(0.0, lift),
                Look {
                    tint: lifeless.then_some(super::revival::LIFELESS),
                    ..look
                },
            ));
        }
        let mut figures: Vec<(f32, Look, (i32, i32))> = figures
            .into_iter()
            .map(|(position, look)| (position.y, look, self.corner_of(look, position)))
            .chain(self.barricade_figures(world))
            .collect();
        figures.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Dashing, the player leaves fading silhouettes of themselves behind.
        if let Some(dash) = player.dash {
            let look =
                sprites::hero_in(world.hero_facing(), outfit, world.torch_on() == Some(false));
            for (behind, colour) in AFTERIMAGES {
                let at = player.position - dash.way * behind;
                self.paint_at(
                    Look {
                        tint: Some(colour),
                        ..look
                    },
                    at,
                );
            }
        }
        let ground = (!world.strikes_coming.is_empty()).then(|| self.pixels.clone());
        for (_, look, corner) in figures {
            self.paint(look, corner);
        }
        // The hordes' veiled, under the night but in the dark: as far as the light shows.
        if !world.player.build.horde_veil_in_dark() {
            self.paint_veils(world);
        }
        ground.map_or_else(Vec::new, |ground| {
            ground
                .iter()
                .zip(&self.pixels)
                .map(|(under, now)| under != now)
                .collect()
        })
    }

    /// The enemies of a horde veiled in red, as Horde Hunter has it: the very last ones of
    /// their horde pulsing, the others of the share veiled faint and steady. An enemy just hit
    /// stays white.
    pub(super) fn paint_veils(&mut self, world: &World) {
        let pulse = night::share(world.ticks() % VEIL_PULSE, VEIL_PULSE);
        let pulsing = VEIL_MOST * (1.0 - (pulse * std::f32::consts::TAU).cos()) / 2.0;
        for enemy in &world.enemies {
            let look = sprites::enemy(enemy, world.ticks());
            let share = match world.horde_veil(enemy) {
                _ if look.tint.is_some() => continue,
                Some(Veil::Pulsing) => pulsing,
                Some(Veil::Steady) => VEIL_STEADY,
                None => continue,
            };
            let centre = pixel(enemy.position);
            let sprite = self.sprites.get(look.sprite);
            let (left, top) = (
                centre.0 - sprite.width() / 2,
                centre.1 - sprite.height() / 2,
            );
            for (x, y, colour) in sprite.pixels(look.mirrored) {
                self.set(
                    (left + x, top + y),
                    super::super::redden(colour, HUNTER_VEIL, share),
                );
            }
        }
    }
}
