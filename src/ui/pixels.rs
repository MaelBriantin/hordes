//! The map in pixels: a tile is 2 × 2 pixels, a cell holds 2 pixels one above the other. The
//! scene is painted in pixels first, then each cell shows its two as a half block.

mod blows;
mod c4;
mod figures;
mod fire;
mod fists;
mod flashball;
mod gatling;
mod gear;
mod grenade;
mod napalm;
mod ninja;
mod ram;
mod revival;
mod saw;
mod smoke;
mod sniper;
mod stabs;
mod thrust;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::night::{self, BloodLook};
use super::sprites::{self, Look, Outfit, SpriteId, Sprites};
use super::{ViewState, level_up};
use crate::game::{
    Barricade, Bullet, Choice, OBSTACLE_SIZE, Obstacle, STRIKE_RADIUS, SWING_TICKS, Stain, Swing,
    Vec2, Weapon, World,
};
use gear::{draw_gear_levels, draw_gear_list, gear_slots, list_size};

/// The glow of a gem, by what it is worth, as its colour.
fn gem_glow(value: u32) -> (u8, u8, u8) {
    match value {
        0 | 1 => (150, 180, 230),
        2..=4 => (60, 130, 255),
        _ => (245, 190, 60),
    }
}
/// A spark where a bullet hits a tree or a rock: bright, then fading.
const SPARK: (Color, Color) = (Color::Rgb(255, 240, 180), Color::Rgb(200, 150, 70));
/// Pixels per tile, each way.
const TILE_PIXELS: i32 = 2;

/// A campfire's light swaying, `frame` frames in, from -1 to 1: two waves of its own, one slow,
/// one quick, never quite the same as another fire's.
fn sway(frame: u32, (x, y): (i32, i32)) -> f32 {
    let seed = f32::from(u16::try_from((x * 31 + y * 17).rem_euclid(1_000)).unwrap_or(0));
    let time = f32::from(u16::try_from(frame % 10_000).unwrap_or(0));
    (time * 0.21 + seed).sin() * 0.6 + (time * 0.67 + seed * 1.7).sin() * 0.4
}

