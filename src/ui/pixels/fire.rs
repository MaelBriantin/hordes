//! What burns and heals on the map, and the lights it casts: the campfires, the Molotov's
//! bottles and puddles, the enemies on fire, the gems' glow, a heal.

use ratatui::style::Color;

use super::super::night;
use super::super::sprites::{Look, SpriteId};
use super::{BURNING_LIGHT, Canvas, FLAME_COLOURS, gem_glow, pixel, sway};
use crate::game::{Feature, Vec2, World, campfire_light};

/// How far off the canvas a gem still lights it, in pixels: its halo's reach.
const GEM_HALO_MARGIN: i32 = 4;
/// How much a campfire's light sways, as a share of its reach.
const FIRE_SWAY: f32 = 0.12;
/// The frames a campfire's flame keeps one look before the other.
const FLICKER: u32 = 6;
/// A Molotov's puddle: how far it lights against its own radius; a bottle: how high its arc
/// rises, in tiles, and its green.
const PUDDLE_LIGHT: f32 = 2.25;
const BOTTLE_ARC: f32 = 1.5;
const BOTTLE: Color = Color::Rgb(70, 160, 80);
/// What burns, its halo: its colour, how strong at its middle, how far it reaches in pixels.
/// Fire, a puddle's, a campfire's or an enemy's, and a bottle bursting into flames.
const FIRE_HALO: ((u8, u8, u8), f32, i32) = ((245, 130, 40), 0.35, 10);
const BURNING_HALO: ((u8, u8, u8), f32, i32) = ((245, 130, 40), 0.28, 8);
const BURST_HALO: ((u8, u8, u8), f32, i32) = ((255, 170, 60), 0.56, 16);
/// How dim a fire's halo gets as it flickers, every other look of its flame.
const FIRE_FLICKER: f32 = 0.85;
/// The ticks a bottle's burst lights, before its puddle's own light alone.
const BURST_TICKS: u32 = 8;
/// A heal around the player: how long its crosses rise, in ticks; where they start from the
/// player's middle, in pixels; their green, then darker.
const HEAL_TICKS: u32 = 18;
const HEAL_CROSSES: [(i32, i32); 3] = [(-5, 1), (5, -1), (0, -5)];
const HEAL_BRIGHT: Color = Color::Rgb(110, 235, 120);
const HEAL_DIM: Color = Color::Rgb(50, 140, 60);

