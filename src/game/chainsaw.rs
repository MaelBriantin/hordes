//! The chainsaw: what touches it ahead of the player, the way they face, ground on and on, a bite
//! every few ticks, and slowed while it is.

use std::f32::consts::FRAC_PI_4;

/// The ticks between two bites, before Haste.
pub const BITE_TICKS: u32 = 6;
/// How much slower an enemy the saw grinds goes, in percent; and for how many ticks once out of
/// it.
pub const SLOW_PERCENT: u32 = 50;
pub const SLOW_TICKS: u32 = 2;

/// The chainsaw at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chainsaw {
    /// What a bite takes from each enemy it grinds, before Might.
    pub damage: u32,
    /// How far ahead it reaches, in tiles; half its arc, in radians.
    pub reach: f32,
    pub half_arc: f32,
}

/// The chainsaw at a level.
pub fn at(level: u8) -> Chainsaw {
    Chainsaw {
        damage: match level {
            0..=2 => 4,
            3 | 4 => 6,
            _ => 8,
        },
        reach: if level >= 2 { 2.0 } else { 1.5 },
        half_arc: if level >= 4 {
            FRAC_PI_4 * 5.0 / 3.0
        } else {
            FRAC_PI_4
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chainsaw_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.damage, first.reach), (4, 1.5));
        assert_eq!((last.damage, last.reach), (8, 2.0));
        assert!(last.half_arc > first.half_arc);
    }
}
