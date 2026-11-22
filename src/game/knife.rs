//! The combat knife, the special agent's own: quick stabs at the nearest enemy, whichever way, close
//! by; little a stab, but many.

use super::{TICKS_PER_SECOND, Vec2};

/// How long a stab shows, in ticks.
pub const STAB_TICKS: u32 = 5;

/// The combat knife at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knife {
    /// The ticks between two stabs, before Haste.
    pub every: u32,
    /// What a stab takes, before Might.
    pub damage: u32,
    /// How near the player an enemy has to be, in tiles.
    pub reach: f32,
    /// How many of the nearest enemies a stab hits.
    pub targets: usize,
}

/// The combat knife at a level.
pub fn at(level: u8) -> Knife {
    let tenths = |count: u32| count * u32::from(TICKS_PER_SECOND) / 10;
    Knife {
        every: if level >= 5 { tenths(2) } else { tenths(4) },
        damage: if level >= 3 { 12 } else { 8 },
        reach: if level >= 2 { 2.0 } else { 1.5 },
        targets: if level >= 4 { 2 } else { 1 },
    }
}

/// A stab as it shows: where it hit, and its age in ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stab {
    pub at: Vec2,
    pub age: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_knife_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.every, first.damage, first.targets), (12, 8, 1));
        assert_eq!((last.every, last.damage, last.targets), (6, 12, 2));
        assert!(last.reach > first.reach);
    }
}
