//! The map in pixels: a tile is 2 × 2 pixels, a cell holds 2 pixels one above the other. The
//! scene is painted in pixels first, then each cell shows its two as a half block.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::ViewState;
use super::level_up;
use super::night;
use super::sprites::{self, Look, Skin, SpriteId, Sprites};
use crate::game::{
    Bullet, Choice, EnemyKind, Feature, MAX_LEVEL, OBSTACLE_SIZE, Obstacle, Passive, STRIKE_RADIUS,
    SWING_TICKS, Stain, Swing, Vec2, Weapon, World, campfire_light,
};

/// The Shadows' eyes, the red of the sprites' palette, and the darker red they shine with in the
/// dark.
const SHADOW_EYES: Color = Color::Rgb(230, 40, 40);
const SHADOW_EYES_GLOW: Color = Color::Rgb(160, 22, 26);
/// How strong the halo around a Shadow's eyes, against a gem's.
const EYES_GLOW_STRENGTH: f32 = 0.3;
/// The silhouettes a dashing player leaves behind: how far behind, in tiles, and their colour,
/// fading.
const AFTERIMAGES: [(f32, Color); 2] = [
    (1.6, Color::Rgb(40, 44, 62)),
    (0.8, Color::Rgb(70, 78, 108)),
];
/// The glow of a gem, by what it is worth, as its colour.
fn gem_glow(value: u32) -> (u8, u8, u8) {
    match value {
        0 | 1 => (150, 180, 230),
        2..=4 => (60, 130, 255),
        _ => (245, 190, 60),
    }
}
/// How far off the canvas a gem still lights it, in pixels: its halo's reach.
const GEM_HALO_MARGIN: i32 = 4;
/// Pixels per tile, each way.
const TILE_PIXELS: i32 = 2;
/// How much a campfire's light sways, as a share of its reach.
const FIRE_SWAY: f32 = 0.12;

/// A campfire's light swaying, `frame` frames in, from -1 to 1: two waves of its own, one slow,
/// one quick, never quite the same as another fire's.
fn sway(frame: u32, (x, y): (i32, i32)) -> f32 {
    let seed = f32::from(u16::try_from((x * 31 + y * 17).rem_euclid(1_000)).unwrap_or(0));
    let time = f32::from(u16::try_from(frame % 10_000).unwrap_or(0));
    (time * 0.21 + seed).sin() * 0.6 + (time * 0.67 + seed * 1.7).sin() * 0.4
}

/// The frames a campfire's flame keeps one look before the other.
const FLICKER: u32 = 6;
/// The compass's arrow: on the red of the herbs, and a darker red as it pulses, every so many
/// frames.
const COMPASS: Color = Color::Rgb(220, 45, 65);
const COMPASS_DIM: Color = Color::Rgb(130, 25, 38);
const COMPASS_PULSE: u32 = 10;
/// The ground: dark grass, and the earth showing through it, out of 256 pixels.
const GRASS: Color = Color::Rgb(32, 46, 28);
const EARTH: Color = Color::Rgb(44, 38, 28);
const EARTH_SHARE: u32 = 90;
/// The arrow over the player's head, pointing down at them, as wide as they are; the pixels
/// between it and their head, and the ticks it spends up, then down, bouncing.
const ARROW: [&str; 2] = ["####", ".##."];
const ARROW_GAP: i32 = 2;
const ARROW_BOUNCE: u32 = 15;
/// The small HP bar over the player's head: its width, the pixels between it and the head, or
/// the arrow when it shows.
const MINI_BAR: i32 = 6;
const MINI_BAR_GAP: i32 = 1;
/// The arrow over the player's head, to see them at once in the crowd, in its colour: a setting,
/// off at first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrow {
    Off,
    White,
    Yellow,
    Red,
}

impl Arrow {
    /// Every choice, in the order the settings go through them.
    const ALL: [Arrow; 4] = [Arrow::Off, Arrow::White, Arrow::Yellow, Arrow::Red];

    /// The next choice, round and round.
    pub fn next(self) -> Self {
        self.step(1)
    }

    /// The choice before, round and round.
    pub fn previous(self) -> Self {
        self.step(Self::ALL.len() - 1)
    }

    fn step(self, by: usize) -> Self {
        let index = Self::ALL.iter().position(|look| *look == self).unwrap_or(0);
        Self::ALL[(index + by) % Self::ALL.len()]
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Arrow::Off => "off",
            Arrow::White => "white",
            Arrow::Yellow => "yellow",
            Arrow::Red => "red",
        }
    }

    /// The arrow's colour: white when off, for the arrow of a ping.
    fn colour(self) -> Color {
        match self {
            Arrow::Off | Arrow::White => Color::Rgb(240, 240, 245),
            Arrow::Yellow => Color::Rgb(255, 225, 90),
            Arrow::Red => Color::Rgb(235, 45, 45),
        }
    }
}

