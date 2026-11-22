//! The night: darkness beyond the player's light, rain, lightning, and blood, painted on a
//! canvas of pixels. All of it moves on its own, from the frames drawn, and its luck is a hash of
//! numbers: the same frame always looks the same.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::pixels::{Canvas, pixel};
use super::sprites::Sprites;
use crate::game::{Lightning, STRIKE_RADIUS, STRIKE_WARNING_TICKS, Stain, Weather, stain_lasts};

/// The frames between two strikes of lightning: 5 seconds.
const LIGHTNING_CYCLE: u32 = 150;
/// How long the sky flashes on the title screen, how long the bolt shows, and how long the map
/// stays lit after a strike, fading, in frames (ticks, in a run).
const FLASH_FRAMES: u32 = 2;
const BOLT_FRAMES: u32 = 6;
const AFTERGLOW: u32 = 24;
/// How heavy the rain is: its drops per thousand pixels, their colour and their tails'.
#[derive(Debug, Clone, Copy)]
pub struct Rain {
    per_thousand: f32,
    drop: Color,
    tail: Color,
}

/// On the title screen: heavy and bright.
const TITLE_RAIN: Rain = Rain {
    per_thousand: 16.0,
    drop: Color::Rgb(150, 170, 205),
    tail: Color::Rgb(70, 80, 110),
};
/// The drops per thousand pixels of each weather, from the dusk to the tempest.
const RAIN_BY_WEATHER: [f32; 6] = [0.0, 0.0, 4.0, 8.0, 14.0, 20.0];
/// The share of a stage of weather, at its end, over which the rain turns into the next one's.
const RAIN_TURN: f32 = 0.25;

/// How blood shows: real, for fun as ketchup, mayonnaise or pesto, or not at all. The player's
/// choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloodLook {
    Red,
    Ketchup,
    Mayo,
    Pesto,
    Off,
}

impl BloodLook {
    /// Every look, in the order the settings go through them.
    const ALL: [BloodLook; 5] = [
        BloodLook::Red,
        BloodLook::Ketchup,
        BloodLook::Mayo,
        BloodLook::Pesto,
        BloodLook::Off,
    ];

    /// The next look, round and round: red, ketchup, mayo, pesto, off.
    pub fn next(self) -> Self {
        self.step(1)
    }

    /// The look before, round and round.
    pub fn previous(self) -> Self {
        self.step(Self::ALL.len() - 1)
    }

    fn step(self, by: usize) -> Self {
        let at = Self::ALL.iter().position(|look| *look == self).unwrap_or(0);
        Self::ALL[(at + by) % Self::ALL.len()]
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            BloodLook::Red => "red",
            BloodLook::Ketchup => "ketchup",
            BloodLook::Mayo => "mayo",
            BloodLook::Pesto => "pesto",
            BloodLook::Off => "off",
        }
    }

    /// The two colours its stains are made of; none when it doesn't show.
    pub(super) fn colours(self) -> Option<[Color; 2]> {
        match self {
            BloodLook::Red => Some([Color::Rgb(115, 10, 16), Color::Rgb(155, 20, 26)]),
            BloodLook::Ketchup => Some([Color::Rgb(205, 30, 22), Color::Rgb(240, 55, 35)]),
            BloodLook::Mayo => Some([Color::Rgb(225, 190, 60), Color::Rgb(245, 215, 95)]),
            BloodLook::Pesto => Some([Color::Rgb(70, 140, 45), Color::Rgb(110, 175, 60)]),
            BloodLook::Off => None,
        }
    }
}

/// How the rain shows during a run: the player's choice, for readability; the weather is the
/// same whatever it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainDisplay {
    Full,
    Reduced,
    Off,
}

impl RainDisplay {
    /// The next choice, round and round: full, reduced, off.
    pub fn next(self) -> Self {
        match self {
            RainDisplay::Full => RainDisplay::Reduced,
            RainDisplay::Reduced => RainDisplay::Off,
            RainDisplay::Off => RainDisplay::Full,
        }
    }

    /// The choice before, round and round.
    pub fn previous(self) -> Self {
        self.next().next()
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            RainDisplay::Full => "full",
            RainDisplay::Reduced => "reduced",
            RainDisplay::Off => "off",
        }
    }
}

/// How long blood spurts from the player when hurt, in ticks, and how many drops.
const SPURT_TICKS: u32 = 10;
const SPURT_DROPS: u32 = 10;

