//! Drones: drones circling the player, each shooting at the nearest enemy within its reach, again
//! and again, nothing breaking them; their shots come from one magazine, a few each, then they
//! reload.

use std::f32::consts::TAU;

use super::build::Weapon;
use super::guns::Gun;
use super::{TICKS_PER_SECOND, Vec2, tick_seconds};

/// The shots each drone brings to their magazine.
const SHOTS_PER_DRONE: u32 = 6;
/// How near a drone an enemy has to be to be shot at, in tiles; how fast its bullets fly, in
/// tiles a second, and how far.
const REACH: f32 = 6.0;
const BULLET_SPEED: f32 = 16.0;
const BULLET_RANGE: f32 = 7.0;
/// The ticks between two shots of a drone, before Haste: 0.4 s.
const INTERVAL_TENTHS: u32 = 4;

/// The drones at a level: 1, 2 from level 2, 3 from level 4, 4 at level 5.
pub fn count(level: u8) -> u8 {
    match level {
        0 => 0,
        1 => 1,
        2 | 3 => 2,
        4 => 3,
        _ => 4,
    }
}

/// What a drone's bullet takes from an enemy, in HP, at a level: 4, 6 from level 3.
pub fn damage(level: u8) -> u32 {
    if level >= 3 { 6 } else { 4 }
}

/// A drone as a gun: what it shoots, how often, how far.
pub fn gun(level: u8) -> Gun {
    Gun {
        weapon: Weapon::Drones,
        magazine: SHOTS_PER_DRONE * u32::from(count(level)),
        interval: INTERVAL_TENTHS * u32::from(TICKS_PER_SECOND) / 10,
        reload: reload(level),
        damage: damage(level),
        bullets: 1,
        pellets: 1,
        spread: 0.0,
        pierce: 0,
        ricochets: 0,
        speed: BULLET_SPEED,
        range: BULLET_RANGE,
    }
}

/// How near a drone an enemy has to be for the drone to shoot at it, in tiles.
pub fn reach() -> f32 {
    REACH
}

/// How far from the player the drones circle, in tiles: 2.5, 3.5 at level 5.
fn radius(level: u8) -> f32 {
    if level >= 5 { 3.5 } else { 2.5 }
}

/// How far the drones turn each tick, in radians: a turn in 2 seconds, 1.5 at level 5.
pub fn turn_per_tick(level: u8) -> f32 {
    let seconds_per_turn = if level >= 5 { 1.5 } else { 2.0 };
    TAU / seconds_per_turn * tick_seconds()
}

/// The ticks the drones take to reload their magazine once empty: 3 seconds, 2 from level 4.
pub fn reload(level: u8) -> u32 {
    let seconds = if level >= 4 { 2 } else { 3 };
    seconds * u32::from(TICKS_PER_SECOND)
}

/// Where the drones are at a level, around `centre`, the first at `angle`, the others evenly
/// spaced after it.
pub fn positions(centre: Vec2, level: u8, angle: f32) -> Vec<Vec2> {
    let count = count(level);
    (0..count)
        .map(|index| {
            let angle = angle + TAU * f32::from(index) / f32::from(count);
            centre + Vec2::new(angle.cos(), angle.sin()) * radius(level)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drones_are_evenly_spaced_on_their_circle() {
        let centre = Vec2::new(10.0, 10.0);
        let drones = positions(centre, 4, 0.0);
        assert_eq!(drones.len(), 3);
        for drone in &drones {
            assert!(((*drone - centre).length() - 2.5).abs() < 1e-4);
        }
        assert!((drones[0].x - 12.5).abs() < 1e-4);
        let apart = (drones[0] - drones[1]).length();
        assert!((apart - (drones[1] - drones[2]).length()).abs() < 1e-4);
    }

    #[test]
    fn level_five_widens_the_circle() {
        let drones = positions(Vec2::ZERO, 5, 0.0);
        assert!((drones[0].length() - 3.5).abs() < 1e-4);
        assert!(turn_per_tick(5) > turn_per_tick(1));
    }
}
