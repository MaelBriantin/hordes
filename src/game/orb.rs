//! Orb: orbs circling the player, hurting the enemies they pass through and pushing them back.
//! An enemy an orb has hit can't be hit by an orb again for a while.

use std::f32::consts::TAU;

use super::{TICKS_PER_SECOND, Vec2, tick_seconds};

/// Half the width of an orb, in tiles: 3 pixels across.
pub const HALF_WIDTH: f32 = 0.75;
/// How far an orb pushes the enemy it hits, in tiles.
pub const KNOCKBACK: f32 = 0.5;

/// The orbs at a level: 1, 2 from level 2, 3 from level 4.
pub fn count(level: u8) -> u8 {
    match level {
        0 => 0,
        1 => 1,
        2 | 3 => 2,
        _ => 3,
    }
}

/// What an orb takes from an enemy, in HP, at a level: 8, 12 from level 3.
pub fn damage(level: u8) -> u32 {
    if level >= 3 { 12 } else { 8 }
}

/// How far from the player the orbs circle, in tiles: 2.5, 3.5 at level 5.
fn radius(level: u8) -> f32 {
    if level >= 5 { 3.5 } else { 2.5 }
}

/// How far the orbs turn each tick, in radians: a turn in 2 seconds, 1.5 at level 5.
pub fn turn_per_tick(level: u8) -> f32 {
    let seconds_per_turn = if level >= 5 { 1.5 } else { 2.0 };
    TAU / seconds_per_turn * tick_seconds()
}

/// The ticks before an orb can hit the same enemy again: half a second.
pub fn rehit() -> u32 {
    u32::from(TICKS_PER_SECOND) / 2
}

/// Where the orbs are at a level, around `centre`, the first at `angle`, the others evenly
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
    fn orbs_are_evenly_spaced_on_their_circle() {
        let centre = Vec2::new(10.0, 10.0);
        let orbs = positions(centre, 4, 0.0);
        assert_eq!(orbs.len(), 3);
        for orb in &orbs {
            assert!(((*orb - centre).length() - 2.5).abs() < 1e-4);
        }
        assert!((orbs[0].x - 12.5).abs() < 1e-4);
        let apart = (orbs[0] - orbs[1]).length();
        assert!((apart - (orbs[1] - orbs[2]).length()).abs() < 1e-4);
    }

    #[test]
    fn level_five_widens_the_circle() {
        let orbs = positions(Vec2::ZERO, 5, 0.0);
        assert!((orbs[0].length() - 3.5).abs() < 1e-4);
        assert!(turn_per_tick(5) > turn_per_tick(1));
    }
}
