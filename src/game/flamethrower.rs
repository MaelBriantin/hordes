//! The flamethrower: a cone of fire from the player, lit at the nearest enemy and burning its
//! whole tank once lit, turning after the nearest enemy as it burns; every enemy in the cone
//! burns. Empty, the tank fills again. Each level reaches further, burns harder or holds more.

use std::f32::consts::{PI, TAU};

use super::{TICKS_PER_SECOND, Vec2};

/// Half the cone's opening, in degrees: 60° in all.
const HALF_ANGLE_DEGREES: f32 = 30.0;
/// How close to the nozzle an enemy is in the cone whatever its angle, in tiles: at its foot.
const MOUTH: f32 = 0.8;
/// The ticks between two bursts of damage to the enemies in the cone.
pub const BURN_EVERY: u32 = 8;
/// How far ahead of the player's middle its nozzle is, in tiles: its fire starts there.
pub const NOZZLE: f32 = 0.9;
/// The seconds an empty tank takes to fill again, before Haste.
const REFILL_SECONDS: u32 = 3;
/// How fast its fire turns after the nearest enemy as it burns, in degrees a tick: 180° a second.
pub const TURN_DEGREES: f32 = 6.0;

/// The flamethrower at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flamethrower {
    /// How far the cone reaches, in tiles.
    pub range: f32,
    /// What each burst of damage does to each enemy in the cone, before Might.
    pub damage: u32,
    /// The ticks of fire a full tank holds.
    pub tank: u32,
}

/// The flamethrower at a level.
pub fn at(level: u8) -> Flamethrower {
    let second = u32::from(TICKS_PER_SECOND);
    Flamethrower {
        range: match level {
            0 | 1 => 4.5,
            2..=4 => 5.5,
            _ => 6.5,
        },
        damage: if level >= 3 { 3 } else { 2 },
        tank: if level >= 4 { 3 } else { 2 } * second,
    }
}

/// The ticks an empty tank takes to fill again, before Haste.
pub fn refill() -> u32 {
    REFILL_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// Whether a body (half as wide as `half`) centred on `at` is in the cone from `from` the way
/// `way` goes (length 1), `range` long.
pub fn in_cone(from: Vec2, way: Vec2, range: f32, at: Vec2, half: f32) -> bool {
    let to = at - from;
    let distance = to.length();
    if distance > range + half {
        return false;
    }
    if distance <= MOUTH {
        return true;
    }
    let along = to.x * way.x + to.y * way.y;
    // Its edge, not its middle, has to be within the opening.
    let widened =
        (HALF_ANGLE_DEGREES.to_radians() + (half / distance).atan()).min(89_f32.to_radians());
    along > 0.0 && along >= distance * widened.cos()
}

/// The way `way` (length 1) goes, turned towards `toward` the short way round, by `most`
/// radians at most.
pub fn turn(way: Vec2, toward: Vec2, most: f32) -> Vec2 {
    let (from, to) = (way.y.atan2(way.x), toward.y.atan2(toward.x));
    let apart = (to - from + PI).rem_euclid(TAU) - PI;
    let angle = from + apart.clamp(-most, most);
    Vec2::new(angle.cos(), angle.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_fire_turns_towards_the_enemy_the_short_way_and_no_faster_than_it_may() {
        let most = TURN_DEGREES.to_radians();
        let way = turn(Vec2::new(1.0, 0.0), Vec2::new(0.0, 3.0), most);
        assert!((way.y.atan2(way.x) - most).abs() < 1e-5, "{way:?}");
        let way = turn(Vec2::new(1.0, 0.0), Vec2::new(0.0, -3.0), most);
        assert!((way.y.atan2(way.x) + most).abs() < 1e-5, "{way:?}");
        let near = Vec2::new(1.0, 0.05);
        let way = turn(Vec2::new(1.0, 0.0), near, most);
        assert!((way.y.atan2(way.x) - near.y.atan2(near.x)).abs() < 1e-5);
        assert!((way.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn the_cone_reaches_ahead_within_its_opening_not_behind_nor_beyond() {
        let (from, way) = (Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
        assert!(in_cone(from, way, 3.0, Vec2::new(2.5, 0.0), 0.45));
        assert!(in_cone(from, way, 3.0, Vec2::new(2.0, 1.0), 0.45));
        assert!(
            !in_cone(from, way, 3.0, Vec2::new(1.0, 2.5), 0.45),
            "outside the opening"
        );
        assert!(
            !in_cone(from, way, 3.0, Vec2::new(-2.0, 0.0), 0.45),
            "behind"
        );
        assert!(
            !in_cone(from, way, 3.0, Vec2::new(4.0, 0.0), 0.45),
            "beyond"
        );
        assert!(
            in_cone(from, way, 3.0, Vec2::new(0.0, 0.6), 0.45),
            "at its foot"
        );
    }

    #[test]
    fn the_flamethrower_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.range, first.damage, first.tank), (4.5, 2, 60));
        assert_eq!((last.range, last.damage, last.tank), (6.5, 3, 90));
        assert_eq!(at(4).range, 5.5, "a tile further at the last level alone");
    }
}
