//! The shield ram, from the Hold the line achievement: the player braced a moment, then a charge
//! straight ahead, shield first, nothing hurting the player meanwhile; every enemy in the way hurt, stunned a moment and thrown aside,
//! to its side of the charge and a little on: the crowd parting in a V.

use super::{TICKS_PER_SECOND, Vec2};

/// How far the throws go, in tiles, over how many ticks; how much of a throw goes on the way
/// of the charge, against aside.
pub const THROW: f32 = 4.0;
pub const THROW_TICKS: u32 = 8;
const ON: f32 = 0.3;
/// The ticks the player stands braced before charging: 0.3 s.
pub const WINDUP_TICKS: u32 = 9;
/// How near the player an enemy is hit by the charge, beyond both their bodies, in tiles.
pub const REACH: f32 = 1.0;

/// The ram at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ram {
    /// The ticks before it can be played again, before Haste.
    pub recharge: u32,
    /// What it takes from each enemy in the way, before Might.
    pub damage: u32,
    /// How far the charge goes, in tiles.
    pub tiles: f32,
    /// How long the enemies it hits stand stunned, in ticks.
    pub stun: u32,
}

/// The ram at a level.
pub fn at(level: u8) -> Ram {
    let second = u32::from(TICKS_PER_SECOND);
    Ram {
        recharge: if level >= 2 { 6 } else { 8 } * second,
        damage: if level >= 3 { 30 } else { 20 },
        tiles: if level >= 4 { 7.0 } else { 5.0 },
        stun: if level >= 5 { second } else { second * 3 / 5 },
    }
}

/// The way an enemy at `offset` from the player is thrown by a charge going `way` (length 1):
/// aside, to its side of the charge, and a little on; length 1.
pub fn throw(way: Vec2, offset: Vec2) -> Vec2 {
    let across = Vec2::new(-way.y, way.x);
    let side = if offset.x * across.x + offset.y * across.y >= 0.0 {
        1.0
    } else {
        -1.0
    };
    (across * side + way * ON).normalized()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ram_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.recharge, first.damage, first.stun), (240, 20, 18));
        assert_eq!((last.recharge, last.damage, last.stun), (180, 30, 30));
        assert!(last.tiles > first.tiles);
    }

    #[test]
    fn the_crowd_parts_in_a_v_each_to_its_side_and_on() {
        let way = Vec2::new(1.0, 0.0);
        let left = throw(way, Vec2::new(0.5, -0.3));
        let right = throw(way, Vec2::new(0.5, 0.3));
        assert!(left.y < 0.0 && right.y > 0.0, "each to its side");
        assert!(left.x > 0.0 && right.x > 0.0, "and on");
    }
}