/// How far the infiltrator sees around them, their torch out, in tiles: less than a torch.
const NIGHT_VISION_RADIUS: f32 = 7.0;
/// The ground: dark grass, and the earth showing through it, out of 256 pixels.
const GRASS: Color = Color::Rgb(32, 46, 28);
const EARTH: Color = Color::Rgb(44, 38, 28);
const EARTH_SHARE: u32 = 90;
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
/// An enemy on fire: how far its flames light, in tiles; where they flicker, from the middle of
/// the top of its sprite, in pixels; their colours.
const BURNING_LIGHT: f32 = 1.5;
const FLAMES: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, -1), (0, -2)];
const FLAME_COLOURS: [Color; 3] = [
    Color::Rgb(255, 210, 80),
    Color::Rgb(245, 130, 40),
    Color::Rgb(220, 60, 30),
];
/// The crowbar's blow, iron grey, then as it fades.
const CROWBAR_BLOW: (Color, Color) = (Color::Rgb(160, 165, 180), Color::Rgb(90, 92, 105));
/// A shotgun's pellets, every other one: the gold and the red of its shells.
const PELLETS: (Color, Color) = (Color::Rgb(240, 190, 60), Color::Rgb(220, 50, 50));
/// A machine gun's tracer, from its head to the end of its trail.
/// A shuriken in flight: the star, then its streak.
const SHURIKEN: [Color; 3] = [
    Color::Rgb(245, 245, 250),
    Color::Rgb(205, 205, 215),
    Color::Rgb(110, 112, 120),
];
const TRACER: [Color; 3] = [
    Color::Rgb(255, 225, 160),
    Color::Rgb(230, 140, 60),
    Color::Rgb(130, 70, 30),
];
/// A barricade that can't be set down, blinking every so many ticks.
const REFUSED: Color = Color::Rgb(220, 50, 50);
const REFUSED_BLINK: u32 = 3;
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

    /// A bullet: a pistol's, a white pixel; a machine gun's, a thin tracer, its trail fading
    /// behind it.
    fn paint_bullet(&mut self, bullet: &Bullet) {
        if bullet.weapon == Weapon::Pistol {
            self.paint_at(Look::plain(SpriteId::Bullet), bullet.position);
            return;
        }
        // A shuriken: a small star, a streak behind it.
        if bullet.weapon == Weapon::Shuriken {
            let way = bullet.way();
            for (step, colour) in (0_u8..).zip(SHURIKEN) {
                self.set(
                    pixel(bullet.position - way * (0.5 * f32::from(step))),
                    colour,
                );
            }
            return;
        }
        // The flashball's ball: black, a grey trail behind it.
        if bullet.weapon == Weapon::Flashball {
            let way = bullet.way();
            self.set(pixel(bullet.position - way * 0.5), flashball::BALL_TRAIL);
            self.set(pixel(bullet.position), flashball::BALL);
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

    /// The melee weapons' strikes over the figures: the katana's cuts, the chainsaw grinding,
    /// the baton's thrust, the defibrillator's shock, the knife's stabs.
    fn paint_strikes(&mut self, world: &World, outfit: Outfit, blood: BloodLook) {
        self.paint_cuts(world, outfit);
        self.paint_saw(world, blood);
        self.paint_thrust(world);
        self.paint_shock(world);
        self.paint_stabs(world, blood);
        self.paint_punch(world);
        self.paint_brace(world);
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

    /// A sprite at its top left `corner`, greyed as a locked card in the collection: a spent
    /// card's icon.
    fn paint_greyed(&mut self, sprite: SpriteId, (left, top): (i32, i32)) {
        let picture = self.sprites.get(sprite);
        for (x, y, color) in picture.pixels(false) {
            if let Some(index) = self.index((left + x, top + y)) {
                self.pixels[index] = Some(super::collection::grey(color));
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

    /// A machete's swing around `centre`: an arc of pixels, sweeping from one end to the other
    /// in the first half of its time, fading in the second.
    fn paint_swing(&mut self, swing: &Swing, centre: Vec2) {
        let shown = SWING_TICKS - swing.left;
        let sweeping = shown < SWING_TICKS / 2;
        // The crowbar's and the truncheon's blow, iron grey; the machete's slash, a blade's white.
        let (bright, fading) = if matches!(swing.weapon, Weapon::Crowbar | Weapon::Truncheon) {
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

    /// Where a bullet hit a tree or a rock: a spark, a cross at first, then a point, fading.
    fn paint_sparks(&mut self, world: &World) {
        for &(at, tick) in &world.sparks {
            let age = world.ticks().saturating_sub(tick);
            let (x, y) = pixel(at);
            let colour = if age < 2 { SPARK.0 } else { SPARK.1 };
            self.set((x, y), colour);
            if age < 2 {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.set((x + dx, y + dy), colour);
                }
            }
        }
    }

    /// A sprite centred on a point of the map.
    fn paint_at(&mut self, look: Look, position: Vec2) {
        self.paint(look, self.corner_of(look, position));
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

    /// The barricades standing as figures, each with how low it stands; white a moment once
    /// hit.
    fn barricade_figures(&self, world: &World) -> Vec<(f32, Look, (i32, i32))> {
        world
            .barricades
            .iter()
            .map(|barricade| {
                let (sprite, corner) = self.barricade_placed(barricade);
                let look = Look {
                    tint: (barricade.flash > 0).then_some(sprites::FLASH),
                    ..Look::plain(sprite)
                };
                (barricade.centre().y, look, corner)
            })
            .collect()
    }

    /// A barricade's sprite, and its top left pixel: its crates on the bottom of the tiles it
    /// covers, a file's one pixel past them on the right.
    fn barricade_placed(&self, barricade: &Barricade) -> (SpriteId, (i32, i32)) {
        let sprite = match (barricade.file(), barricade.metal) {
            (false, false) => SpriteId::Barricade,
            (false, true) => SpriteId::BarricadeMetal,
            (true, false) => SpriteId::BarricadeFile,
            (true, true) => SpriteId::BarricadeFileMetal,
        };
        let (x, y) = barricade.corner;
        let bottom = (y + barricade.size().1) * TILE_PIXELS;
        let height = self.sprites.get(sprite).height();
        (sprite, (x * TILE_PIXELS, bottom - height))
    }

    /// A barricade that couldn't be set down, the ground taken: in red where it would have
    /// stood, blinking, over everything.
    fn paint_refused(&mut self, world: &World) {
        let Some((barricade, at)) = world.barricade_refused else {
            return;
        };
        if (world.ticks().saturating_sub(at) / REFUSED_BLINK).is_multiple_of(2) {
            let (sprite, corner) = self.barricade_placed(&barricade);
            let look = Look {
                tint: Some(REFUSED),
                ..Look::plain(sprite)
            };
            self.paint(look, corner);
        }
    }

    /// Where a sprite centred on `position` has its top left pixel.
    fn corner_of(&self, look: Look, position: Vec2) -> (i32, i32) {
        let centre = pixel(position);
        let sprite = self.sprites.get(look.sprite);
        (
            centre.0 - sprite.width() / 2,
            centre.1 - sprite.height() / 2,
        )
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

impl Canvas<'_> {
    /// What lights the night: the torch, the campfires (painted), the fires and the blasts,
    /// with their halos; the gems and the charges' lights, with their glows.
    fn lights(
        &mut self,
        world: &World,
        frame: u32,
        outfit: Outfit,
    ) -> (Vec<night::Light>, Vec<night::Halo>, Vec<night::Glow>) {
        let mut lights = Vec::new();
        if world.torch_shines() {
            lights.push(night::Light::torch(
                pixel(world.player.position),
                world.light_radius(),
            ));
        }
        // The infiltrator, their torch out, sees around them in green.
        if world.night_vision() {
            // The second outfit's goggles are amber.
            let goggles = if outfit.colours == 1 {
                night::YELLOW_GOGGLES
            } else {
                night::GREEN_GOGGLES
            };
            lights.push(night::Light::night_vision(
                pixel(world.player.position),
                NIGHT_VISION_RADIUS,
                goggles,
            ));
        }
        let campfires = self.paint_healing(world, frame);
        let (shining, mut halos) = Canvas::shines(world, &campfires, frame);
        lights.extend(campfires);
        lights.extend(shining);
        let (blasting, blast_halos) = Canvas::blast_lights(world);
        lights.extend(blasting);
        halos.extend(blast_halos);
        let mut glows = self.glows(world);
        glows.extend(Canvas::charge_glows(world));
        glows.extend(Canvas::sniper_glows(world));
        lights.extend(glows.iter().map(|&(at, _)| night::Light::gem(at)));
        (lights, halos, glows)
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
    for blast in &world.blasts {
        canvas.paint_blast(blast);
    }

    // The hammer's wave on the ground, under the enemies it throws out; a dodge's ghost, under
    // the hero.
    canvas.paint_blow(world);
    canvas.paint_charges(world);
    canvas.paint_dodge(world, view.outfit());
    canvas.paint_vault(world);
    let covered = canvas.paint_figures(world, view.outfit());
    if let Some(hurt_at) = world.hurt_at {
        let since = world.ticks().saturating_sub(hurt_at);
        night::paint_spurt(&mut canvas, centre, since, hurt_at, blood);
    }
    canvas.paint_strikes(world, view.outfit(), blood);
    for drone in world.drones() {
        canvas.paint_at(Look::plain(SpriteId::Drone), drone);
    }
    canvas.paint_gatling(world, frame);
    canvas.paint_flashball(world, blood);
    canvas.paint_revival(world, view.outfit());
    // The scout's smoke over what stands in it.
    canvas.paint_smoke(world, view.outfit(), frame);

    // The night falls on all of that; what glows is painted after it, untouched.
    let strike = night::run_strike(world.lightning, world.ticks());
    let (lights, halos, glows) = canvas.lights(world, frame, view.outfit());
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
    canvas.paint_shadow_eyes(world);
    canvas.paint_refused(world);
    // With Horde Hunter from its level 3, the hordes' veiled show in the dark too.
    if world.player.build.horde_veil_in_dark() {
        canvas.paint_veils(world);
    }
    canvas.paint_flames(world, frame);
    canvas.paint_molotovs(world, frame);
    canvas.paint_grenades(world, frame);
    canvas.paint_flamethrower(world, frame);
    canvas.paint_fireballs(world);
    canvas.paint_bombs(world);
    // Each gem on screen glows its colour around it, then shows over its glow; each charge's
    // light, lit, too.
    night::paint_glows(&mut canvas, &glows);
    canvas.paint_charge_lights(world);
    canvas.paint_sniper_dot(world);
    canvas.paint_sniper_shot(world);
    canvas.paint_sniper_impacts(world, blood);
    for gem in &world.gems {
        canvas.paint_at(Look::plain(sprites::gem(gem.value)), gem.position);
    }
    if let Some(swing) = &world.swing {
        canvas.paint_swing(swing, player.position);
    }
    for bullet in &world.bullets {
        canvas.paint_bullet(bullet);
    }
    canvas.paint_sparks(world);
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
    // The player picked out over everything of the map: gems, rain, lightning.
    canvas.mark(world, view);
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
        .map(|enemy| {
            let lift = sprites::lift(enemy, world.ticks());
            (
                enemy.position - Vec2::new(0.0, lift),
                sprites::enemy(enemy, world.ticks()),
            )
        })
        .collect();
    if dawn {
        figures.push((
            player.position,
            sprites::hero(world.hero_facing(), view.outfit()),
        ));
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

/// A card's icon alone, in the middle of `area`, framed in its rarity's colour, a pixel of room
/// between; greyed and crossed out in red if it is spent.
pub fn render_card_icon(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    choice: Choice,
    spent: bool,
) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let sprite = level_up::icon(choice);
    let picture = sprites.get(sprite);
    let corner = (-picture.width() / 2, -picture.height() / 2);
    canvas.frame_card(corner, picture.width(), choice);
    if spent {
        canvas.paint_greyed(sprite, corner);
        canvas.cross_out(corner, picture.width());
    } else {
        canvas.paint(Look::plain(sprite), corner);
    }
    canvas.draw(buffer, area);
}

/// A card's level as the gear writes it, ending just before `right` on row `y`, on a dark
/// badge: its number, or MAX in gold at the last.
pub fn render_level_badge(buffer: &mut Buffer, (right, y): (u16, u16), level: u8) {
    gear::draw_level_badge(buffer, (right, y), level);
}

/// A sprite alone, `scale` times as big, in the middle of `area`.
pub fn render_sprite_scaled(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    sprite: SpriteId,
    scale: i32,
) {
    render_look_scaled(buffer, area, sprites, Look::plain(sprite), scale);
}

/// A sprite alone as `look` has it (mirrored or not), `scale` times as big, in the middle of
/// `area`.
pub fn render_look_scaled(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    look: Look,
    scale: i32,
) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let picture = sprites.get(look.sprite);
    // Its top on a cell's top row, for its rows not to straddle the cells' halves.
    let height = i32::from(area.height) * 2;
    let top = ((height - picture.height() * scale) / 2).max(0) / 2 * 2 - height / 2;
    let left = -picture.width() * scale / 2;
    for (x, y, color) in picture.pixels(look.mirrored) {
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

/// How many of `pixels` a share of `part` in `whole` lights: rounded up, for one round left to
/// show; or down, for a reload to start empty and fill.
fn share_of(part: u32, whole: u32, pixels: i32, up: bool) -> i32 {
    let pixels_u32 = u32::try_from(pixels).unwrap_or(0);
    let lit = if up {
        (part * pixels_u32).div_ceil(whole.max(1))
    } else {
        part * pixels_u32 / whole.max(1)
    };
    i32::try_from(lit.min(pixels_u32)).unwrap_or(0)
}

/// The pixel a point of the map falls on.
/// A length of the map, in tiles, in pixels, rounded.
fn burst_pixels(tiles: f32) -> i32 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "a burst's reach, a few tiles"
    )]
    let pixels = (tiles * TILE_PIXELS as f32).round() as i32;
    pixels
}

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
    fn a_share_lights_its_pixels_one_round_left_showing() {
        assert_eq!(share_of(0, 6, 8, true), 0);
        assert_eq!(share_of(1, 20, 8, true), 1);
        assert_eq!(share_of(3, 6, 8, true), 4);
        assert_eq!(share_of(9, 9, 8, true), 8);
        // A reload just started shows empty.
        assert_eq!(share_of(1, 20, 8, false), 0);
        assert_eq!(share_of(20, 20, 8, false), 8);
    }
}
