//! The scout's smoke bomb: a dense grey cloud, churning, its edge ragged; the scout in it a grey
//! silhouette, for the player to see where they stand.

use ratatui::style::Color;

use super::super::sprites::{self, Look, Outfit};
use super::{Canvas, pixel};
use crate::game::{SMOKE_CLEAR_TICKS, SMOKE_SPREAD_TICKS, World};

/// The smoke's greys, its light top, its body, its shade; and the scout's silhouette in it.
const GREYS: [Color; 3] = [
    Color::Rgb(205, 205, 210),
    Color::Rgb(172, 174, 180),
    Color::Rgb(140, 142, 150),
];
const SILHOUETTE: Color = Color::Rgb(95, 97, 105);
/// How much flatter than round the cloud is: a little wider than tall.
const FLAT: f32 = 1.15;

impl Canvas<'_> {
    /// The smoke bomb's cloud, if one is up: spreading out at first, thinning out at its end;
    /// the scout in it a grey silhouette.
    pub(super) fn paint_smoke(&mut self, world: &World, outfit: Outfit, frame: u32) {
        let Some(cloud) = world.smoke else {
            return;
        };
        let (age, left) = (cloud.age(), cloud.left);
        let fraction = |part: u32, whole: u32| {
            f32::from(u16::try_from(part).unwrap_or(u16::MAX))
                / f32::from(u16::try_from(whole).unwrap_or(1))
        };
        let (grow, thick) = if age < SMOKE_SPREAD_TICKS {
            (fraction(age, SMOKE_SPREAD_TICKS), 1.0)
        } else if left < SMOKE_CLEAR_TICKS {
            let gone = fraction(SMOKE_CLEAR_TICKS - left, SMOKE_CLEAR_TICKS);
            (1.0 + 0.2 * gone, 1.0 - gone)
        } else {
            (1.0, 1.0)
        };
        let (cx, cy) = pixel(cloud.at);
        let radius = cloud.radius * 2.0 * grow;
        #[allow(clippy::cast_possible_truncation, reason = "a cloud of a few tiles")]
        let reach = radius.ceil() as i32 + 1;
        let at = |value: i32| f32::from(i16::try_from(value).unwrap_or(0));
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let (x, y) = (cx + dx, cy + dy);
                let distance = at(dx).hypot(at(dy) * FLAT);
                // A ragged edge, churning.
                let ragged = luck(x, y, frame / 4) % 3;
                let edge = radius + (at(i32::try_from(ragged).unwrap_or(1)) - 1.0) * 0.6;
                if distance > edge {
                    continue;
                }
                let luck = luck(x, y, frame / 3);
                if thick < 1.0 && fraction(luck % 100, 100) > thick {
                    continue;
                }
                let mut shade = if at(dy) < -radius / 3.0 {
                    0
                } else if luck.is_multiple_of(3) {
                    2
                } else {
                    1
                };
                if distance > radius - 1.2 {
                    shade = if luck.is_multiple_of(2) { 1 } else { 2 };
                }
                if luck.is_multiple_of(5) {
                    shade = (shade + 1) % 3;
                }
                self.set((x, y), GREYS[shade]);
            }
        }
        if grow > 0.5 && thick > 0.3 && cloud.holds(world.player.position) {
            let look =
                sprites::hero_in(world.hero_facing(), outfit, world.torch_on() == Some(false));
            self.paint_at(
                Look {
                    tint: Some(SILHOUETTE),
                    ..look
                },
                world.player.position,
            );
        }
    }
}

/// A pixel's luck at a time: the same pixel, the same time, the same luck.
fn luck(x: i32, y: i32, time: u32) -> u32 {
    x.cast_unsigned().wrapping_mul(73_856_093)
        ^ y.cast_unsigned().wrapping_mul(19_349_663)
        ^ time.wrapping_mul(83_492_791)
}
