//! The night: darkness beyond the player's light, rain, lightning, and blood, painted on a
//! canvas of pixels. All of it moves on its own, from the frames drawn, and its luck is a hash of
//! numbers: the same frame always looks the same.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::pixels::{Canvas, pixel};
use super::sprites::Sprites;
use crate::game::{Lightning, Stain, Weather, stain_lasts};

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
/// The drops per thousand pixels of each weather, from the clear night to the tempest.
const RAIN_BY_WEATHER: [f32; 5] = [0.0, 4.0, 8.0, 14.0, 20.0];
/// The share of a stage of weather, at its end, over which the rain turns into the next one's.
const RAIN_TURN: f32 = 0.25;

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

/// A run's rain in that weather, `progress` through its stage, shown as `display` says: dimmer
/// than the title's, so as not to hide the horde, and turning into the next weather's at the
/// end of the stage. None when there is none to show.
pub fn run_rain(weather: Weather, progress: f32, display: RainDisplay) -> Option<Rain> {
    let stage = |weather: Weather| RAIN_BY_WEATHER[weather as usize];
    let now = stage(weather);
    let next = weather.next().map_or(now, stage);
    let turn = ((progress - (1.0 - RAIN_TURN)) / RAIN_TURN).clamp(0.0, 1.0);
    let full = now + (next - now) * turn;
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
const LIGHT_FADE: f32 = 6.0;
/// How much of its brightness a thing keeps in the dark.
const DARK: f32 = 0.1;
/// How much of its colour the night leaves a thing, and the blue it casts over it.
const SATURATION: f32 = 0.7;
const TINT: (f32, f32, f32) = (0.85, 0.9, 1.05);
const BLOOD: [Color; 2] = [Color::Rgb(115, 10, 16), Color::Rgb(155, 20, 26)];

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
    let cycle = frame / LIGHTNING_CYCLE;
    let at = noise(cycle) % (LIGHTNING_CYCLE - AFTERGLOW);
    let since = (frame % LIGHTNING_CYCLE).checked_sub(at)?;
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
        canvas.set((target.0 + offset.0, target.1 + offset.1), BOLT_GLOW);
    }
}

/// The night over what the canvas holds: its colours dimmed and cooled, fully lit only within
/// `radius` tiles of `centre` (the player's pixel), or all over as brightly as `flash`.
pub fn light(canvas: &mut Canvas, centre: (i32, i32), radius: f32, flash: f32) {
    let (left, top) = canvas.origin;
    let width = canvas.width;
    for (index, pixel) in (0..).zip(canvas.pixels.iter_mut()) {
        let Some(Color::Rgb(red, green, blue)) = *pixel else {
            continue;
        };
        let (x, y) = (left + index % width, top + index / width);
        let distance = tiles_apart((x - centre.0, y - centre.1));
        let lit = ((radius + LIGHT_FADE - distance) / LIGHT_FADE).clamp(0.0, 1.0);
        let level = (DARK + (1.0 - DARK) * lit).max(flash);
        *pixel = Some(night_colour((red, green, blue), level));
    }
}

/// Blood stains on the canvas, fading as they age, `now` the run's tick: a few pixels each,
/// fewer as time goes, scattered the same way for as long as the stain lasts.
pub fn paint_stains(canvas: &mut Canvas, stains: &[Stain], now: u32) {
    for stain in stains {
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
            canvas.set(at, BLOOD[usize::try_from(drop % 2).unwrap_or(0)]);
        }
    }
}

/// `part` as a share of `whole`, from 0 to 1.
fn share(part: u32, whole: u32) -> f32 {
    f32::from(u16::try_from(part.min(whole)).unwrap_or(u16::MAX))
        / f32::from(u16::try_from(whole.max(1)).unwrap_or(u16::MAX))
}

/// How far apart, in tiles, two pixels are that lie `(dx, dy)` pixels apart.
fn tiles_apart((dx, dy): (i32, i32)) -> f32 {
    let clamp = |delta: i32| f32::from(i16::try_from(delta).unwrap_or(i16::MAX));
    clamp(dx).hypot(clamp(dy)) / 2.0
}

/// A colour at night: less of its colour, a little bluer, at `level` of its brightness.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped between 0 and 255 first"
)]
fn night_colour((red, green, blue): (u8, u8, u8), level: f32) -> Color {
    let (red, green, blue) = (f32::from(red), f32::from(green), f32::from(blue));
    let grey = (red + green + blue) / 3.0;
    let channel = |value: f32, tint: f32| {
        ((grey + (value - grey) * SATURATION) * tint * level)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::Rgb(
        channel(red, TINT.0),
        channel(green, TINT.1),
        channel(blue, TINT.2),
    )
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
        assert_eq!(
            night_colour((200, 200, 200), 1.0),
            Color::Rgb(170, 180, 210)
        );
        assert_eq!(night_colour((200, 200, 200), 0.0), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn the_rain_follows_the_weather_and_turns_at_the_end_of_a_stage() {
        let heavy = |rain: Option<Rain>| rain.map_or(0.0, |rain| rain.per_thousand);
        assert_eq!(heavy(run_rain(Weather::Clear, 0.5, RainDisplay::Full)), 0.0);
        assert!((heavy(run_rain(Weather::Clear, 0.875, RainDisplay::Full)) - 2.0).abs() < 1e-4);
        assert!((heavy(run_rain(Weather::Rain, 0.1, RainDisplay::Full)) - 8.0).abs() < 1e-4);
        assert!((heavy(run_rain(Weather::Rain, 0.1, RainDisplay::Reduced)) - 4.0).abs() < 1e-4);
        assert!(run_rain(Weather::Tempest, 0.9, RainDisplay::Off).is_none());
    }
}