/// The gear over the top left of the map: the share of the light its veil leaves, the room
/// between two icons and around them, in pixels; the levels written on the icons, on their
/// badge, MAX in gold; the rounds under the icons.
const VEIL: f32 = 0.55;
const GEAR_GAP: i32 = 2;
/// The rows of veil above the icons and under their gauges, in pixels.
const GEAR_PAD: i32 = 1;
const GEAR_LEVEL: Color = Color::Rgb(235, 235, 240);
const GEAR_ROUNDS: Color = Color::Rgb(235, 210, 120);
const GEAR_RELOADING: Color = Color::Rgb(165, 165, 172);
const GEAR_EMPTY: Color = Color::Rgb(70, 70, 80);
const GEAR_MAX: Color = Color::Rgb(255, 200, 60);
const GEAR_BADGE: Color = Color::Rgb(35, 35, 45);
/// The gear as a list: the names' colour, and the gauges' width, in cells.
const GEAR_NAME: Color = Color::Rgb(200, 200, 210);
const LIST_GAUGE: i32 = 6;
/// A Molotov's puddle: how far it lights against its own radius; a bottle: how high its arc
/// rises, in tiles, and its green.
const PUDDLE_LIGHT: f32 = 2.25;
const BOTTLE_ARC: f32 = 1.5;
const BOTTLE: Color = Color::Rgb(70, 160, 80);
/// What shines, its halo: its colour, how strong at its middle, how far it reaches in pixels.
/// Fire, a puddle's, a campfire's or an enemy's, and a bottle bursting into flames; a gun's
/// flash at its mouth.
const FIRE_HALO: ((u8, u8, u8), f32, i32) = ((245, 130, 40), 0.35, 10);
const BURNING_HALO: ((u8, u8, u8), f32, i32) = ((245, 130, 40), 0.28, 8);
const BURST_HALO: ((u8, u8, u8), f32, i32) = ((255, 170, 60), 0.56, 16);
const MUZZLE_HALO: ((u8, u8, u8), f32, i32) = ((255, 210, 130), 0.42, 8);
/// How dim a fire's halo gets as it flickers, every other look of its flame.
const FIRE_FLICKER: f32 = 0.85;
/// The ticks a gun's flash lasts, a shot just fired; and how far ahead of where it was shot
/// from, in tiles: the gun's mouth.
const MUZZLE_TICKS: u8 = 2;
const MUZZLE_AHEAD: f32 = 1.0;
/// A gun's flash: its heart, the pixel ahead of it, the pixels on its sides.
const MUZZLE_FLASH: [Color; 3] = [
    Color::Rgb(255, 250, 220),
    Color::Rgb(255, 220, 140),
    Color::Rgb(255, 200, 110),
];
/// The ticks a bottle's burst lights, before its puddle's own light alone.
const BURST_TICKS: u32 = 8;
/// An enemy on fire: how far its flames light, in tiles; where they flicker, from the middle of
/// the top of its sprite, in pixels; their colours.
const BURNING_LIGHT: f32 = 1.5;
const FLAMES: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, -1), (0, -2)];
const FLAME_COLOURS: [Color; 3] = [
    Color::Rgb(255, 210, 80),
    Color::Rgb(245, 130, 40),
    Color::Rgb(220, 60, 30),
];
/// A heal around the player: how long its crosses rise, in ticks; where they start from the
/// player's middle, in pixels; their green, then darker.
const HEAL_TICKS: u32 = 18;
const HEAL_CROSSES: [(i32, i32); 3] = [(-5, 1), (5, -1), (0, -5)];
const HEAL_BRIGHT: Color = Color::Rgb(110, 235, 120);
const HEAL_DIM: Color = Color::Rgb(50, 140, 60);
/// The crowbar's blow, iron grey, then as it fades.
const CROWBAR_BLOW: (Color, Color) = (Color::Rgb(160, 165, 180), Color::Rgb(90, 92, 105));
/// A shotgun's pellets, every other one: the gold and the red of its shells.
const PELLETS: (Color, Color) = (Color::Rgb(240, 190, 60), Color::Rgb(220, 50, 50));
/// A machine gun's tracer, from its head to the end of its trail.
const TRACER: [Color; 3] = [
    Color::Rgb(255, 225, 160),
    Color::Rgb(230, 140, 60),
    Color::Rgb(130, 70, 30),
];
/// The colour of a broken drone, repairing.
const BROKEN: Color = Color::Rgb(75, 75, 82);
/// The colour of a machete's swing, then as it fades.
const SLASH: Color = Color::Rgb(235, 235, 245);
const SLASH_FADING: Color = Color::Rgb(130, 130, 145);
/// Where a slash shows, as shares of its reach: its inner and outer edge.
const SLASH_RADII: [f32; 2] = [0.76, 0.92];

/// A canvas of pixels the size of an area, its top-left pixel at `origin` on the map.
pub struct Canvas<'a> {
    sprites: &'a Sprites,
    pub origin: (i32, i32),
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<Option<Color>>,
}

impl<'a> Canvas<'a> {
    pub fn new(sprites: &'a Sprites, area: Rect, centre: (i32, i32)) -> Self {
        let width = i32::from(area.width);
        let height = i32::from(area.height) * 2;
        Self {
            sprites,
            origin: (centre.0 - width / 2, centre.1 - height / 2),
            width,
            height,
            pixels: vec![None; usize::from(area.width) * usize::from(area.height) * 2],
        }
    }

    /// The canvas index of a pixel of the map, if the canvas shows it.
    pub fn index(&self, (x, y): (i32, i32)) -> Option<usize> {
        let (x, y) = (x - self.origin.0, y - self.origin.1);
        let inside = (0..self.width).contains(&x) && (0..self.height).contains(&y);
        inside.then(|| usize::try_from(y * self.width + x).ok())?
    }

    /// Whether a pixel of the map is on the canvas, or within `margin` pixels of it.
    fn near(&self, (x, y): (i32, i32), margin: i32) -> bool {
        let (x, y) = (x - self.origin.0, y - self.origin.1);
        (-margin..self.width + margin).contains(&x) && (-margin..self.height + margin).contains(&y)
    }

    /// The tiles the canvas shows, even partly, as two ranges: across, then down.
    fn tiles(&self) -> (std::ops::Range<i32>, std::ops::Range<i32>) {
        let first = |pixel: i32| pixel.div_euclid(TILE_PIXELS);
        let past = |pixel: i32| (pixel - 1).div_euclid(TILE_PIXELS) + 1;
        (
            first(self.origin.0)..past(self.origin.0 + self.width),
            first(self.origin.1)..past(self.origin.1 + self.height),
        )
    }

    /// A pixel of the map in a colour, if the canvas shows it.
    pub fn set(&mut self, pixel: (i32, i32), color: Color) {
        if let Some(index) = self.index(pixel) {
            self.pixels[index] = Some(color);
        }
    }

