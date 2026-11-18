//! The map in pixels: a tile is 2 × 2 pixels, a cell holds 2 pixels one above the other. The
//! scene is painted in pixels first, then each cell shows its two as a half block.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use super::level_up;
use super::night;
use super::sprites::{self, Look, SpriteId, Sprites};
use crate::game::{
    Choice, Feature, MAX_LEVEL, OBSTACLE_SIZE, Obstacle, STRIKE_RADIUS, SWING_TICKS, Stain, Swing,
    Vec2, World, campfire_light, daylight, weather_progress,
};

/// Pixels per tile, each way.
const TILE_PIXELS: i32 = 2;
/// The frames a campfire's flame keeps one look before the other.
const FLICKER: u32 = 6;
/// The compass's arrow: on the red of the herbs, and a darker red as it pulses, every so many
/// frames.
const COMPASS: Color = Color::Rgb(220, 45, 65);
const COMPASS_DIM: Color = Color::Rgb(130, 25, 38);
const COMPASS_PULSE: u32 = 10;
/// The gear over the top left of the map: the share of the light its veil leaves, the room
/// between two icons and around them, in pixels; the levels and rounds under the icons.
const VEIL: f32 = 0.55;
const GEAR_GAP: i32 = 2;
const GEAR_LEVEL: Color = Color::Rgb(235, 235, 240);
const GEAR_ROUNDS: Color = Color::Rgb(235, 210, 120);
const GEAR_RELOADING: Color = Color::Rgb(165, 165, 172);
const GEAR_EMPTY: Color = Color::Rgb(70, 70, 80);
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

    /// A sprite with its top-left pixel at a pixel of the map.
    fn paint(&mut self, look: Look, (left, top): (i32, i32)) {
        let sprite = self.sprites.get(look.sprite);
        for (x, y, color) in sprite.pixels(look.mirrored) {
            if let Some(index) = self.index((left + x, top + y)) {
                self.pixels[index] = Some(look.tint.unwrap_or(color));
            }
        }
    }

    /// The player's gear, over the top left of what the canvas shows, on a dark veil: each
    /// weapon's icon, then each passive's, as on the level-up cards; under each, its level, a
    /// pixel a level; under a gun's, its rounds left, or its reload as it goes.
    fn paint_gear(&mut self, world: &World) {
        let build = &world.player.build;
        let ammo = world.ammo();
        let weapons = build.weapons().iter().map(|&(weapon, level)| {
            let rounds = ammo.iter().find(|gun| gun.0 == weapon).map(|gun| {
                let (_, magazine, full, reload) = *gun;
                if magazine.reloading > 0 {
                    (reload - magazine.reloading, reload, GEAR_RELOADING)
                } else {
                    (magazine.rounds, full, GEAR_ROUNDS)
                }
            });
            (level_up::icon(Choice::Weapon(weapon)), level, rounds)
        });
        let passives = build
            .passives()
            .iter()
            .map(|&(passive, level)| (level_up::icon(Choice::Passive(passive)), level, None));
        let gear: Vec<_> = weapons.chain(passives).collect();
        if gear.is_empty() {
            return;
        }
        let size = self.sprites.get(SpriteId::IconMachete).width();
        let step = size + GEAR_GAP;
        let width = GEAR_GAP + step * i32::try_from(gear.len()).unwrap_or(0);
        // The icon, a row, the level, a row, the rounds, a row.
        let height = GEAR_GAP + size + 6;
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
        for (slot, (sprite, level, rounds)) in (0..).zip(gear) {
            let x = left + GEAR_GAP + step * slot;
            let y = top + GEAR_GAP;
            self.paint(Look::plain(sprite), (x, y));
            for pixel in 0..i32::from(MAX_LEVEL) {
                let lit = pixel < i32::from(level);
                self.set(
                    (x + pixel, y + size + 1),
                    if lit { GEAR_LEVEL } else { GEAR_EMPTY },
                );
            }
            if let Some((part, whole, colour)) = rounds {
                let lit = share_of(part, whole, size);
                for pixel in 0..size {
                    self.set(
                        (x + pixel, y + size + 3),
                        if pixel < lit { colour } else { GEAR_EMPTY },
                    );
                }
            }
        }
    }

    /// A machete's swing around `centre`: an arc of pixels, sweeping from one end to the other
    /// in the first half of its time, fading in the second.
    fn paint_swing(&mut self, swing: &Swing, centre: Vec2) {
        let shown = SWING_TICKS - swing.left;
        let sweeping = shown < SWING_TICKS / 2;
        let color = if sweeping { SLASH } else { SLASH_FADING };
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
                        lights.push(night::Light::fire(centre, campfire_light(left)));
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

    /// The ground: grass and pebbles scattered on it, always the same, so it is seen to move
    /// under the player. And the obstacles standing on it.
    fn paint_map(&mut self, world: &World) {
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
/// player and by lightning, under the `rain`, `frame` frames in; blood stains as `blood` says.
pub fn render_map(
    buffer: &mut Buffer,
    area: Rect,
    world: &World,
    sprites: &Sprites,
    frame: u32,
    blood: night::BloodLook,
    rain: night::RainDisplay,
) {
    let player = &world.player;
    let centre = pixel(player.position);
    let mut canvas = Canvas::new(sprites, area, centre);
    canvas.paint_map(world);
    night::paint_stains(&mut canvas, &world.stains, world.ticks(), blood);

    // The player and the enemies, the lowest on the map last: in front of those above them.
    let mut figures: Vec<(Vec2, Look)> = world
        .enemies
        .iter()
        .map(|enemy| (enemy.position, sprites::enemy(enemy)))
        .collect();
    // Blinking while nothing can hurt them.
    let blinks_out = player.invincible > 0 && (world.ticks() / 3).is_multiple_of(2);
    if !blinks_out {
        figures.push((player.position, sprites::hero(player.facing)));
    }
    figures.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
    for (position, look) in figures {
        canvas.paint_at(look, position);
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
    let mut lights = vec![night::Light::torch(centre, world.light_radius())];
    lights.extend(canvas.paint_healing(world, frame));
    night::light(
        &mut canvas,
        &lights,
        night::flash(strike),
        daylight(world.ticks()),
    );
    for gem in &world.gems {
        canvas.paint_at(Look::plain(SpriteId::Gem), gem.position);
    }
    if let Some(swing) = &world.swing {
        canvas.paint_swing(swing, player.position);
    }
    for bullet in &world.bullets {
        canvas.paint_at(Look::plain(SpriteId::Bullet), bullet.position);
    }
    let weather = world.weather();
    if let Some(rain) = night::run_rain(
        weather,
        weather_progress(world.seconds()),
        rain,
        daylight(world.ticks()),
    ) {
        night::paint_rain(&mut canvas, frame, rain);
    }
    if let Some(strike) = strike {
        night::paint_bolt(&mut canvas, strike);
        if weather.strikes_hurt() {
            night::paint_impact(&mut canvas, strike, STRIKE_RADIUS);
        }
    }
    canvas.paint_gear(world);
    canvas.draw(buffer, area);
}

/// Where the player fell, for the death screen: the map around them in the dark, lit only by
/// their fallen torch, the horde still about, dimmed for the words over it; a pool of blood
/// where they lay, as `blood` says; the weather they fell in, `frame` frames in, its rain shown as
/// `rain` says. Held out until the dawn, they stand there instead, in the light of the rising
/// sun, the rain over.
pub fn render_death_scene(
    buffer: &mut Buffer,
    area: Rect,
    world: &World,
    sprites: &Sprites,
    frame: u32,
    blood: night::BloodLook,
    rain: night::RainDisplay,
) {
    let player = &world.player;
    let centre = pixel(player.position);
    let mut canvas = Canvas::new(sprites, area, centre);
    canvas.paint_map(world);
    let pool = Stain {
        position: player.position,
        tick: world.ticks(),
        size: DEATH_POOL,
    };
    let dawn = world.survived();
    night::paint_stains(&mut canvas, &world.stains, world.ticks(), blood);
    let mut figures: Vec<(Vec2, Look)> = world
        .enemies
        .iter()
        .map(|enemy| (enemy.position, sprites::enemy(enemy)))
        .collect();
    if dawn {
        figures.push((player.position, sprites::hero(player.facing)));
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
        if dawn { 1.0 } else { daylight(world.ticks()) },
    );
    night::dim(&mut canvas, DEATH_DIM);
    if let Some(rain) = night::run_rain(
        weather,
        weather_progress(world.seconds()),
        rain,
        daylight(world.ticks()),
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
    buffer.set_string(
        area.x + x.min(area.width - 1),
        area.y + y.min(area.height - 1),
        arrow,
        // A white arrow on a red cell, pulsing: seen at a glance on the edge of the night.
        Style::new()
            .fg(Color::Rgb(255, 255, 255))
            .bg(if (frame / COMPASS_PULSE).is_multiple_of(2) {
                COMPASS
            } else {
                COMPASS_DIM
            })
            .add_modifier(Modifier::BOLD),
    );
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
    fn the_compass_arrow_stands_on_the_edge_the_healing_lies_beyond() {
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
        assert_eq!(row.trim(), "→");
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