impl Canvas<'_> {
    /// What burns besides the campfires: the bottles bursting, the puddles of fire and the
    /// enemies burning; their lights, and the halos of all that burns, the `campfires` lit
    /// included.
    pub(super) fn shines(
        world: &World,
        campfires: &[night::Light],
        frame: u32,
    ) -> (Vec<night::Light>, Vec<night::Halo>) {
        let halo =
            |at: (i32, i32), (colour, strength, reach): ((u8, u8, u8), f32, i32)| night::Halo {
                centre: at,
                colour,
                strength,
                reach,
            };
        let flicker = if (frame / FLICKER).is_multiple_of(2) {
            1.0
        } else {
            FIRE_FLICKER
        };
        let mut lights = Vec::new();
        let mut halos = Vec::new();
        for puddle in &world.puddles {
            let centre = pixel(puddle.centre);
            let age = puddle.age();
            if age < BURST_TICKS {
                let left = 1.0 - night::share(age, BURST_TICKS);
                lights.push(night::Light::burst(centre, left));
                let (colour, strength, reach) = BURST_HALO;
                halos.push(halo(centre, (colour, strength * left, reach)));
            }
            // It lights around it, warm.
            lights.push(night::Light::fire(centre, puddle.radius * PUDDLE_LIGHT));
            let (colour, strength, reach) = FIRE_HALO;
            halos.push(halo(centre, (colour, strength * flicker, reach)));
        }
        for light in campfires {
            let (colour, strength, reach) = FIRE_HALO;
            halos.push(halo(light.centre, (colour, strength * flicker, reach)));
        }
        // An enemy on fire lights a little around it, warm.
        for enemy in world.enemies.iter().filter(|enemy| enemy.burning > 0) {
            let (x, y) = pixel(enemy.position);
            lights.push(night::Light::fire((x, y), BURNING_LIGHT));
            halos.push(halo((x, y - 2), BURNING_HALO));
        }
        (lights, halos)
    }

    /// Small green crosses rising around the player at `centre`, `since` ticks after they were
    /// healed, darker as they go: healed, it shows.
    pub(super) fn paint_heal(&mut self, centre: (i32, i32), since: u32) {
        if since >= HEAL_TICKS {
            return;
        }
        let colour = if since < HEAL_TICKS / 2 {
            HEAL_BRIGHT
        } else {
            HEAL_DIM
        };
        let rise = i32::try_from(since / 3).unwrap_or(0);
        for (dx, dy) in HEAL_CROSSES {
            let (x, y) = (centre.0 + dx, centre.1 + dy - rise);
            for (px, py) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                self.set((x + px, y + py), colour);
            }
        }
    }

    /// The gems the canvas shows, or nearly, each with its glow's colour.
    pub(super) fn glows(&self, world: &World) -> Vec<night::Glow> {
        world
            .gems
            .iter()
            .map(|gem| (pixel(gem.position), gem_glow(gem.value)))
            .filter(|&(at, _)| self.near(at, GEM_HALO_MARGIN))
            .collect()
    }

    /// The Molotov's puddles of fire, flames flickering over them, and its bottles in the air,
    /// arcing to where they burst.
    pub(super) fn paint_molotovs(&mut self, world: &World, frame: u32) {
        for puddle in &world.puddles {
            let (cx, cy) = pixel(puddle.centre);
            #[allow(clippy::cast_possible_truncation, reason = "a puddle of a few tiles")]
            let reach = (puddle.radius * 2.0).ceil() as i32;
            for dy in -reach..=reach {
                for dx in -reach..=reach {
                    let distance = f32::from(i16::try_from(dx).unwrap_or(0))
                        .hypot(f32::from(i16::try_from(dy).unwrap_or(0)));
                    if distance > puddle.radius * 2.0 {
                        continue;
                    }
                    // Each pixel its own flicker: some flames lit, others not, each frame.
                    let luck = (cx + dx).cast_unsigned().wrapping_mul(73_856_093)
                        ^ (cy + dy).cast_unsigned().wrapping_mul(19_349_663)
                        ^ (frame / 3).wrapping_mul(83_492_791);
                    if luck.is_multiple_of(3) {
                        continue;
                    }
                    let shade = usize::try_from(luck % 3).unwrap_or(0);
                    self.set((cx + dx, cy + dy), FLAME_COLOURS[shade]);
                }
            }
        }
        for bottle in &world.bottles {
            let progress = bottle.progress();
            let along = bottle.from + (bottle.to - bottle.from) * progress;
            // An arc: up, then down to where it bursts.
            let lift = 4.0 * progress * (1.0 - progress) * BOTTLE_ARC;
            let (x, y) = pixel(along - Vec2::new(0.0, lift));
            self.set((x, y), BOTTLE);
            self.set(
                (x, y - 1),
                FLAME_COLOURS[usize::try_from(frame % 2).unwrap_or(0)],
            );
        }
    }

    /// The healing on the ground the canvas shows, before the night falls on it: herbs, which
    /// only the player's torch or a strike of lightning shows, campfires, their flame flickering,
    /// or their logs once out. The lights of the campfires still burning.
    pub(super) fn paint_healing(&mut self, world: &World, frame: u32) -> Vec<night::Light> {
        let burning = if (frame / FLICKER).is_multiple_of(2) {
            SpriteId::Campfire1
        } else {
            SpriteId::Campfire2
        };
        let mut lights = Vec::new();
        for (tile, feature) in self.healing(world) {
            let sprite = match feature {
                Feature::Herb => SpriteId::Herb,
                Feature::Campfire => {
                    let left = world.campfire_left(tile);
                    if left == 0 {
                        SpriteId::Ashes
                    } else {
                        let centre = pixel(Vec2::middle_of(tile));
                        let radius = campfire_light(left) * (1.0 + FIRE_SWAY * sway(frame, tile));
                        lights.push(night::Light::fire(centre, radius));
                        burning
                    }
                }
            };
            self.paint_at(Look::plain(sprite), Vec2::middle_of(tile));
        }
        lights
    }

    /// The healing on the ground within the canvas, and a little around it: a sprite may stick
    /// out of its tile.
    pub(super) fn healing(&self, world: &World) -> Vec<((i32, i32), Feature)> {
        let (columns, rows) = self.tiles();
        world.features_in(
            (columns.start - 2, rows.start - 2),
            (columns.end + 1, rows.end + 1),
        )
    }
}