/// Blood spurting from the player at `centre`, `since` ticks after the blow, in their `look`:
/// drops flying out every way, each at its own speed, the same for the same `seed`.
pub fn paint_spurt(
    canvas: &mut Canvas,
    centre: (i32, i32),
    since: u32,
    seed: u32,
    look: BloodLook,
) {
    let Some(colours) = look.colours() else {
        return;
    };
    if since >= SPURT_TICKS {
        return;
    }
    let since = f32::from(u16::try_from(since).unwrap_or(0)) + 1.0;
    for drop in 0..SPURT_DROPS {
        let luck = noise(seed.wrapping_add(drop));
        let angle = f32::from(u16::try_from(luck % 360).unwrap_or(0)).to_radians();
        // From half a pixel to a pixel a tick.
        let pace = 0.5 + f32::from(u16::try_from((luck >> 9) % 50).unwrap_or(0)) / 100.0;
        let reach = since * pace;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a few pixels from the player"
        )]
        let at = (
            centre.0 + (angle.cos() * reach).round() as i32,
            centre.1 + (angle.sin() * reach).round() as i32,
        );
        canvas.set(at, colours[usize::try_from(drop % 2).unwrap_or(0)]);
    }
}

/// A run's rain in that weather, `next` to come, `progress` through its stage, shown as `display` says: dimmer
/// than the title's, so as not to hide the horde, and turning into the next weather's at the
/// end of the stage; less and less as the sun rises, the `daylight` there is. None when there is
/// none to show.
pub fn run_rain(
    weather: Weather,
    next: Option<Weather>,
    progress: f32,
    display: RainDisplay,
    daylight: f32,
) -> Option<Rain> {
    let stage = |weather: Weather| RAIN_BY_WEATHER[weather as usize];
    let now = stage(weather);
    let next = next.map_or(now, stage);
    let turn = ((progress - (1.0 - RAIN_TURN)) / RAIN_TURN).clamp(0.0, 1.0);
    let full = (now + (next - now) * turn) * (1.0 - daylight);
    let per_thousand = match display {
        RainDisplay::Full => full,
        RainDisplay::Reduced => full * 0.5,
        RainDisplay::Off => 0.0,
    };
    (per_thousand > 0.0).then_some(Rain {
        per_thousand,
        drop: Color::Rgb(95, 110, 140),
        tail: Color::Rgb(45, 52, 70),
    })
}

const FLASH_SKY: Color = Color::Rgb(70, 72, 95);
const BOLT: Color = Color::Rgb(255, 255, 240);
const BOLT_GLOW: Color = Color::Rgb(170, 180, 255);
/// The pixels around the ring of a hurting strike's impact.
const IMPACT_STEPS: u8 = 32;
/// How far the player's light fades into the night, past its full reach, in tiles.
const LIGHT_FADE: f32 = crate::game::TORCH_FADE;
/// An XP gem's glow: how strong at the gem, how far it reaches, in pixels; and the
/// screen's ground it shows on where nothing is drawn.
const GEM_GLOW_STRENGTH: f32 = 0.2;
/// The light of a gem on what is drawn around it: full within this many tiles, fading over
/// that many more.
const GEM_LIGHT: (f32, f32) = (0.5, 1.5);
const GEM_GLOW_REACH: i32 = 4;
/// How far a bottle bursting lights at first, in tiles: fully, then fading.
const BURST_LIGHT: (f32, f32) = (6.0, 4.0);
const SCREEN_GROUND: (u8, u8, u8) = (12, 12, 18);
/// What an enemy burnt to death leaves: ash, two greys.
const ASH: [Color; 2] = [Color::Rgb(60, 58, 56), Color::Rgb(95, 92, 88)];
/// A fire's light, against the torch's white: orange.
const FIRE_TINT: (f32, f32, f32) = (1.3, 0.95, 0.55);
/// How a strike's ring blinks while it foretells it: every so many ticks at first, and at the
/// end; and its colour then, lightning's glow, dimmed.
const WARNING_BLINK: (f32, f32) = (4.0, 2.0);
const WARNING_RING: Color = Color::Rgb(95, 105, 160);
/// How much of its brightness a thing keeps in the dark.
const DARK: f32 = 0.10;
/// How much of its colour the night leaves a thing, and the blue it casts over it.
const SATURATION: f32 = 0.7;
const TINT: (f32, f32, f32) = (0.85, 0.9, 1.05);
/// The warm light the setting sun casts over everything.
const SUNSET_TINT: (f32, f32, f32) = (1.1, 0.88, 0.72);

