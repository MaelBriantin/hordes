//! The wall of fire, the exterminator's own action: their tank leaks a while, an unbroken line
//! of fire following their path, each stretch burning on a while after: walls of fire drawn
//! around a crowd or across a way. The enemies don't avoid it: they burn crossing it.

use super::TICKS_PER_SECOND;

/// How far the player goes between two stretches of the wall, in tiles: close enough for them to
/// make one line.
pub const STEP: f32 = 0.5;

/// The wall of fire at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FireWall {
    /// The ticks the fire follows the player.
    pub trail: u32,
    /// The ticks each stretch burns.
    pub lasts: u32,
    /// The ticks after it is lit before it can be again, before Haste.
    pub recharge: u32,
    /// How far each stretch spreads around the player's path, in tiles: half the wall's width.
    pub radius: f32,
}

/// The wall of fire at a level: longer walls at 2, a longer trail at 3, sooner back at 4, twice
/// as wide at 5.
pub fn at(level: u8) -> FireWall {
    let second = u32::from(TICKS_PER_SECOND);
    FireWall {
        // 1.5 s, then 2: the wall some 8 tiles long, then 11, as the exterminator goes.
        trail: if level >= 3 {
            2 * second
        } else {
            second * 3 / 2
        },
        lasts: if level >= 2 { 7 } else { 5 } * second,
        recharge: if level >= 4 { 7 } else { 10 } * second,
        radius: if level >= 5 { 1.0 } else { 0.5 },
    }
}