    /// The player picked out of the crowd as `view` says: an arrow over their head, always or a
    /// moment once pinged; and their HP in a small bar over it all, if it shows.
    fn mark(&mut self, world: &World, view: ViewState) {
        let player = &world.player;
        let look = sprites::hero(player.facing, view.skin);
        let arrow = view.arrow != Arrow::Off || view.pinged;
        if arrow {
            self.arrow(look, player.position, world.ticks(), view.arrow.colour());
        }
        if view.mini_bars {
            self.mini_bars(world, look, arrow);
        }
    }

    /// The player's HP in a small bar over the head of their sprite as `paint_at` would place
    /// it; over the arrow if it shows, out of its way.
    fn mini_bars(&mut self, world: &World, look: Look, over_arrow: bool) {
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
        self.mini_bar((left, row), MINI_BAR, hp, super::HP_COLOR);
    }

    /// A small bar of `width` pixels from `start`, `filled` of it (from 0 to 1) in `colour`, the
    /// rest faded.
    fn mini_bar(&mut self, start: (i32, i32), width: i32, filled: f32, colour: Color) {
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
                super::faded(colour, super::BAR_EMPTY)
            };
            self.set((start.0 + x, start.1), shade);
        }
    }

    /// An arrow over the head of a sprite as `paint_at` would place it, pointing down at it,
    /// bouncing a pixel up and down, `now` the run's tick, in `colour`.
    fn arrow(&mut self, look: Look, position: Vec2, now: u32, colour: Color) {
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

    /// A bullet: a pistol's, a white pixel; a machine gun's, a thin tracer, its trail fading
    /// behind it.
    fn paint_bullet(&mut self, bullet: &Bullet) {
        if bullet.weapon == Weapon::Pistol {
            self.paint_at(Look::plain(SpriteId::Bullet), bullet.position);
            return;
        }
        if bullet.weapon == Weapon::Shotgun {
            let colour = if bullet.other { PELLETS.1 } else { PELLETS.0 };
            self.set(pixel(bullet.position), colour);
            return;
        }
        let way = bullet.way();
        for (step, colour) in (0..).zip(TRACER) {
            let behind = bullet.position - way * (0.5 * f32::from(u8::try_from(step).unwrap_or(0)));
            self.set(pixel(behind), colour);
        }
    }

    /// The guns' flashes the canvas shows: where each bullet just shot was shot from, a tile
    /// ahead, its way. Pellets of a shot share one.
    fn muzzles(world: &World) -> Vec<(Vec2, Vec2)> {
        let mut muzzles: Vec<(Vec2, Vec2)> = Vec::new();
        for bullet in &world.bullets {
            if let Some(from) = bullet.shot_from(MUZZLE_TICKS) {
                let at = from + bullet.way() * MUZZLE_AHEAD;
                if !muzzles.iter().any(|&(other, _)| pixel(other) == pixel(at)) {
                    muzzles.push((at, bullet.way()));
                }
            }
        }
        muzzles
    }

    /// What shines besides the torch and the campfires: the guns' flashes, the bottles bursting,
    /// the puddles of fire and the enemies burning; their lights, and the halos of all that
    /// burns or flashes, the `campfires` lit included.
    fn shines(
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
        for (at, _) in Self::muzzles(world) {
            lights.push(night::Light::muzzle(pixel(at)));
            halos.push(halo(pixel(at), MUZZLE_HALO));
        }
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

    /// The guns' flashes, at their mouths: a bright heart, a pixel ahead, a pixel each side.
    fn paint_muzzles(&mut self, world: &World) {
        for (at, way) in Self::muzzles(world) {
            let side = Vec2::new(-way.y, way.x) * 0.5;
            self.set(pixel(at), MUZZLE_FLASH[0]);
            self.set(pixel(at + way * 0.5), MUZZLE_FLASH[1]);
            self.set(pixel(at + side), MUZZLE_FLASH[2]);
            self.set(pixel(at - side), MUZZLE_FLASH[2]);
        }
    }

    /// Small green crosses rising around the player at `centre`, `since` ticks after they were
    /// healed, darker as they go: healed, it shows.
    fn paint_heal(&mut self, centre: (i32, i32), since: u32) {
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
    fn glows(&self, world: &World) -> Vec<night::Glow> {
        world
            .gems
            .iter()
            .map(|gem| (pixel(gem.position), gem_glow(gem.value)))
            .filter(|&(at, _)| self.near(at, GEM_HALO_MARGIN))
            .collect()
    }

    /// Once the night has fallen, the Shadows' eyes the canvas shows, or nearly, each with its
    /// halo's colour.
    fn eye_glows(&self, world: &World) -> Vec<night::Glow> {
        let Color::Rgb(red, green, blue) = SHADOW_EYES else {
            return Vec::new();
        };
        if world.torch_on().is_none() {
            return Vec::new();
        }
        world
            .enemies
            .iter()
            .filter(|enemy| enemy.kind == EnemyKind::Shadow)
            .map(|enemy| {
                let (x, y) = pixel(enemy.position);
                ((x, y - 1), (red, green, blue))
            })
            .filter(|&(at, _)| self.near(at, GEM_HALO_MARGIN))
            .collect()
    }

    /// The Molotov's puddles of fire, flames flickering over them, and its bottles in the air,
    /// arcing to where they burst.
    fn paint_molotovs(&mut self, world: &World, frame: u32) {
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

    /// Flames over each enemy on fire: a few pixels above its head, flickering.
    fn paint_flames(&mut self, world: &World, frame: u32) {
        for enemy in world.enemies.iter().filter(|enemy| enemy.burning > 0) {
            let look = sprites::enemy(enemy);
            let height = self.sprites.get(look.sprite).height();
            let (x, y) = pixel(enemy.position);
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

    /// At night, the Shadows' eyes, shining red in the dark.
    fn paint_shadow_eyes(&mut self, world: &World) {
        if world.torch_on().is_none() {
            return;
        }
        for enemy in world
            .enemies
            .iter()
            .filter(|enemy| enemy.kind == EnemyKind::Shadow)
        {
            self.repaint(
                sprites::enemy(enemy),
                enemy.position,
                SHADOW_EYES,
                SHADOW_EYES_GLOW,
            );
        }
    }

    /// Only the pixels of a sprite in one `colour`, placed as `paint_at` would, in another one,
    /// `shown`: its eyes, say, shining in the dark.
    fn repaint(&mut self, look: Look, position: Vec2, colour: Color, shown: Color) {
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

    /// The player and the enemies, the lowest on the map last: in front of those above them.
    /// While lightning is about to strike, which pixels they cover, for its ring to show on the
    /// ground under them; none otherwise.
    fn paint_figures(&mut self, world: &World, skin: Skin) -> Vec<bool> {
        let player = &world.player;
        let mut figures: Vec<(Vec2, Look)> = world
            .enemies
            .iter()
            .map(|enemy| (enemy.position, sprites::enemy(enemy)))
            .collect();
        // Blinking while nothing can hurt them.
        let blinks_out = player.invincible > 0 && (world.ticks() / 3).is_multiple_of(2);
        if player.dash.is_some() || !blinks_out {
            figures.push((player.position, sprites::hero(player.facing, skin)));
        }
        figures.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
        // Dashing, the player leaves fading silhouettes of themselves behind.
        if let Some(dash) = player.dash {
            let look = sprites::hero(player.facing, skin);
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
        for (position, look) in figures {
            self.paint_at(look, position);
        }
        ground.map_or_else(Vec::new, |ground| {
            ground
                .iter()
                .zip(&self.pixels)
                .map(|(under, now)| under != now)
                .collect()
        })
    }

    /// A sprite with its top-left pixel at a pixel of the map.
    fn paint(&mut self, look: Look, (left, top): (i32, i32)) {
        let sprite = self.sprites.get(look.sprite);
        for (x, y, color) in sprite.pixels(look.mirrored) {
            if let Some(index) = self.index((left + x, top + y)) {
                self.pixels[index] = Some(look.tint.unwrap_or(color));
            }
        }
    }

    /// A dark veil over the top left of what the canvas shows, `width` × `height` pixels: under
    /// the gear.
    fn veil(&mut self, width: i32, height: i32) {
        let (left, top) = self.origin;
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                if let Some(index) = self.index((left + x, top + y))
                    && let Some(Color::Rgb(red, green, blue)) = self.pixels[index]
                {
                    let dim = |value: u8| {
                        #[allow(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "a share of a colour, between 0 and 255"
                        )]
                        let dimmed = (f32::from(value) * VEIL).round() as u8;
                        dimmed
                    };
                    self.pixels[index] = Some(Color::Rgb(dim(red), dim(green), dim(blue)));
                }
            }
        }
    }

    /// The player's gear as icons, over the top left of what the canvas shows, on a dark veil:
    /// each card's icon, as on the level-up cards, its gauge right under it if it has one. Where
    /// each one's level goes, written over the icon's bottom right corner once the canvas is
    /// drawn.
    fn paint_gear(&mut self, gear: &[GearSlot]) -> Vec<GearLevel> {
        if gear.is_empty() {
            return Vec::new();
        }
        let size = self.sprites.get(SpriteId::IconMachete).width();
        let step = size + GEAR_GAP;
        let width = GEAR_GAP + step * i32::try_from(gear.len()).unwrap_or(0);
        // A row, the icon, its gauge, a row.
        self.veil(width, GEAR_PAD + size + 1 + GEAR_PAD);
        let (left, top) = self.origin;
        let mut levels = Vec::new();
        for (slot, card) in (0..).zip(gear) {
            let x = left + GEAR_GAP + step * slot;
            let y = top + GEAR_PAD;
            self.paint(Look::plain(level_up::icon(card.choice)), (x, y));
            if let Some((part, whole, colour)) = card.gauge {
                let lit = share_of(part, whole, size);
                for pixel in 0..size {
                    self.set(
                        (x + pixel, y + size),
                        if pixel < lit { colour } else { GEAR_EMPTY },
                    );
                }
            }
            if let Some(level) = card.level {
                levels.push(GearLevel {
                    right: x - left + size,
                    // The cell over the icon's last two rows but one: the gauge left clear.
                    row: y - top + size - 2,
                    level,
                });
            }
        }
        levels
    }

    /// A machete's swing around `centre`: an arc of pixels, sweeping from one end to the other
    /// in the first half of its time, fading in the second.
    fn paint_swing(&mut self, swing: &Swing, centre: Vec2) {
        let shown = SWING_TICKS - swing.left;
        let sweeping = shown < SWING_TICKS / 2;
        // The crowbar's blow, iron grey; the machete's slash, a blade's white.
        let (bright, fading) = if swing.weapon == Weapon::Crowbar {
            CROWBAR_BLOW
        } else {
            (SLASH, SLASH_FADING)
        };
        let color = if sweeping { bright } else { fading };
        let reach = if sweeping {
            f32::from(u8::try_from(shown + 1).unwrap_or(1))
                / f32::from(u8::try_from(SWING_TICKS / 2).unwrap_or(1))
        } else {
            1.0
        };
        let way = swing.facing.way();
        let start = way.y.atan2(way.x) - swing.half_arc;
        let span = 2.0 * swing.half_arc * reach;
        for step in 0..=SLASH_STEPS {
            let angle = start + span * f32::from(step) / f32::from(SLASH_STEPS);
            for share in SLASH_RADII {
                let radius = swing.reach * share;
                let point = centre + Vec2::new(angle.cos(), angle.sin()) * radius;
                if let Some(index) = self.index(pixel(point)) {
                    self.pixels[index] = Some(color);
                }
            }
        }
    }

    /// The healing on the ground the canvas shows, before the night falls on it: herbs, which
    /// only the player's torch or a strike of lightning shows, campfires, their flame flickering,
    /// or their logs once out. The lights of the campfires still burning.
    fn paint_healing(&mut self, world: &World, frame: u32) -> Vec<night::Light> {
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
    fn healing(&self, world: &World) -> Vec<((i32, i32), Feature)> {
        let (columns, rows) = self.tiles();
        world.features_in(
            (columns.start - 2, rows.start - 2),
            (columns.end + 1, rows.end + 1),
        )
    }

    /// A sprite centred on a point of the map.
    fn paint_at(&mut self, look: Look, position: Vec2) {
        let centre = pixel(position);
        let sprite = self.sprites.get(look.sprite);
        self.paint(
            look,
            (
                centre.0 - sprite.width() / 2,
                centre.1 - sprite.height() / 2,
            ),
        );
    }

    /// The bare ground under everything: grass, speckled with earth.
    fn paint_ground(&mut self) {
        let (left, top) = self.origin;
        let width = self.width;
        for (index, pixel) in (0..).zip(self.pixels.iter_mut()) {
            *pixel = Some(ground((left + index % width, top + index / width)));
        }
    }

    /// The ground: grass and pebbles scattered on it, always the same, so it is seen to move
    /// under the player, on the screen's dark or, if `coloured`, on grass speckled with earth.
    /// And the obstacles standing on it.
    fn paint_map(&mut self, world: &World, coloured: bool) {
        if coloured {
            self.paint_ground();
        }
        let (columns, rows) = self.tiles();
        for y in rows {
            for x in columns.clone() {
                let corner = (x * TILE_PIXELS, y * TILE_PIXELS);
                let starts_obstacle =
                    x.rem_euclid(OBSTACLE_SIZE) == 0 && y.rem_euclid(OBSTACLE_SIZE) == 0;
                match world.map.obstacle_at((x, y)) {
                    Some(obstacle) if starts_obstacle => {
                        let sprite = match obstacle {
                            Obstacle::Tree => SpriteId::Tree,
                            Obstacle::Rock => SpriteId::Rock,
                        };
                        self.paint(Look::plain(sprite), corner);
                    }
                    Some(_) => {}
                    None => {
                        if let Some(decor) = decor((x, y)) {
                            self.paint(Look::plain(decor), corner);
                        }
                    }
                }
            }
        }
    }

    /// Each cell as a half block: its top pixel in the foreground of `▀`, its bottom one in the
    /// background; an empty half keeps the terminal's background.
    pub fn draw(&self, buffer: &mut Buffer, area: Rect) {
        let width = usize::from(area.width);
        for row in 0..area.height {
            for column in 0..area.width {
                let top_index = usize::from(row) * 2 * width + usize::from(column);
                let top = self.pixels[top_index];
                let bottom = self.pixels[top_index + width];
                let (symbol, style) = match (top, bottom) {
                    (Some(top), Some(bottom)) => ("▀", Style::new().fg(top).bg(bottom)),
                    (Some(top), None) => ("▀", Style::new().fg(top)),
                    (None, Some(bottom)) => ("▄", Style::new().fg(bottom)),
                    (None, None) => continue,
                };
                buffer.set_string(area.x + column, area.y + row, symbol, style);
            }
        }
    }
}

/// The map around the player, the player in the middle of `area`, at night: lit around the
/// player and by lightning, under the rain, blood stains and the arrow over the player as `view`
/// says, `view.frame` frames in.
pub fn render_map(
    buffer: &mut Buffer,
    area: Rect,
    world: &World,
    sprites: &Sprites,
    view: ViewState,
) {
    let (frame, blood) = (view.frame, view.blood);
    let player = &world.player;
    let centre = pixel(player.position);
    let mut canvas = Canvas::new(sprites, area, centre);
    canvas.paint_map(world, view.ground);
    night::paint_stains(&mut canvas, &world.stains, world.ticks(), blood);

    let covered = canvas.paint_figures(world, view.skin);
    if let Some(hurt_at) = world.hurt_at {
        let since = world.ticks().saturating_sub(hurt_at);
        night::paint_spurt(&mut canvas, centre, since, hurt_at, blood);
    }
    for (drone, broken) in world.drones() {
        let look = Look {
            tint: broken.then_some(BROKEN),
            ..Look::plain(SpriteId::Drone)
        };
        canvas.paint_at(look, drone);
    }

    // The night falls on all of that; what glows is painted after it, untouched.
    let strike = night::run_strike(world.lightning, world.ticks());
    let mut lights = Vec::new();
    if world.torch_shines() {
        lights.push(night::Light::torch(centre, world.light_radius()));
    }
    let campfires = canvas.paint_healing(world, frame);
    let (shining, halos) = Canvas::shines(world, &campfires, frame);
    lights.extend(campfires);
    lights.extend(shining);
    let glows = canvas.glows(world);
    lights.extend(glows.iter().map(|&(at, _)| night::Light::gem(at)));
    night::light(&mut canvas, &lights, night::flash(strike), world.daylight());
    night::paint_halos(&mut canvas, &halos);
    // Where lightning will strike, a moment before: its ring on the ground, blinking, under the
    // figures.
    for coming in &world.strikes_coming {
        let since = world.ticks().saturating_sub(coming.tick);
        night::paint_warning(&mut canvas, pixel(coming.at), since, &covered);
    }
    if let Some(healed_at) = world.healed_at {
        canvas.paint_heal(centre, world.ticks().saturating_sub(healed_at));
    }
    canvas.mark(world, view);
    // The Shadows' eyes glow around them, then show over their glow.
    let eye_glows = canvas.eye_glows(world);
    night::paint_glows(&mut canvas, &eye_glows, EYES_GLOW_STRENGTH);
    canvas.paint_shadow_eyes(world);
    canvas.paint_flames(world, frame);
    canvas.paint_molotovs(world, frame);
    // Each gem on screen glows its colour around it, then shows over its glow.
    night::paint_glows(&mut canvas, &glows, night::GEM_GLOW_STRENGTH);
    for gem in &world.gems {
        canvas.paint_at(Look::plain(sprites::gem(gem.value)), gem.position);
    }
    if let Some(swing) = &world.swing {
        canvas.paint_swing(swing, player.position);
    }
    canvas.paint_muzzles(world);
    for bullet in &world.bullets {
        canvas.paint_bullet(bullet);
    }
    let weather = world.weather();
    if let Some(rain) = night::run_rain(
        weather,
        world.next_weather(),
        world.weather_progress(),
        view.rain,
        world.daylight(),
    ) {
        night::paint_rain(&mut canvas, frame, rain);
    }
    if let Some(strike) = strike {
        night::paint_bolt(&mut canvas, strike);
        if weather.strikes_hurt() {
            night::paint_impact(&mut canvas, strike, STRIKE_RADIUS);
        }
    }
    let gear = if view.gear {
        gear_slots(world)
    } else {
        Vec::new()
    };
    let levels = if view.gear_list {
        // The list over the map dimmed, as the icons are.
        if !gear.is_empty() {
            let (width, lines) = list_size(&gear);
            canvas.veil(i32::from(width), i32::from(lines) * 2);
        }
        Vec::new()
    } else {
        canvas.paint_gear(&gear)
    };
    canvas.draw(buffer, area);
    if view.gear_list {
        draw_gear_list(buffer, area, &gear);
    } else {
        draw_gear_levels(buffer, area, &levels);
    }
}

/// A card the player has, as the gear shows it: its level, none for a legendary; its gauge, if
/// it has one: a gun's rounds or reload, the Dash card's dashes.
struct GearSlot {
    choice: Choice,
    level: Option<u8>,
    gauge: Option<(u32, u32, Color)>,
}

/// The cards the player has, in the order first picked.
fn gear_slots(world: &World) -> Vec<GearSlot> {
    let player = &world.player;
    let build = &player.build;
    let ammo = world.ammo();
    // The Dash card shows its dashes as a gun its rounds: full when all are ready, filling back
    // as they come back.
    let recharge = build.dash_recharge();
    let most = u32::from(build.dashes());
    let dashes = if u32::from(player.dashes.ready) >= most {
        (1, 1, GEAR_ROUNDS)
    } else {
        (
            u32::from(player.dashes.ready) * recharge + player.dashes.charging,
            most * recharge,
            GEAR_RELOADING,
        )
    };
    build
        .picked()
        .iter()
        .map(|&choice| match choice {
            Choice::Weapon(weapon) => GearSlot {
                choice,
                level: Some(build.weapon_level(weapon)),
                gauge: ammo.iter().find(|gun| gun.0 == weapon).map(|gun| {
                    let (_, magazine, full, reload) = *gun;
                    if magazine.reloading > 0 {
                        (reload - magazine.reloading, reload, GEAR_RELOADING)
                    } else {
                        (magazine.rounds, full, GEAR_ROUNDS)
                    }
                }),
            },
            Choice::Passive(passive) => GearSlot {
                choice,
                level: Some(build.passive_level(passive)),
                gauge: (passive == Passive::Dash).then_some(dashes),
            },
            Choice::Legendary(_) | Choice::Heal => GearSlot {
                choice,
                level: None,
                gauge: None,
            },
        })
        .collect()
}

/// A card's level as the gear writes it: its number, or MAX in gold at the last.
fn level_text(level: u8) -> (String, Color) {
    if level >= MAX_LEVEL {
        ("MAX".to_string(), GEAR_MAX)
    } else {
        (level.to_string(), GEAR_LEVEL)
    }
}

/// The longest name in the gear, in cells.
fn longest_name(gear: &[GearSlot]) -> usize {
    gear.iter()
        .map(|card| level_up::name(card.choice).chars().count())
        .max()
        .unwrap_or(0)
}

/// How much of the map the gear's list takes, in cells: a space, the longest name, a space, the
/// level, two spaces, the gauge, a space across; a line a card, and one above and below.
fn list_size(gear: &[GearSlot]) -> (u16, u16) {
    let width = longest_name(gear) + 1 + 1 + 3 + 2 + 6 + 1;
    (
        u16::try_from(width).unwrap_or(u16::MAX),
        u16::try_from(gear.len() + 2).unwrap_or(u16::MAX),
    )
}

/// The gear as a list over the top left of the map, dimmed under it, a line a card: its name,
/// its level, its gauge if it has one.
fn draw_gear_list(buffer: &mut Buffer, area: Rect, gear: &[GearSlot]) {
    let name_width = longest_name(gear);
    let lines = list_size(gear).1.min(area.height);
    let ground = Style::new();
    for (row, card) in (1_u16..).zip(gear) {
        if row + 1 >= lines {
            break;
        }
        let y = area.y + row;
        let name = level_up::name(card.choice).to_uppercase();
        let mut x = area.x + 1;
        buffer.set_string(x, y, &name, ground.fg(GEAR_NAME));
        x += u16::try_from(name_width).unwrap_or(0) + 1;
        if let Some(level) = card.level {
            let (text, colour) = level_text(level);
            buffer.set_string(
                x,
                y,
                format!("{text:>3}"),
                ground.fg(colour).add_modifier(Modifier::BOLD),
            );
        }
        x += 3 + 2;
        if let Some((part, whole, colour)) = card.gauge {
            let lit = share_of(part, whole, LIST_GAUGE);
            for cell in 0..LIST_GAUGE {
                let shade = if cell < lit { colour } else { GEAR_EMPTY };
                buffer.set_string(
                    x + u16::try_from(cell).unwrap_or(0),
                    y,
                    "━",
                    ground.fg(shade),
                );
            }
        }
    }
}

/// Where a card's level is written in the gear: the pixel column just right of its icon, and a
/// pixel row of the cell it goes in, from the canvas' top left; and the level.
struct GearLevel {
    right: i32,
    row: i32,
    level: u8,
}

/// Each card's level in the gear, over its icon's bottom right corner, on a dark badge: its
/// number, or MAX in gold once at the last.
fn draw_gear_levels(buffer: &mut Buffer, area: Rect, levels: &[GearLevel]) {
    for gear in levels {
        let (text, colour) = level_text(gear.level);
        let width = i32::try_from(text.len()).unwrap_or(0);
        let (Ok(x), Ok(y)) = (
            u16::try_from(gear.right - width),
            u16::try_from(gear.row / 2),
        ) else {
            continue;
        };
        if x + 3 > area.width || y >= area.height {
            continue;
        }
        let style = Style::new()
            .fg(colour)
            .bg(GEAR_BADGE)
            .add_modifier(Modifier::BOLD);
        buffer.set_string(area.x + x, area.y + y, text, style);
    }
}

/// Where the player fell, for the death screen: the map around them in the dark, lit only by
/// their fallen torch, the horde still about, dimmed for the words over it; a pool of blood
/// where they lay, as `view.blood` says; the weather they fell in, `view.frame` frames in, its
/// rain shown as `view.rain` says. Held out until the dawn, they stand there instead, as
/// `view.skin` says, in the light of the rising sun, the rain over.
pub fn render_death_scene(
    buffer: &mut Buffer,
    area: Rect,
    world: &World,
    sprites: &Sprites,
    view: ViewState,
) {
    let (frame, blood, rain) = (view.frame, view.blood, view.rain);
    let player = &world.player;
    let centre = pixel(player.position);
    let mut canvas = Canvas::new(sprites, area, centre);
    canvas.paint_map(world, view.ground);
    let pool = Stain {
        position: player.position,
        tick: world.ticks(),
        size: DEATH_POOL,
        ash: false,
    };
    let dawn = world.survived();
    night::paint_stains(&mut canvas, &world.stains, world.ticks(), blood);
    let mut figures: Vec<(Vec2, Look)> = world
        .enemies
        .iter()
        .map(|enemy| (enemy.position, sprites::enemy(enemy)))
        .collect();
    if dawn {
        figures.push((player.position, sprites::hero(player.facing, view.skin)));
    } else {
        night::paint_stains(&mut canvas, &[pool], world.ticks(), blood);
    }
    figures.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
    for (position, look) in figures {
        canvas.paint_at(look, position);
    }
    let weather = world.weather();
    let strike = weather
        .lightning_every()
        .filter(|_| !dawn)
        .and_then(|every| night::strike_every(frame, every));
    let mut lights = vec![night::Light::torch(centre, DEATH_LIGHT)];
    lights.extend(canvas.paint_healing(world, frame));
    night::light(
        &mut canvas,
        &lights,
        night::flash(strike),
        if dawn { 1.0 } else { world.daylight() },
    );
    night::dim(&mut canvas, DEATH_DIM);
    if let Some(rain) = night::run_rain(
        weather,
        world.next_weather(),
        world.weather_progress(),
        rain,
        world.daylight(),
    ) && !dawn
    {
        night::paint_rain(&mut canvas, frame, rain);
    }
    if let Some(strike) = strike {
        night::paint_bolt(&mut canvas, strike);
    }
    canvas.draw(buffer, area);
}

/// The compass's arrow: on the edge of `area`, the way the healing at `target` lies from the
/// player in its middle; none while the healing is on screen, for the torch to find.
pub fn render_compass(buffer: &mut Buffer, area: Rect, player: Vec2, target: Vec2, frame: u32) {
    let (centre, aim) = (pixel(player), pixel(target));
    // In cells from the middle of the area: a cell is a pixel wide and two tall.
    let across = aim.0 - centre.0;
    let down = (aim.1 - centre.1).div_euclid(2);
    let (half_width, half_height) = (i32::from(area.width) / 2, i32::from(area.height) / 2);
    if across.abs() < half_width && down.abs() < half_height {
        return;
    }
    // Along the way to the healing, up to the edge.
    let (edge_x, edge_y) = (half_width - 1, half_height - 1);
    let (across_f, down_f) = (cells(across), cells(down));
    let reach = [
        (across != 0).then(|| cells(edge_x) / across_f.abs()),
        (down != 0).then(|| cells(edge_y) / down_f.abs()),
    ]
    .into_iter()
    .flatten()
    .fold(f32::MAX, f32::min);
    #[allow(
        clippy::cast_possible_truncation,
        reason = "within the area, a few hundred cells"
    )]
    let (x, y) = (
        (across_f * reach).round() as i32 + half_width,
        (down_f * reach).round() as i32 + half_height,
    );
    let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
        return;
    };
    // The way, as the eye sees it: two rows of pixels to a cell.
    let arrow = arrow(across_f, down_f * 2.0);
    // A white arrow on red cells, pulsing, a healing cross beside it, on the inner side: seen
    // at a glance on the edge of the night.
    let style = Style::new()
        .fg(Color::Rgb(255, 255, 255))
        .bg(if (frame / COMPASS_PULSE).is_multiple_of(2) {
            COMPASS
        } else {
            COMPASS_DIM
        })
        .add_modifier(Modifier::BOLD);
    let (x, y) = (x.min(area.width - 1), y.min(area.height - 1));
    let cross = if i32::from(x) >= half_width {
        x.saturating_sub(1)
    } else {
        (x + 1).min(area.width - 1)
    };
    buffer.set_string(area.x + cross, area.y + y, "✚", style);
    buffer.set_string(area.x + x, area.y + y, arrow, style);
}

/// A count of cells, for the compass's sums.
fn cells(count: i32) -> f32 {
    f32::from(i16::try_from(count).unwrap_or(i16::MAX))
}

/// The arrow nearest a way, among the eight.
fn arrow(across: f32, down: f32) -> &'static str {
    const ARROWS: [&str; 8] = ["→", "↘", "↓", "↙", "←", "↖", "↑", "↗"];
    let eighth = (down.atan2(across) / std::f32::consts::FRAC_PI_4).round();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "an eighth of a turn, from -4 to 4"
    )]
    let index = (eighth as i32).rem_euclid(8) as usize;
    ARROWS[index]
}