/// A strike of lightning as it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strike {
    /// The frames since it struck.
    since: u32,
    /// Its own luck, for the shape of its bolt.
    seed: u32,
    /// The pixel it falls on; on the title screen, somewhere along the bottom.
    target: Option<(i32, i32)>,
}

/// The title screen's strike of lightning under way at `frame`, if any: one per cycle, at a
/// moment of its own.
pub fn title_strike(frame: u32) -> Option<Strike> {
    strike_every(frame, LIGHTNING_CYCLE)
}

/// The strike of lightning under way at `frame`, if any, for lightning once every `cycle`
/// frames, at a moment of each cycle's own.
pub fn strike_every(frame: u32, cycle: u32) -> Option<Strike> {
    let cycle_length = cycle.max(AFTERGLOW + 1);
    let cycle = frame / cycle_length;
    let at = noise(cycle) % (cycle_length - AFTERGLOW);
    let since = (frame % cycle_length).checked_sub(at)?;
    (since < AFTERGLOW).then_some(Strike {
        since,
        seed: cycle,
        target: None,
    })
}

/// A run's last strike of lightning as it shows at the run's tick `now`, while it still lights
/// the map.
pub fn run_strike(lightning: Option<Lightning>, now: u32) -> Option<Strike> {
    let lightning = lightning?;
    let since = now.checked_sub(lightning.tick)?;
    (since < AFTERGLOW).then_some(Strike {
        since,
        seed: lightning.tick,
        target: Some(pixel(lightning.at)),
    })
}

/// How brightly a strike lights the whole map: fully as it falls, fading to nothing.
pub fn flash(strike: Option<Strike>) -> f32 {
    strike.map_or(0.0, |strike| 1.0 - share(strike.since, AFTERGLOW))
}

/// The title screen's stormy night over `area`, `frame` frames in: the sky flashes as it
/// strikes, the rain falls.
pub fn render_storm(buffer: &mut Buffer, area: Rect, sprites: &Sprites, frame: u32) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let strike = title_strike(frame);
    if strike.is_some_and(|strike| strike.since < FLASH_FRAMES) {
        let (left, top) = canvas.origin;
        for y in top..top + canvas.height {
            for x in left..left + canvas.width {
                canvas.set((x, y), FLASH_SKY);
            }
        }
    }
    paint_rain(&mut canvas, frame, TITLE_RAIN);
    if let Some(strike) = strike {
        paint_bolt(&mut canvas, strike);
    }
    canvas.draw(buffer, area);
}

/// Everything the canvas holds, `share` as bright.
pub fn dim(canvas: &mut Canvas, share: f32) {
    for pixel in &mut canvas.pixels {
        if let Some(Color::Rgb(red, green, blue)) = *pixel {
            let channel = |value: u8| scale_channel(value, share);
            *pixel = Some(Color::Rgb(channel(red), channel(green), channel(blue)));
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped between 0 and 255 first"
)]
pub fn scale_channel(value: u8, share: f32) -> u8 {
    (f32::from(value) * share).round().clamp(0.0, 255.0) as u8
}

/// Rain over the canvas, `frame` frames in: each drop with its own column, start and speed,
/// falling and drifting left across the screen, wrapping round.
pub fn paint_rain(canvas: &mut Canvas, frame: u32, rain: Rain) {
    let (width, height) = (canvas.width, canvas.height);
    let (left, top) = canvas.origin;
    let frame = i32::try_from(frame % 100_000).unwrap_or(0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "a count of drops, a few thousand at most"
    )]
    let drops = ((width * height) as f32 * rain.per_thousand / 1000.0) as u32;
    for drop in 0..drops {
        let luck = noise(drop);
        let speed = 2 + i32::try_from(luck % 2).unwrap_or(0);
        let start = (
            i32::try_from(luck % 997).unwrap_or(0),
            i32::try_from(noise(luck) % 997).unwrap_or(0),
        );
        let y = (start.1 + frame * speed).rem_euclid(height);
        let x = (start.0 - frame * speed / 2).rem_euclid(width);
        canvas.set((left + x, top + y), rain.drop);
        canvas.set((left + (x + 1).rem_euclid(width), top + y - 1), rain.tail);
    }
}

