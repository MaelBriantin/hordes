//! The C4, the demolisher's own action: Space sets a charge down at their feet, its light
//! blinking faster and faster; Space again blows it, from afar, or it blows by itself a while
//! later. Its blast hurts every enemy around, throwing them back. At the last level, two charges
//! set down one after the other, blowing together.

use super::{TICKS_PER_SECOND, Vec2};

/// The ticks a charge waits before it blows by itself: 3 s.
pub const FUSE_TICKS: u32 = 90;
/// How far the blast throws the enemies it hits back, away from the charge, in tiles, over how
/// many ticks.
pub const KNOCKBACK: f32 = 3.0;
pub const WAVE_TICKS: u32 = 6;
/// How long a blast shows, in ticks: its flash, its fireball, then its smoke.
pub const BLAST_TICKS: u32 = 30;

/// The C4 at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct C4 {
    /// The ticks after a blast before a charge can be set down again, before Haste.
    pub recharge: u32,
    /// What the blast takes from each enemy it hits, before Might.
    pub damage: u32,
    /// How far around the charge the blast reaches, in tiles.
    pub radius: f32,
    /// How many charges can stand at once, blowing together.
    pub charges: usize,
}

/// The C4 at a level.
pub fn at(level: u8) -> C4 {
    let second = u32::from(TICKS_PER_SECOND);
    C4 {
        recharge: if level >= 4 { 4 } else { 5 } * second,
        damage: if level >= 3 { 40 } else { 25 },
        radius: if level >= 2 { 3.0 } else { 2.5 },
        charges: if level >= 5 { 2 } else { 1 },
    }
}

/// A charge set down, and the ticks left before it blows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charge {
    pub at: Vec2,
    pub left: u32,
}

impl Charge {
    /// Whether its light is on: blinking faster and faster as the blast comes.
    pub fn lit(&self) -> bool {
        let half = if self.left <= 8 {
            1
        } else if self.left <= 20 {
            2
        } else {
            5
        };
        (self.left / half).is_multiple_of(2)
    }
}

/// A blast as it shows: where the charge was, how far it reached, and its age in ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blast {
    pub at: Vec2,
    pub radius: f32,
    pub age: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_c4_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!(
            (first.recharge, first.damage, first.radius, first.charges),
            (150, 25, 2.5, 1)
        );
        assert_eq!(
            (last.recharge, last.damage, last.radius, last.charges),
            (120, 40, 3.0, 2)
        );
    }

    #[test]
    fn its_light_blinks_faster_as_the_blast_comes() {
        let switches = |ticks: std::ops::Range<u32>| {
            ticks
                .clone()
                .zip(ticks.skip(1))
                .filter(|&(a, b)| {
                    Charge {
                        at: Vec2::ZERO,
                        left: a,
                    }
                    .lit()
                        != Charge {
                            at: Vec2::ZERO,
                            left: b,
                        }
                        .lit()
                })
                .count()
        };
        assert!(switches(1..9) > switches(21..29));
    }
}
