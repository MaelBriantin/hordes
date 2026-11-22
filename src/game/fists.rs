//! The fists, the infiltrator's own: quick punches at the nearest enemy within arm's reach,
//! whichever way; every third a heavier one, throwing it back, a hook stunning it at the last
//! level.

use super::{TICKS_PER_SECOND, Vec2};

/// Every how many punches one is a heavier one; what it does, against a punch, in times.
pub const COMBO: u32 = 3;
const HEAVY_TIMES: u32 = 2;
/// How far a heavier punch throws back what it hits, in tiles, over how many ticks.
pub const KNOCKBACK: f32 = 1.0;
pub const KNOCK_TICKS: u32 = 4;
/// How long a punch shows, in ticks.
pub const PUNCH_TICKS: u32 = 5;

/// The fists at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fists {
    /// The ticks between two punches, before Haste.
    pub every: u32,
    /// What a punch takes from the enemy it hits, before Might; a heavier one, twice as much.
    pub damage: u32,
    /// How far they reach from the player's middle, in tiles.
    pub reach: f32,
    /// How long a heavier punch stuns, in ticks.
    pub stun: u32,
}

/// The fists at a level.
pub fn at(level: u8) -> Fists {
    let tenths = |count: u32| count * u32::from(TICKS_PER_SECOND) / 10;
    Fists {
        every: if level >= 4 { tenths(3) } else { tenths(4) },
        damage: if level >= 3 { 12 } else { 8 },
        reach: if level >= 2 { 1.6 } else { 1.2 },
        stun: if level >= 5 { tenths(10) } else { 0 },
    }
}

/// What the punch numbered `count` (from 1) does, a punch doing `damage`: every third, twice as
/// much.
pub fn hit(count: u32, damage: u32) -> (u32, bool) {
    let heavy = count.is_multiple_of(COMBO);
    (if heavy { damage * HEAVY_TIMES } else { damage }, heavy)
}

/// A punch as it shows: where it struck, whether a heavier one, and its age in ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Punch {
    pub at: Vec2,
    pub heavy: bool,
    pub age: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fists_grow_with_their_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.every, first.damage, first.stun), (12, 8, 0));
        assert_eq!((last.every, last.damage, last.stun), (9, 12, 30));
        assert!(last.reach > first.reach);
    }

    #[test]
    fn every_third_punch_is_a_heavier_one() {
        let punches: Vec<(u32, bool)> = (1..=6).map(|count| hit(count, 8)).collect();
        assert_eq!(
            punches,
            [
                (8, false),
                (8, false),
                (16, true),
                (8, false),
                (8, false),
                (16, true)
            ]
        );
    }
}