/// A strike's bolt, while it shows: a jagged line from the top of the canvas down to where it
/// falls, drawn towards it.
pub fn paint_bolt(canvas: &mut Canvas, strike: Strike) {
    if strike.since >= BOLT_FRAMES {
        return;
    }
    let (left, top) = canvas.origin;
    let width = u32::try_from(canvas.width.max(1)).unwrap_or(1);
    let target = strike.target.unwrap_or((
        left + i32::try_from(noise(strike.seed.wrapping_mul(7)) % width).unwrap_or(0),
        top + canvas.height - 1,
    ));
    let wander = i32::try_from(noise(strike.seed) % 21).unwrap_or(10) - 10;
    let mut x = target.0 + wander;
    for y in top..=target.1 {
        let rows_left = target.1 - y;
        let towards = (target.0 - x).signum();
        // Free to zigzag while it can still reach where it falls; then straight to it.
        x += if (target.0 - x).abs() >= rows_left {
            towards
        } else {
            let step = noise(strike.seed.wrapping_add(y.cast_unsigned()).wrapping_mul(31)) % 3;
            i32::try_from(step).unwrap_or(1) - 1
        };
        canvas.set((x, y), BOLT);
        canvas.set((x + 1, y), BOLT_GLOW);
    }
}

/// Where a hurting strike fell, while its bolt shows: a ring as wide as the ground it hurts,
/// `radius` tiles.
pub fn paint_impact(canvas: &mut Canvas, strike: Strike, radius: f32) {
    let Some(target) = strike.target.filter(|_| strike.since < BOLT_FRAMES) else {
        return;
    };
    paint_ring(canvas, target, radius, BOLT_GLOW, &[]);
}

/// Where a strike of lightning will fall, `since` ticks into its warning: the ring of the ground
/// it will hurt, blinking faster and faster, on the ground: under the pixels `figured` says
/// a figure covers.
pub fn paint_warning(canvas: &mut Canvas, at: (i32, i32), since: u32, figured: &[bool]) {
    let progress = share(since, STRIKE_WARNING_TICKS);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few ticks"
    )]
    let blink = (WARNING_BLINK.0 - (WARNING_BLINK.0 - WARNING_BLINK.1) * progress).round() as u32;
    if (since / blink.max(1)).is_multiple_of(2) {
        paint_ring(canvas, at, STRIKE_RADIUS, WARNING_RING, figured);
    }
}

/// A ring `radius` tiles around the pixel `centre`, in `colour`, but for the pixels `covered`
/// says something stands on.
fn paint_ring(
    canvas: &mut Canvas,
    centre: (i32, i32),
    radius: f32,
    colour: Color,
    covered: &[bool],
) {
    let pixels = radius * 2.0;
    for step in 0..IMPACT_STEPS {
        let angle = std::f32::consts::TAU * f32::from(step) / f32::from(IMPACT_STEPS);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a few pixels from the impact"
        )]
        let offset = (
            (angle.cos() * pixels).round() as i32,
            (angle.sin() * pixels).round() as i32,
        );
        let at = (centre.0 + offset.0, centre.1 + offset.1);
        let under = canvas
            .index(at)
            .is_some_and(|index| covered.get(index).copied().unwrap_or(false));
        if !under {
            canvas.set(at, colour);
        }
    }
}

/// Something that lights the night: fully within `radius` tiles of its `centre` pixel, fading
/// into the dark over `fade` tiles more.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub centre: (i32, i32),
    pub radius: f32,
    pub fade: f32,
    /// How warm its light, from 0 (white, the torch's) to 1 (a fire's orange).
    warmth: f32,
}

impl Light {
    /// The player's torch, lighting `radius` tiles around them.
    pub fn torch(centre: (i32, i32), radius: f32) -> Self {
        Self {
            centre,
            radius,
            fade: LIGHT_FADE,
            warmth: 0.0,
        }
    }

    /// A campfire, lighting `radius` tiles around it, warm: it fades as far as it lights.
    pub fn fire(centre: (i32, i32), radius: f32) -> Self {
        Self {
            centre,
            radius,
            fade: radius,
            warmth: 1.0,
        }
    }

