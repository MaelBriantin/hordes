//! The smoke bomb, the scout's own action: a cloud at their feet a while. Inside it, the scout
//! is lost to the enemies, even with the torch on: they lose track of them, but for those right
//! on them. At the last level, the enemies in it go slower.

use super::{TICKS_PER_SECOND, Vec2};

/// The ticks the cloud takes to spread, and to clear at its end: its look alone.
pub const SPREAD_TICKS: u32 = 12;
pub const CLEAR_TICKS: u32 = 21;
/// How much slower the enemies in the cloud go at the last level, in percent.
pub const SLOW_PERCENT: u32 = 30;

/// The smoke bomb at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Smoke {
    /// How far the cloud reaches around where it was thrown, in tiles.
    pub radius: f32,
    /// The ticks it stays.
    pub lasts: u32,
    /// The ticks after it is thrown before another can be, before Haste.
    pub recharge: u32,
    /// Whether the enemies in it go slower.
    pub slows: bool,
}

/// The smoke bomb at a level: longer at 2, wider at 3, sooner back at 4, slowing at 5.
pub fn at(level: u8) -> Smoke {
    let second = u32::from(TICKS_PER_SECOND);
    Smoke {
        radius: if level >= 3 { 4.0 } else { 3.0 },
        lasts: if level >= 2 { 5 } else { 4 } * second,
        recharge: if level >= 4 { 11 } else { 15 } * second,
        slows: level >= 5,
    }
}

/// A cloud of smoke on the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cloud {
    pub at: Vec2,
    pub radius: f32,
    /// Ticks left it stays, out of how many in all.
    pub left: u32,
    pub lasts: u32,
    pub slows: bool,
}

impl Cloud {
    /// Whether `point` is in it.
    pub fn holds(&self, point: Vec2) -> bool {
        (point - self.at).length() <= self.radius
    }

    /// The ticks since it was thrown.
    pub fn age(&self) -> u32 {
        self.lasts.saturating_sub(self.left)
    }
}
