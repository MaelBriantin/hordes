//! Blood stains: where an enemy fell, or the player was hit. They fade away with time. The game
//! only keeps where and when; how they show (or don't) is the screen's choice.

use super::{TICKS_PER_SECOND, Vec2};

/// The most stains at once: past it, the oldest go.
pub const MAX_STAINS: usize = 400;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stain {
    pub position: Vec2,
    /// The tick it was left at.
    pub tick: u32,
    /// How big it is: 1 for a scratch, 6 for an abomination.
    pub size: u8,
    /// Ash rather than blood: what an enemy burnt to death leaves.
    pub ash: bool,
}

/// How long a stain lasts, in ticks: 30 seconds.
pub fn lasts() -> u32 {
    30 * u32::from(TICKS_PER_SECOND)
}