    /// An XP gem: what passes by it, an enemy, is lit out of the dark, a tile and a half or so
    /// around it.
    pub fn gem(centre: (i32, i32)) -> Self {
        Self {
            centre,
            radius: GEM_LIGHT.0,
            fade: GEM_LIGHT.1,
            warmth: 0.0,
        }
    }

    /// A bottle bursting into flames: a blaze, as much of it `left` as is still to fade, from 1
    /// down to 0.
    pub fn burst(centre: (i32, i32), left: f32) -> Self {
        Self {
            centre,
            radius: BURST_LIGHT.0 * left,
            fade: BURST_LIGHT.1,
            warmth: 1.0,
        }
    }

    /// How lit it leaves a pixel `(dx, dy)` pixels from its centre, from 0 to 1.
    fn on(&self, (dx, dy): (i32, i32)) -> f32 {
        let distance = tiles_apart((dx, dy));
        ((self.radius + self.fade - distance) / self.fade.max(0.1)).clamp(0.0, 1.0)
    }

    /// How far it reaches, in pixels: two a tile.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a reach of a few dozen pixels"
    )]
    fn reach(&self) -> i32 {
        ((self.radius + self.fade) * 2.0).ceil() as i32
    }
}

/// The night over what the canvas holds: its colours dimmed and cooled, lit only by the
/// `lights` (the brightest of them where several meet), or all over as brightly as `flash`.
/// While `daylight` is left (from 0 to 1), the setting sun lights it all, warm, as much.
pub fn light(canvas: &mut Canvas, lights: &[Light], flash: f32, daylight: f32) {
    let ambient = DARK + (1.0 - DARK) * daylight;
    let look = Dusk::of(daylight);
    let (left, top) = canvas.origin;
    let (width, height) = (canvas.width, canvas.height);
    // Each light over the pixels it reaches only: the brightest where they meet, and the
    // warmest.
    let mut lit = vec![0.0_f32; canvas.pixels.len()];
    let mut warm = vec![0.0_f32; canvas.pixels.len()];
    for light in lights {
        let reach = light.reach();
        let (cx, cy) = (light.centre.0 - left, light.centre.1 - top);
        for y in (cy - reach).max(0)..(cy + reach + 1).min(height) {
            for x in (cx - reach).max(0)..(cx + reach + 1).min(width) {
                let on = light.on((x - cx, y - cy));
                let Ok(index) = usize::try_from(y * width + x) else {
                    continue;
                };
                lit[index] = lit[index].max(on);
                warm[index] = warm[index].max(on * light.warmth);
            }
        }
    }
    for ((pixel, on), warm) in canvas.pixels.iter_mut().zip(lit).zip(warm) {
        let Some(Color::Rgb(red, green, blue)) = *pixel else {
            continue;
        };
        let level = (ambient + (1.0 - ambient) * on).max(flash);
        *pixel = Some(warmed(look.colour((red, green, blue), level), warm));
    }
}

/// A colour under a fire's light, as warm as it is (from 0 to 1): more red, less blue.
fn warmed(color: Color, warm: f32) -> Color {
    let Color::Rgb(red, green, blue) = color else {
        return color;
    };
    if warm <= 0.0 {
        return color;
    }
    Color::Rgb(
        scale_channel(red, 1.0 + (FIRE_TINT.0 - 1.0) * warm),
        scale_channel(green, 1.0 + (FIRE_TINT.1 - 1.0) * warm),
        scale_channel(blue, 1.0 + (FIRE_TINT.2 - 1.0) * warm),
    )
}

/// A glow on the map: the pixel it shines from, and its colour.
pub type Glow = ((i32, i32), (u8, u8, u8));

