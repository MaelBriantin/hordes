//! The katana, the ninja's own: a cut at the nearest enemy, then on to the next one near it, and
//! the next, through the crowd.

use super::{TICKS_PER_SECOND, Vec2};

/// How near the player the first enemy cut has to be, and how near each next one to the last,
/// in tiles.
pub const REACH: f32 = 1.8;
pub const LEAP: f32 = 2.5;
/// How much less each cut of a chain takes than the one before, in percent of the first, and the
/// least a cut takes: 100%, 75%, 50%, 25%, then 25% on.
const FADE_PERCENT: u32 = 25;
const LEAST_PERCENT: u32 = 25;
/// How long a chain of cuts shows, in ticks.
pub const CUT_TICKS: u32 = 12;

/// The katana at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Katana {
    /// The ticks between two chains of cuts, before Haste.
    pub every: u32,
    /// What a cut takes, before Might.
    pub damage: u32,
    /// How many enemies a chain cuts, at most.
    pub chain: usize,
}

/// The katana at a level.
pub fn at(level: u8) -> Katana {
    let tenths = |count: u32| count * u32::from(TICKS_PER_SECOND) / 10;
    Katana {
        every: if level >= 4 { tenths(6) } else { tenths(8) },
        damage: if level >= 3 { 20 } else { 14 },
        chain: match level {
            0 | 1 => 3,
            2..=4 => 4,
            _ => 6,
        },
    }
}

/// A chain of cuts as it shows: from the player through each enemy cut, in turn, and its age in
/// ticks.
#[derive(Debug, Clone, PartialEq)]
pub struct Cuts {
    pub path: Vec<Vec2>,
    pub age: u32,
}

/// What the cut at `step` of a chain (from 0) takes, the first taking `damage`.
pub fn cut_damage(damage: u32, step: usize) -> u32 {
    let step = u32::try_from(step).unwrap_or(u32::MAX);
    let percent = 100_u32
        .saturating_sub(FADE_PERCENT.saturating_mul(step))
        .max(LEAST_PERCENT);
    damage * percent / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_cut_of_a_chain_takes_less_than_the_one_before_a_quarter_at_least() {
        let cuts: Vec<u32> = (0..6).map(|step| cut_damage(20, step)).collect();
        assert_eq!(cuts, [20, 15, 10, 5, 5, 5]);
    }

    #[test]
    fn the_katana_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.every, first.damage, first.chain), (24, 14, 3));
        assert_eq!((last.every, last.damage, last.chain), (18, 20, 6));
    }
}
