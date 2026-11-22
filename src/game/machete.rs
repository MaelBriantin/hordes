//! The melee weapons: a swing in an arc before the player, hitting every enemy in it, once an
//! enemy is within its reach, whichever way. The machete, quick: from level 3, a swing is a slash
//! ahead then one behind, and at level 5 one ahead again. The crowbar, the weapon every run
//! starts with: slower and heavier, pushing the enemies it hits back, a backhand behind from
//! level 4, stunning them at level 5.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4};

use super::build::Weapon;
use super::{Facing, TICKS_PER_SECOND, Vec2};

/// What a slash takes from each enemy it hits, in HP.
pub const DAMAGE: u32 = 12;
/// How many ticks a slash shows.
pub const SWING_TICKS: u32 = 6;
/// The ticks between two slashes of a swing: 0.2 seconds.
pub const SLASH_GAP: u32 = 6;

/// A melee weapon at a level: how it swings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Melee {
    /// The ticks between two swings, before Haste.
    pub cooldown: u32,
    /// What a slash takes from each enemy it hits, before Might.
    pub damage: u32,
    /// How far a slash reaches from the player's centre, in tiles; half its arc, in radians.
    pub reach: f32,
    pub half_arc: f32,
    /// The slashes of a swing, ahead and behind in turn.
    pub slashes: u32,
    /// How far an enemy hit is pushed back, in tiles; how long it stands stunned, in ticks.
    pub knockback: f32,
    pub stun: u32,
}

/// A melee weapon at a level; none for a weapon that isn't one.
pub fn melee(weapon: Weapon, level: u8) -> Option<Melee> {
    let second = u32::from(TICKS_PER_SECOND);
    match weapon {
        Weapon::Machete => Some(Melee {
            cooldown: cooldown(),
            damage: DAMAGE,
            reach: reach(level),
            half_arc: half_arc(level),
            slashes: slashes(level),
            knockback: 0.0,
            stun: 0,
        }),
        // The truncheon: a blow ahead throwing back what it hits.
        Weapon::Truncheon => Some(Melee {
            cooldown: second,
            damage: if level >= 3 { 20 } else { 14 },
            reach: if level >= 2 { 2.5 } else { 2.0 },
            half_arc: if level >= 4 { FRAC_PI_2 } else { FRAC_PI_3 },
            slashes: 1,
            knockback: 1.5,
            stun: if level >= 5 { second / 2 } else { 0 },
        }),
        Weapon::Crowbar => Some(Melee {
            cooldown: second * 3 / 2,
            damage: if level >= 3 { 24 } else { 18 },
            reach: 2.5,
            half_arc: if level >= 2 { FRAC_PI_3 } else { FRAC_PI_4 },
            slashes: if level >= 4 { 2 } else { 1 },
            knockback: 1.0,
            stun: if level >= 5 { second / 2 } else { 0 },
        }),
        _ => None,
    }
}

/// A slash on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swing {
    /// The weapon it is of.
    pub weapon: Weapon,
    /// The way it swings: the way the player faced, or behind them.
    pub facing: Facing,
    /// Ticks left showing it.
    pub left: u32,
    /// How far it reaches from the player's centre, in tiles.
    pub reach: f32,
    /// Half its arc, in radians.
    pub half_arc: f32,
}

/// The ticks between two swings: 0.9 seconds.
pub fn cooldown() -> u32 {
    u32::from(TICKS_PER_SECOND) * 9 / 10
}

/// How far a slash reaches from the player's centre at a level, in tiles: 2.5, 3.5 from level 2.
pub fn reach(level: u8) -> f32 {
    if level >= 2 { 3.5 } else { 2.5 }
}

/// The slashes of a swing at a level, ahead and behind in turn: 1, 2 from level 3, 3 at level 5.
pub fn slashes(level: u8) -> u32 {
    match level {
        0..=2 => 1,
        3 | 4 => 2,
        _ => 3,
    }
}

/// Half the arc of a slash at a level, in radians: 45° (a 90° arc), 90° (180°) from level 4.
pub fn half_arc(level: u8) -> f32 {
    if level >= 4 { FRAC_PI_2 } else { FRAC_PI_4 }
}

/// Whether an enemy (half as wide as `half_width`) at `offset` from the player is within
/// `reach`.
pub fn in_reach(offset: Vec2, half_width: f32, reach: f32) -> bool {
    offset.length() <= reach + half_width
}

/// Whether an enemy at `offset` from the player is in a slash's arc towards `way`: within
/// `reach`, and within `half_arc` of its way. One right on the player is always hit.
pub fn in_arc(offset: Vec2, half_width: f32, reach: f32, way: Vec2, half_arc: f32) -> bool {
    let distance = offset.length();
    if !in_reach(offset, half_width, reach) {
        return false;
    }
    if distance <= half_width {
        return true;
    }
    let cosine = (offset.x * way.x + offset.y * way.y) / distance;
    cosine >= half_arc.cos() - 1e-4
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: Vec2 = Vec2::new(1.0, 0.0);

    #[test]
    fn a_slash_hits_before_the_player_not_beside_them() {
        let (reach, arc) = (reach(1), half_arc(1));
        assert!(in_arc(Vec2::new(2.0, 0.5), 0.45, reach, RIGHT, arc));
        assert!(!in_arc(Vec2::new(0.0, 2.0), 0.45, reach, RIGHT, arc));
        assert!(!in_arc(Vec2::new(-2.0, 0.0), 0.45, reach, RIGHT, arc));
    }

    #[test]
    fn a_slash_reaches_further_from_level_two() {
        assert!(in_arc(
            Vec2::new(2.9, 0.0),
            0.45,
            reach(1),
            RIGHT,
            half_arc(1)
        ));
        assert!(!in_arc(
            Vec2::new(3.1, 0.0),
            0.45,
            reach(1),
            RIGHT,
            half_arc(1)
        ));
        assert!(in_arc(
            Vec2::new(3.9, 0.0),
            0.45,
            reach(2),
            RIGHT,
            half_arc(2)
        ));
    }

    #[test]
    fn at_level_four_the_arc_spans_half_a_turn() {
        assert!(in_arc(
            Vec2::new(0.0, 2.0),
            0.45,
            reach(4),
            RIGHT,
            half_arc(4)
        ));
        assert!(!in_arc(
            Vec2::new(-2.0, 0.5),
            0.45,
            reach(4),
            RIGHT,
            half_arc(4)
        ));
    }

    #[test]
    fn swings_hold_more_slashes_with_the_levels() {
        assert_eq!([1, 2, 3, 4, 5].map(slashes), [1, 1, 2, 2, 3]);
    }

    #[test]
    fn the_crowbar_hits_heavy_and_slow() {
        let crowbar = melee(Weapon::Crowbar, 1).unwrap();
        let machete = melee(Weapon::Machete, 1).unwrap();
        assert!(crowbar.damage > machete.damage && crowbar.cooldown > machete.cooldown);
        assert!(crowbar.knockback > 0.0);
        assert_eq!(melee(Weapon::Crowbar, 4).unwrap().slashes, 2);
        assert!(melee(Weapon::Crowbar, 5).unwrap().stun > 0);
    }
}