/// A glow around each gem or cell, in its colour, added to what is there, the bare ground included:
/// strongest at the gem, fading out over `GEM_GLOW_REACH` pixels.
pub fn paint_glows(canvas: &mut Canvas, glows: &[Glow]) {
    let (left, top) = canvas.origin;
    let (width, height) = (canvas.width, canvas.height);
    let reach = GEM_GLOW_REACH;
    for &((gx, gy), colour) in glows {
        let (cx, cy) = (gx - left, gy - top);
        for y in (cy - reach).max(0)..(cy + reach + 1).min(height) {
            for x in (cx - reach).max(0)..(cx + reach + 1).min(width) {
                let distance = f32::from(i16::try_from(x - cx).unwrap_or(i16::MAX))
                    .hypot(f32::from(i16::try_from(y - cy).unwrap_or(i16::MAX)));
                let strength = GEM_GLOW_STRENGTH
                    * (1.0 - distance / (f32::from(i16::try_from(reach).unwrap_or(1)) + 1.0))
                        .max(0.0);
                if strength <= 0.0 {
                    continue;
                }
                let Ok(index) = usize::try_from(y * width + x) else {
                    continue;
                };
                let (red, green, blue) = match canvas.pixels[index] {
                    Some(Color::Rgb(red, green, blue)) => (red, green, blue),
                    _ => SCREEN_GROUND,
                };
                let add = |value: u8, glow: u8| value.saturating_add(scale_channel(glow, strength));
                canvas.pixels[index] = Some(Color::Rgb(
                    add(red, colour.0),
                    add(green, colour.1),
                    add(blue, colour.2),
                ));
            }
        }
    }
}

/// A halo around something that shines, a fire or a gun's flash: its colour added to what is
/// around it, the bare ground included, as strong as `strength` at its middle, fading out over
/// `reach` pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Halo {
    pub centre: (i32, i32),
    pub colour: (u8, u8, u8),
    pub strength: f32,
    pub reach: i32,
}

/// The halos over the canvas: where several meet, the strongest of each colour, not their sum,
/// lest a crowd on fire turn into a fog.
pub fn paint_halos(canvas: &mut Canvas, halos: &[Halo]) {
    if halos.is_empty() {
        return;
    }
    let (left, top) = canvas.origin;
    let (width, height) = (canvas.width, canvas.height);
    let mut added = vec![(0_u8, 0_u8, 0_u8); canvas.pixels.len()];
    for halo in halos {
        let (cx, cy) = (halo.centre.0 - left, halo.centre.1 - top);
        let reach = halo.reach;
        let span = f32::from(i16::try_from(reach).unwrap_or(1)) + 1.0;
        for y in (cy - reach).max(0)..(cy + reach + 1).min(height) {
            for x in (cx - reach).max(0)..(cx + reach + 1).min(width) {
                // In pixels: twice the tiles apart.
                let distance = tiles_apart((x - cx, y - cy)) * 2.0;
                let share = halo.strength * (1.0 - distance / span).max(0.0);
                if share <= 0.0 {
                    continue;
                }
                let Ok(index) = usize::try_from(y * width + x) else {
                    continue;
                };
                let (red, green, blue) = added[index];
                added[index] = (
                    red.max(scale_channel(halo.colour.0, share)),
                    green.max(scale_channel(halo.colour.1, share)),
                    blue.max(scale_channel(halo.colour.2, share)),
                );
            }
        }
    }
    for (pixel, (red, green, blue)) in canvas.pixels.iter_mut().zip(added) {
        if (red, green, blue) == (0, 0, 0) {
            continue;
        }
        let (r, g, b) = match *pixel {
            Some(Color::Rgb(r, g, b)) => (r, g, b),
            _ => SCREEN_GROUND,
        };
        *pixel = Some(Color::Rgb(
            r.saturating_add(red),
            g.saturating_add(green),
            b.saturating_add(blue),
        ));
    }
}

/// Blood stains on the canvas in their `look`, fading as they age, `now` the run's tick: a few
/// pixels each, fewer as time goes, scattered the same way for as long as the stain lasts.
pub fn paint_stains(canvas: &mut Canvas, stains: &[Stain], now: u32, look: BloodLook) {
    let blood = look.colours();
    for stain in stains {
        // Ash shows whatever the blood: it isn't blood.
        let Some(colours) = (if stain.ash { Some(ASH) } else { blood }) else {
            continue;
        };
        let lasts = stain_lasts().max(1);
        let age = now.saturating_sub(stain.tick).min(lasts);
        let count = u32::from(stain.size) * 4 * (lasts - age) / lasts;
        let centre = pixel(stain.position);
        let span = u32::from(stain.size) + 2;
        let seed =
            noise(centre.0.cast_unsigned() ^ centre.1.cast_unsigned().rotate_left(16) ^ stain.tick);
        for drop in 0..count {
            let luck = noise(seed.wrapping_add(drop));
            let offset = |luck: u32| {
                i32::try_from(luck % span).unwrap_or(0) - i32::try_from(span / 2).unwrap_or(0)
            };
            let at = (centre.0 + offset(luck), centre.1 + offset(luck >> 8));
            canvas.set(at, colours[usize::try_from(drop % 2).unwrap_or(0)]);
        }
    }
}

