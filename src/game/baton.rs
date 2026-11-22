//! The stun baton, the exterminator's own: a thrust straight ahead of the player, the way they
//! face, hitting every enemy on its line, each paralysed a while.

use super::{TICKS_PER_SECOND, Vec2};

/// How wide the thrust's line is, each side of it, in tiles.
const HALF_WIDTH: f32 = 0.5;
/// How long a thrust shows, in ticks.
pub const THRUST_TICKS: u32 = 8;

/// The stun baton at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baton {
    /// The ticks between two thrusts, before Haste.
    pub every: u32,
    /// What a thrust takes from each enemy it hits, before Might.
    pub damage: u32,
    /// How far ahead it reaches, in tiles.
    pub reach: f32,
    /// How long the enemies it hits stand paralysed, in ticks.
    pub paralysis: u32,
}

/// The stun baton at a level.
pub fn at(level: u8) -> Baton {
    let tenths = |count: u32| count * u32::from(TICKS_PER_SECOND) / 10;
    Baton {
        every: if level >= 4 { tenths(9) } else { tenths(12) },
        damage: if level >= 3 { 14 } else { 10 },
        reach: if level >= 2 { 3.5 } else { 3.0 },
        paralysis: if level >= 5 { tenths(20) } else { tenths(10) },
    }
}

/// Whether a body (half as wide as `half`) at `offset` from the player is on a thrust's line, the
/// way `way` goes (length 1), `reach` long.
pub fn on_line(offset: Vec2, half: f32, way: Vec2, reach: f32) -> bool {
    let along = offset.x * way.x + offset.y * way.y;
    let across = (offset.x * way.y - offset.y * way.x).abs();
    along > 0.0 && along <= reach + half && across <= HALF_WIDTH + half
}

/// A thrust as it shows: the way it went, how far, and its age in ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thrust {
    pub way: Vec2,
    pub reach: f32,
    pub age: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_thrust_hits_on_its_line_ahead_not_aside_nor_behind_nor_beyond() {
        let way = Vec2::new(1.0, 0.0);
        assert!(on_line(Vec2::new(2.0, 0.3), 0.45, way, 3.0));
        assert!(!on_line(Vec2::new(2.0, 1.5), 0.45, way, 3.0), "aside");
        assert!(!on_line(Vec2::new(-1.0, 0.0), 0.45, way, 3.0), "behind");
        assert!(!on_line(Vec2::new(4.0, 0.0), 0.45, way, 3.0), "beyond");
    }

    #[test]
    fn the_baton_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!((first.every, first.damage, first.paralysis), (36, 10, 30));
        assert_eq!((last.every, last.damage, last.paralysis), (27, 14, 60));
    }
}
