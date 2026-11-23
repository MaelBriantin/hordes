//! The defibrillator, the medic's own: a shock at the nearest enemy, leaping on to the next ones
//! near it; each enemy it fells gives the medic back some HP.

use super::{TICKS_PER_SECOND, Vec2};

/// How near the player the first enemy shocked has to be, and how near each next one to the
/// last, in tiles.
pub const REACH: f32 = 2.0;
pub const LEAP: f32 = 2.0;
/// How long a shock shows, in ticks.
pub const SHOCK_TICKS: u32 = 5;

/// The defibrillator at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Defibrillator {
    /// The ticks between two shocks, before Haste.
    pub every: u32,
    /// What a shock takes from each enemy it goes through, before Might.
    pub damage: u32,
    /// How many enemies a shock goes through, at most.
    pub chain: usize,
    /// The share of an HP each enemy it fells gives back, in percent: half an HP, a whole one at
    /// the last level.
    pub heal_percent: u32,
}

/// The defibrillator at a level.
pub fn at(level: u8) -> Defibrillator {
    let tenths = |count: u32| count * u32::from(TICKS_PER_SECOND) / 10;
    Defibrillator {
        every: if level >= 4 { tenths(8) } else { tenths(10) },
        damage: if level >= 3 { 16 } else { 12 },
        chain: if level >= 2 { 4 } else { 3 },
        heal_percent: if level >= 5 { 100 } else { 50 },
    }
}

/// A shock as it shows: from the player through each enemy it went through, and its age.
#[derive(Debug, Clone, PartialEq)]
pub struct Shock {
    pub path: Vec<Vec2>,
    pub age: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defibrillator_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!(
            (first.every, first.damage, first.chain, first.heal_percent),
            (30, 12, 3, 50)
        );
        assert_eq!(
            (last.every, last.damage, last.chain, last.heal_percent),
            (24, 16, 4, 100)
        );
    }
}