/// How far the fallen torch lights the death screen, in tiles.
const DEATH_LIGHT: f32 = 6.0;
/// How big the pool of blood where the player fell.
const DEATH_POOL: u8 = 10;
/// How bright the death screen's scene is, behind its words.
const DEATH_DIM: f32 = 0.55;

/// A sprite alone, in the middle of `area`: a picture on a card.
pub fn render_sprite(buffer: &mut Buffer, area: Rect, sprites: &Sprites, sprite: SpriteId) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let picture = sprites.get(sprite);
    canvas.paint(
        Look::plain(sprite),
        (-picture.width() / 2, -picture.height() / 2),
    );
    canvas.draw(buffer, area);
}

/// A sprite alone, `scale` times as big, in the middle of `area`.
pub fn render_sprite_scaled(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    sprite: SpriteId,
    scale: i32,
) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let picture = sprites.get(sprite);
    let (left, top) = (-picture.width() * scale / 2, -picture.height() * scale / 2);
    for (x, y, color) in picture.pixels(false) {
        for dy in 0..scale {
            for dx in 0..scale {
                let point = (left + x * scale + dx, top + y * scale + dy);
                if let Some(index) = canvas.index(point) {
                    canvas.pixels[index] = Some(color);
                }
            }
        }
    }
    canvas.draw(buffer, area);
}

