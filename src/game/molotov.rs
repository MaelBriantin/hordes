//! The Molotov: now and then, a bottle thrown at an enemy nearby, bursting into a puddle of fire
//! where it lands; every enemy in the puddle catches fire, and burns on once out of it.

use super::{TICKS_PER_SECOND, Vec2};

/// How near the player an enemy has to be for a bottle to be thrown at it, in tiles.
pub const RANGE: f32 = 7.0;
/// The ticks a bottle flies before it bursts.
pub const FLIGHT_TICKS: u32 = 12;

/// The Molotov at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Molotov {
    /// The ticks between two throws, before Haste.
    pub every: u32,
    /// The bottles of a throw, each at an enemy of its own.
    pub bottles: usize,
    /// How far a puddle burns around where it lands, in tiles.
    pub radius: f32,
    /// The ticks a puddle burns.
    pub lasts: u32,
}

/// The Molotov at a level.
pub fn at(level: u8) -> Molotov {
    let second = u32::from(TICKS_PER_SECOND);
    Molotov {
        every: if level >= 5 { 2 } else { 3 } * second,
        bottles: if level >= 3 { 2 } else { 1 },
        radius: if level >= 2 { 2.0 } else { 1.5 },
        lasts: if level >= 4 { 5 } else { 3 } * second,
    }
}

/// A bottle in the air, from where it was thrown to where it bursts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bottle {
    pub from: Vec2,
    pub to: Vec2,
    /// Ticks left before it bursts.
    pub left: u32,
    /// The puddle it bursts into.
    pub radius: f32,
    pub lasts: u32,
}

impl Bottle {
    /// How far through its flight it is, from 0 (thrown) to 1 (bursting).
    pub fn progress(&self) -> f32 {
        let flown = FLIGHT_TICKS.saturating_sub(self.left);
        f32::from(u16::try_from(flown).unwrap_or(0))
            / f32::from(u16::try_from(FLIGHT_TICKS).unwrap_or(1))
    }
}

/// A puddle of fire on the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Puddle {
    pub centre: Vec2,
    pub radius: f32,
    /// Ticks left it burns, out of how many in all.
    pub left: u32,
    pub lasts: u32,
    /// Whether it burst from a bottle, in a flash; a fire of the flamethrower's trail doesn't.
    pub burst: bool,
}

impl Puddle {
    /// The ticks since it burst.
    pub fn age(&self) -> u32 {
        self.lasts.saturating_sub(self.left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_molotov_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.bottles, first.every), (1, 90));
        assert!((first.radius - 1.5).abs() < f32::EPSILON);
        assert_eq!((last.bottles, last.every, last.lasts), (2, 60, 150));
    }
}
