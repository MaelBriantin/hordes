//! The grenade, from the Demolisher achievement: now and then, a grenade thrown at the biggest
//! crowd near the player, arcing; where it lands, a blast hurting every enemy around and
//! throwing them back. Two at once at the last level.

use super::{TICKS_PER_SECOND, Vec2};

/// The ticks a grenade flies before it blows; how high its arc rises, in tiles.
pub const FLIGHT_TICKS: u32 = 15;
pub const ARC: f32 = 1.5;
/// How far a blast throws the enemies it hits back, in tiles, over how many ticks.
pub const KNOCKBACK: f32 = 1.5;
pub const WAVE_TICKS: u32 = 6;

/// The grenade at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grenade {
    /// The ticks between two throws, before Haste.
    pub every: u32,
    /// What a blast takes from each enemy it hits, before Might.
    pub damage: u32,
    /// How far around where it lands a blast reaches, in tiles.
    pub radius: f32,
    /// The grenades of a throw, each at a crowd of its own.
    pub count: usize,
}

/// The grenade at a level.
pub fn at(level: u8) -> Grenade {
    let second = u32::from(TICKS_PER_SECOND);
    Grenade {
        every: if level >= 4 { 2 } else { 3 } * second,
        damage: if level >= 3 { 30 } else { 20 },
        radius: if level >= 2 { 2.5 } else { 2.0 },
        count: if level >= 5 { 2 } else { 1 },
    }
}

/// A grenade in the air, from where it was thrown to where it blows, and the ticks left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thrown {
    pub from: Vec2,
    pub to: Vec2,
    pub left: u32,
}

impl Thrown {
    /// How far through its flight it is, from 0 (thrown) to 1 (blowing).
    pub fn progress(&self) -> f32 {
        let flown = FLIGHT_TICKS.saturating_sub(self.left);
        f32::from(u16::try_from(flown).unwrap_or(0))
            / f32::from(u16::try_from(FLIGHT_TICKS).unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grenade_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.every, first.damage, first.count), (90, 20, 1));
        assert_eq!((last.every, last.damage, last.count), (60, 30, 2));
        assert!(last.radius > first.radius);
    }
}
