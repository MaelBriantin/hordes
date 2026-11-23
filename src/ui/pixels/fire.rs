//! What burns and heals on the map, and the lights it casts: the campfires, the Molotov's
//! bottles and puddles, the enemies on fire, the gems' glow, a heal.

use ratatui::style::Color;

use super::super::night;
use super::super::sprites::{Look, SpriteId};
use super::{BURNING_LIGHT, Canvas, FLAME_COLOURS, gem_glow, pixel, sway};
use crate::game::{Feature, Puddle, Vec2, World, campfire_light};

/// How far the flamethrower's nozzle sweeps each side, in degrees, and in how many steps (two
/// frames each) it goes from side to side and back.
const SWEEP: f32 = 20.0;
const SWEEP_PERIOD: u32 = 24;

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
/// The wall of fire going out: its last second, embers alone, and their red.
const WALL_EMBER_TICKS: u32 = 30;
const EMBER: Color = Color::Rgb(150, 40, 25);
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
/// The flamethrower's fire, a stream as from a hose: the points drawn along it, a tile's worth;
/// how wide it grows to at its end, in pixels each side; the steps its far end lags behind the
/// nozzle's sweep, bending it; the share of it, from the nozzle, its yellow core runs, and its
/// orange, the rest red and ragged. How far its fire lights, against the stream's length.
const STREAM_POINTS_PER_TILE: f32 = 7.0;
const STREAM_WIDTH: f32 = 1.5;
const STREAM_LAG: f32 = 8.0;
const STREAM_CORE: f32 = 0.6;
const STREAM_ORANGE: f32 = 0.8;
const FLAME_LIGHT: f32 = 0.8;
/// How far apart the points looked at between the nozzle and the fire are, in tiles: a tree or
/// a rock in between hides it.
const FIRE_SIGHT: f32 = 0.25;

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
            if puddle.burst && age < BURST_TICKS {
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
        // The flamethrower's fire lights around its cone, warm.
        if let Some((from, way, range)) = world.flame_cone() {
            let middle = pixel(from + way * (range / 2.0));
            lights.push(night::Light::fire(middle, range * FLAME_LIGHT));
            let (colour, strength, reach) = FIRE_HALO;
            halos.push(halo(middle, (colour, strength * flicker, reach)));
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
            if puddle.wall {
                self.paint_wall_stretch(puddle, frame);
                continue;
            }
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

    /// A stretch of the exterminator's wall of fire: a band of red and orange on the ground,
    /// tongues of fire licking up from it, their tips yellow; its last second, scattered embers.
    fn paint_wall_stretch(&mut self, puddle: &Puddle, frame: u32) {
        let (cx, cy) = pixel(puddle.centre);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a stretch of a tile or two"
        )]
        let reach = (puddle.radius * 2.0).ceil() as i32;
        let going_out = puddle.left <= WALL_EMBER_TICKS;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let distance = f32::from(i16::try_from(dx).unwrap_or(0))
                    .hypot(f32::from(i16::try_from(dy).unwrap_or(0)));
                if distance > puddle.radius * 2.0 {
                    continue;
                }
                let (x, y) = (cx + dx, cy + dy);
                let luck = x.cast_unsigned().wrapping_mul(73_856_093)
                    ^ y.cast_unsigned().wrapping_mul(19_349_663)
                    ^ (frame / 2).wrapping_mul(83_492_791);
                if going_out {
                    if !luck.is_multiple_of(3) {
                        self.set((x, y), EMBER);
                    }
                    continue;
                }
                let [yellow, orange, red] = FLAME_COLOURS;
                self.set((x, y), if luck.is_multiple_of(4) { orange } else { red });
                // A tongue up from it now and then, its tip yellow.
                if luck.is_multiple_of(5) {
                    let height = 1 + i32::try_from(luck % 3).unwrap_or(0);
                    for up in 1..=height {
                        self.set((x, y - up), if up == height { yellow } else { orange });
                    }
                }
            }
        }
    }

    /// The flamethrower's fire while it burns: a stream out of its nozzle, as from a hose,
    /// widening as it goes, a yellow core, orange around, its end red and ragged; the nozzle
    /// sweeping from side to side, its far end lagging, the stream bends. Not behind a tree or a
    /// rock.
    pub(super) fn paint_flamethrower(&mut self, world: &World, frame: u32) {
        let Some((from, way, range)) = world.flame_cone() else {
            return;
        };
        let step = frame / 2;
        let side = Vec2::new(-way.y, way.x);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few dozen points"
        )]
        let points = (range * STREAM_POINTS_PER_TILE).ceil() as u32;
        for point in 0..points {
            let share = night::share(point, points);
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a few steps"
            )]
            let lag = (share * STREAM_LAG) as u32;
            let swept = step.saturating_sub(lag) % SWEEP_PERIOD;
            let angle = (std::f32::consts::TAU * night::share(swept, SWEEP_PERIOD)).sin()
                * SWEEP.to_radians();
            let line = way * angle.cos() + side * angle.sin();
            let across = Vec2::new(-line.y, line.x);
            let middle = from + line * (share * range);
            let width = 0.25 + share * STREAM_WIDTH;
            // Half a pixel apart across it, as far as it is wide there.
            for half_pixels in -3_i8..=3 {
                let off = f32::from(half_pixels) * 0.5;
                if off.abs() > width {
                    continue;
                }
                if share > 0.75 {
                    let luck = point.wrapping_mul(73_856_093)
                        ^ u32::from(half_pixels.cast_unsigned()).wrapping_mul(19_349_663)
                        ^ frame.wrapping_mul(83_492_791);
                    if luck.is_multiple_of(3) {
                        continue;
                    }
                }
                let at = middle + across * (off / 2.0);
                if !Self::in_flame_sight(world, from, at) {
                    continue;
                }
                let core = off.abs() < width * 0.35;
                let colour = FLAME_COLOURS[if core && share < STREAM_CORE {
                    0
                } else if share < STREAM_ORANGE {
                    1
                } else {
                    2
                }];
                self.set(Self::from_player(world, at), colour);
            }
        }
    }

    /// The pixel of a point near the player, counted from the pixel the player is drawn
    /// around: the same pixels on either side of them, wherever they stand in their tile.
    fn from_player(world: &World, at: Vec2) -> (i32, i32) {
        let (x, y) = pixel(world.player.position);
        let apart = (at - world.player.position) * 2.0;
        #[allow(clippy::cast_possible_truncation, reason = "a few pixels")]
        (x + apart.x.floor() as i32, y + apart.y.floor() as i32)
    }

    /// Whether the fire reaches `to` from `from`: no tree nor rock in between.
    fn in_flame_sight(world: &World, from: Vec2, to: Vec2) -> bool {
        let along = to - from;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the cone, a few tiles"
        )]
        let steps = (along.length() / FIRE_SIGHT).ceil() as u32;
        (1..=steps).all(|step| {
            let point = from + along * night::share(step, steps.max(1));
            !world.map.stops_shots(point.tile())
        })
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

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::super::super::Sprites;
    use super::*;
    use crate::game::{Choice, Facing, Weapon};

    #[test]
    fn the_flamethrower_s_fire_never_covers_the_player_whichever_way_they_face() {
        let sprites = Sprites::load().unwrap();
        // Anywhere in their tile: the middle, and near either edge.
        let spots = [0.5, 0.26, 0.74, 0.95, 0.05].map(|share| Vec2::new(10.0 + share, 10.5));
        for (facing, at) in [Facing::Left, Facing::Right]
            .into_iter()
            .flat_map(|facing| spots.map(|at| (facing, at)))
        {
            let mut world = World::new(1);
            world
                .player
                .build
                .take(Choice::Weapon(Weapon::Flamethrower));
            world.player.facing = facing;
            world.player.position = at;
            world.flaming = true;
            let (x, y) = pixel(world.player.position);
            for frame in 0..48 {
                let mut canvas = Canvas::new(&sprites, Rect::new(0, 0, 60, 30), (x, y));
                canvas.paint_flamethrower(&world, frame);
                // The hero, seen from the side: the two columns of the middle of their sprite.
                for dy in -2..=1 {
                    for dx in -1..=0 {
                        let index = canvas.index((x + dx, y + dy)).unwrap();
                        assert_eq!(
                            canvas.pixels[index], None,
                            "{facing:?} at {at:?}, frame {frame}"
                        );
                    }
                }
            }
        }
    }
}