/// The pixels along a swing's arc, for each of its radii.
const SLASH_STEPS: u8 = 24;

/// Half the map `area` shows each way, in tiles.
pub fn view(area: Rect) -> Vec2 {
    // A column is a pixel wide and a row two pixels tall, a tile 2 × 2 pixels.
    Vec2::new(f32::from(area.width) / 4.0, f32::from(area.height) / 2.0)
}

/// How many of `pixels` a share of `part` in `whole` lights, rounded up: one round left shows.
fn share_of(part: u32, whole: u32, pixels: i32) -> i32 {
    let pixels_u32 = u32::try_from(pixels).unwrap_or(0);
    let lit = (part * pixels_u32).div_ceil(whole.max(1)).min(pixels_u32);
    i32::try_from(lit).unwrap_or(0)
}

/// The pixel a point of the map falls on.
pub fn pixel(point: Vec2) -> (i32, i32) {
    #[allow(clippy::cast_precision_loss, reason = "2 is exact")]
    let scale = TILE_PIXELS as f32;
    (point * scale).tile()
}

/// What lies on a tile of open ground, if anything: scattered, always the same.
/// The ground's colour at a pixel of the map: grass, speckled with earth, the same pixels always.
fn ground((x, y): (i32, i32)) -> Color {
    let hash =
        x.cast_unsigned().wrapping_mul(2_654_435_761) ^ y.cast_unsigned().wrapping_mul(40_503);
    if hash.wrapping_mul(2_246_822_519) >> 24 < EARTH_SHARE {
        EARTH
    } else {
        GRASS
    }
}