/// `part` as a share of `whole`, from 0 to 1.
pub fn share(part: u32, whole: u32) -> f32 {
    f32::from(u16::try_from(part.min(whole)).unwrap_or(u16::MAX))
        / f32::from(u16::try_from(whole.max(1)).unwrap_or(u16::MAX))
}

/// How far apart, in tiles, two pixels are that lie `(dx, dy)` pixels apart.
fn tiles_apart((dx, dy): (i32, i32)) -> f32 {
    let clamp = |delta: i32| f32::from(i16::try_from(delta).unwrap_or(i16::MAX));
    clamp(dx).hypot(clamp(dy)) / 2.0
}

/// How colours look as the sun sets into the night.
struct Dusk {
    saturation: f32,
    tint: (f32, f32, f32),
}

impl Dusk {
    /// The look with `daylight` left: the setting sun's at 1, the night's at 0.
    fn of(daylight: f32) -> Self {
        let mix = |night: f32, sun: f32| night + (sun - night) * daylight;
        Dusk {
            saturation: mix(SATURATION, 1.0),
            tint: (
                mix(TINT.0, SUNSET_TINT.0),
                mix(TINT.1, SUNSET_TINT.1),
                mix(TINT.2, SUNSET_TINT.2),
            ),
        }
    }

    /// A colour in this look: less of its colour, tinted, at `level` of its brightness.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped between 0 and 255 first"
    )]
    fn colour(&self, (red, green, blue): (u8, u8, u8), level: f32) -> Color {
        let (red, green, blue) = (f32::from(red), f32::from(green), f32::from(blue));
        let grey = (red + green + blue) / 3.0;
        let channel = |value: f32, tint: f32| {
            ((grey + (value - grey) * self.saturation) * tint * level)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Color::Rgb(
            channel(red, self.tint.0),
            channel(green, self.tint.1),
            channel(blue, self.tint.2),
        )
    }
}

/// A number scrambled from another, always the same: the night's luck, without a generator.
fn noise(n: u32) -> u32 {
    let mut n = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    n ^= n >> 15;
    n = n.wrapping_mul(0x2C1B_3C6D);
    n ^ (n >> 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lightning_strikes_once_a_cycle() {
        let strikes = (0..LIGHTNING_CYCLE * 4)
            .filter(|frame| title_strike(*frame).is_some_and(|strike| strike.since == 0))
            .count();
        assert_eq!(strikes, 4);
    }

    #[test]
    fn the_dark_dims_and_cools_a_colour() {
        let night = Dusk::of(0.0);
        assert_eq!(
            night.colour((200, 200, 200), 1.0),
            Color::Rgb(170, 180, 210)
        );
        assert_eq!(night.colour((200, 200, 200), 0.0), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn the_setting_sun_warms_a_colour() {
        assert_eq!(
            Dusk::of(1.0).colour((200, 200, 200), 1.0),
            Color::Rgb(220, 176, 144)
        );
    }

    #[test]
    fn the_rain_follows_the_weather_and_turns_at_the_end_of_a_stage() {
        let heavy = |rain: Option<Rain>| rain.map_or(0.0, |rain| rain.per_thousand);
        assert_eq!(
            heavy(run_rain(
                Weather::Clear,
                Weather::Clear.next(),
                0.5,
                RainDisplay::Full,
                0.0
            )),
            0.0
        );
        assert!(
            (heavy(run_rain(
                Weather::Clear,
                Weather::Clear.next(),
                0.875,
                RainDisplay::Full,
                0.0
            )) - 2.0)
                .abs()
                < 1e-4
        );
        assert!(
            (heavy(run_rain(
                Weather::Rain,
                Weather::Rain.next(),
                0.1,
                RainDisplay::Full,
                0.0
            )) - 8.0)
                .abs()
                < 1e-4
        );
        assert!(
            (heavy(run_rain(
                Weather::Rain,
                Weather::Rain.next(),
                0.1,
                RainDisplay::Reduced,
                0.0
            )) - 4.0)
                .abs()
                < 1e-4
        );
        assert!(
            run_rain(
                Weather::Tempest,
                Weather::Tempest.next(),
                0.9,
                RainDisplay::Off,
                0.0
            )
            .is_none()
        );
    }
}
