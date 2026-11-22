//! The ninja's effects: a dark ghost of them by each enemy the katana cuts, in turn, a slash on
//! it; and, a blow dodged, a ghost of them left aside, fading.

use ratatui::style::Color;

use super::super::sprites::{self, Look, Outfit};
use super::{Canvas, pixel};
use crate::game::{Facing, Vec2, World};

/// How dark a ghost of the ninja is.
const GHOST: Color = Color::Rgb(60, 64, 90);
const SLASH: Color = Color::Rgb(245, 245, 250);
/// How long each ghost of a chain shows, one after the other a tick apart, and its slash, in
/// ticks; how far from the enemy cut it stands, in tiles.
const GHOST_TICKS: u32 = 5;
const SLASH_TICKS: u32 = 3;
const GHOST_APART: f32 = 1.5;
/// How long the ghost of a dodge shows, in ticks; how far aside it stands, in tiles.
const DODGE_TICKS: u32 = 6;
const DODGE_APART: f32 = 1.0;

impl Canvas<'_> {
    /// The katana's last chain of cuts: by each enemy cut, in turn, a dark ghost of the hero,
    /// come from the one before, and a slash across it.
    pub(super) fn paint_cuts(&mut self, world: &World, outfit: Outfit) {
        let Some(cuts) = &world.cuts else {
            return;
        };
        for (step, pair) in (0_u32..).zip(cuts.path.windows(2)) {
            let Some(age) = cuts.age.checked_sub(step) else {
                continue;
            };
            let (from, at) = (pair[0], pair[1]);
            if age < GHOST_TICKS {
                let back = (from - at).normalized();
                let facing = Facing::default().after(at - from);
                let look = Look {
                    tint: Some(GHOST),
                    ..sprites::hero(facing, outfit)
                };
                self.paint_at(look, at + back * GHOST_APART);
            }
            if age < SLASH_TICKS {
                let (x, y) = pixel(at);
                for k in -1..=2 {
                    self.set((x + k, y + 1 - k), SLASH);
                }
            }
        }
    }

    /// A blow just dodged: a dark ghost of the hero left aside, where the blow fell.
    pub(super) fn paint_dodge(&mut self, world: &World, outfit: Outfit) {
        let Some(at) = world.dodged_at else {
            return;
        };
        if world.ticks().saturating_sub(at) >= DODGE_TICKS {
            return;
        }
        let player = &world.player;
        let way = player.facing.way();
        let aside = Vec2::new(-way.y, way.x) * DODGE_APART - way * (DODGE_APART / 2.0);
        let look = Look {
            tint: Some(GHOST),
            ..sprites::hero(player.facing, outfit)
        };
        self.paint_at(look, player.position + aside);
    }
}
