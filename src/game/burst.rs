//! An abomination's death: it swells, blinking, then bursts, a disc of bile pushing back and
//! hurting everything around, the player and the enemies alike, and lets out its parasites.

use super::{Facing, Vec2};

/// The ticks an abomination swells before it bursts: 0.6 s, for the player to step away.
pub const SWELL_TICKS: u32 = 18;
/// How far the burst reaches, in tiles; how far it pushes what it reaches, and what it costs
/// it, in HP.
pub const REACH: f32 = 3.0;
pub const PUSH: f32 = 2.0;
pub const DAMAGE: u32 = 10;
/// The ticks the disc of bile spreads to its reach, then fades.
pub const SPREAD_TICKS: u32 = 6;
pub const FADE_TICKS: u32 = 6;
/// How far out of the burst its parasites are thrown, in tiles.
pub const THROW: f32 = 2.5;

/// A felled abomination, swelling before it bursts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swelling {
    pub position: Vec2,
    pub facing: Facing,
    /// Ticks left before it bursts.
    pub left: u32,
}

impl Swelling {
    /// The ticks since it started to swell.
    pub fn age(&self) -> u32 {
        SWELL_TICKS.saturating_sub(self.left)
    }
}

/// A burst's disc of bile, as it shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blast {
    pub centre: Vec2,
    /// The ticks since it burst.
    pub age: u32,
}

impl Blast {
    /// Whether it still shows.
    pub fn shows(&self) -> bool {
        self.age < SPREAD_TICKS + FADE_TICKS
    }

    /// How far it has spread, in tiles: out to its reach, then no further.
    pub fn radius(&self) -> f32 {
        REACH * share(self.age, SPREAD_TICKS)
    }

    /// How strongly it shows, from 1 (spreading) to 0 (faded).
    pub fn strength(&self) -> f32 {
        1.0 - share(self.age.saturating_sub(SPREAD_TICKS), FADE_TICKS)
    }
}

/// `part` as a share of `whole`, from 0 to 1.
fn share(part: u32, whole: u32) -> f32 {
    f32::from(u16::try_from(part.min(whole)).unwrap_or(u16::MAX))
        / f32::from(u16::try_from(whole.max(1)).unwrap_or(u16::MAX))
}