fn decor((x, y): (i32, i32)) -> Option<SpriteId> {
    let hash =
        x.cast_unsigned().wrapping_mul(73_856_093) ^ y.cast_unsigned().wrapping_mul(19_349_663);
    match hash % 29 {
        0..=2 => Some(SpriteId::Grass),
        3 => Some(SpriteId::Pebble),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_compass_arrow_points_the_way_the_eye_sees() {
        assert_eq!(arrow(1.0, 0.0), "→");
        assert_eq!(arrow(0.0, 1.0), "↓");
        assert_eq!(arrow(-1.0, -1.0), "↖");
        assert_eq!(arrow(1.0, -1.0), "↗");
    }

    #[test]
    fn the_compass_arrow_stands_on_the_edge_the_healing_lies_beyond_a_cross_beside_it() {
        let area = Rect::new(0, 0, 40, 20);
        let mut buffer = Buffer::empty(area);
        render_compass(
            &mut buffer,
            area,
            Vec2::new(0.5, 0.5),
            Vec2::new(200.5, 0.5),
            0,
        );
        let row: String = (0..40)
            .map(|x| buffer[(x, 10)].symbol().to_owned())
            .collect();
        assert_eq!(row.trim(), "✚→");
        assert_eq!(buffer[(39, 10)].symbol(), "→");
    }

    #[test]
    fn a_share_lights_its_pixels_one_round_left_showing() {
        assert_eq!(share_of(0, 6, 8), 0);
        assert_eq!(share_of(1, 20, 8), 1);
        assert_eq!(share_of(3, 6, 8), 4);
        assert_eq!(share_of(9, 9, 8), 8);
    }

    #[test]
    fn the_compass_says_nothing_of_healing_on_screen() {
        let area = Rect::new(0, 0, 40, 20);
        let mut buffer = Buffer::empty(area);
        render_compass(
            &mut buffer,
            area,
            Vec2::new(0.5, 0.5),
            Vec2::new(5.5, 2.5),
            0,
        );
        assert_eq!(buffer, Buffer::empty(area));
    }
}
