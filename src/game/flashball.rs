//! The Eye-gouger, the flashball the riot officer unlocks: a slow shot at the nearest enemy, as
//! a gun's (see `guns`); the enemy hit thrown back and stunned, a felled one bursting in a spray.
//! "Non-lethal".

use super::TICKS_PER_SECOND;
use super::vec2::Vec2;

/// How far a hit throws the enemy back, in tiles, over how many ticks.
pub const THROW_TILES: f32 = 3.0;
pub const THROW_TICKS: u32 = 8;
/// How long a felled enemy's burst shows, in ticks.
pub const BURST_TICKS: u32 = 36;

/// The ticks an enemy hit stands stunned, by the flashball's level: 1 s, then 1.5 s.
pub fn stun(level: u8) -> u32 {
    let second = u32::from(TICKS_PER_SECOND);
    if level >= 4 { second * 3 / 2 } else { second }
}

/// An enemy felled by the flashball, bursting: where, the way the shot went, how long ago.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burst {
    pub at: Vec2,
    pub way: Vec2,
    pub age: u32,
}
