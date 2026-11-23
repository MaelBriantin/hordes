//! The sniper rifle, from the Shadow achievement: it aims at an enemy far off, the toughest, the
//! furthest of those, a red dot on it a moment, then a heavy shot. Unlike the other guns, it
//! aims at the enemies out of the light too: it shoots in the dark.

use super::{TICKS_PER_SECOND, Vec2};

/// The ticks its red dot stays on the enemy before the shot: half a second; the ticks the bullet
/// flies to it; the ticks its impact shows.
pub const AIM_TICKS: u32 = 15;
pub const FLIGHT_TICKS: u32 = 4;
pub const IMPACT_TICKS: u32 = 14;

/// A bullet flying to the enemy aimed at: from where, to where it was, the enemy, the ticks left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shot {
    pub from: Vec2,
    pub to: Vec2,
    pub target: u32,
    pub left: u32,
}

impl Shot {
    /// How far through its flight it is, from 0 (fired) to 1 (striking).
    pub fn progress(&self) -> f32 {
        let flown = FLIGHT_TICKS.saturating_sub(self.left);
        f32::from(u16::try_from(flown).unwrap_or(0))
            / f32::from(u16::try_from(FLIGHT_TICKS).unwrap_or(1))
    }
}

/// Where a bullet struck, the way it was going (length 1), and the ticks since.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    pub at: Vec2,
    pub way: Vec2,
    pub age: u32,
}

/// The sniper rifle at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sniper {
    /// The ticks between two shots, before Haste.
    pub every: u32,
    /// What a shot takes, before Might: enough for a Shadow.
    pub damage: u32,
    /// How far it aims, in tiles.
    pub range: f32,
    /// Whether the bullet goes through every enemy on its line.
    pub pierces: bool,
}

/// The sniper rifle at a level.
pub fn at(level: u8) -> Sniper {
    let second = u32::from(TICKS_PER_SECOND);
    Sniper {
        every: if level >= 4 { 2 } else { 3 } * second,
        damage: if level >= 3 { 70 } else { 45 },
        range: if level >= 2 { 15.0 } else { 12.0 },
        pierces: level >= 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sniper_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.every, first.damage, first.pierces), (90, 45, false));
        assert_eq!((last.every, last.damage, last.pierces), (60, 70, true));
        assert!(last.range > first.range);
    }
}
