//! The stun baton, the exterminator's own: a thrust straight ahead of the player, the way they
//! face, hitting every enemy on its line, each paralysed a while. From its third level, a strike
//! now and then is a whirlwind: the player spins round, the baton held out, hitting every enemy
//! around.

use std::f32::consts::{FRAC_PI_2, TAU};

use super::{Facing, TICKS_PER_SECOND, Vec2};

/// How wide the thrust's line is, each side of it, in tiles.
const HALF_WIDTH: f32 = 0.5;
/// How long a thrust shows, in ticks.
pub const THRUST_TICKS: u32 = 8;
/// How far around the player a whirlwind reaches, in tiles; how long the player spins, in ticks.
pub const WHIRL_RADIUS: f32 = 2.0;
pub const WHIRL_TICKS: u32 = 12;

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
    /// Every how many strikes one is a whirlwind, if any is.
    pub whirl_every: Option<u32>,
}

/// The stun baton at a level.
pub fn at(level: u8) -> Baton {
    let tenths = |count: u32| count * u32::from(TICKS_PER_SECOND) / 10;
    Baton {
        every: if level >= 4 { tenths(9) } else { tenths(12) },
        damage: if level >= 3 { 22 } else { 16 },
        reach: if level >= 2 { 3.5 } else { 3.0 },
        paralysis: if level >= 5 { tenths(20) } else { tenths(10) },
        whirl_every: match level {
            0..=2 => None,
            3 | 4 => Some(3),
            _ => Some(2),
        },
    }
}

/// Whether a body (half as wide as `half`) at `offset` from the player is on a thrust's line, the
/// way `way` goes (length 1), `reach` long.
pub fn on_line(offset: Vec2, half: f32, way: Vec2, reach: f32) -> bool {
    let along = offset.x * way.x + offset.y * way.y;
    let across = (offset.x * way.y - offset.y * way.x).abs();
    along > 0.0 && along <= reach + half && across <= HALF_WIDTH + half
}

/// A strike as it shows: the way the player faced, how far it reached, its age in ticks, and
/// whether it is a whirlwind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thrust {
    pub way: Vec2,
    pub reach: f32,
    pub age: u32,
    pub whirl: bool,
}

impl Thrust {
    /// How long it shows, in ticks.
    pub fn lasts(&self) -> u32 {
        if self.whirl {
            WHIRL_TICKS
        } else {
            THRUST_TICKS
        }
    }

    /// A whirlwind under way: where the baton points, in radians (`y` growing downwards),
    /// from the way the player faced, round as down goes to right then up then left, a whole
    /// turn over its ticks.
    pub fn whirl_angle(&self) -> Option<f32> {
        let share = f32::from(u16::try_from(self.age).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(WHIRL_TICKS).unwrap_or(1));
        (self.whirl && self.age < WHIRL_TICKS).then(|| self.way.y.atan2(self.way.x) - share * TAU)
    }
}

/// The way the player faces, the baton pointing `angle` radians round: the nearest of the four.
pub fn facing_at(angle: f32) -> Facing {
    let quarter = (angle.rem_euclid(TAU) + FRAC_PI_2 / 2.0) / FRAC_PI_2;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a quarter, 0 to 4"
    )]
    match quarter as u32 % 4 {
        0 => Facing::Right,
        1 => Facing::Down,
        2 => Facing::Left,
        _ => Facing::Up,
    }
}

/// Whether a body (half as wide as `half`) at `offset` from the player is within a whirlwind.
pub fn in_whirl(offset: Vec2, half: f32) -> bool {
    offset.length() <= WHIRL_RADIUS + half
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
        assert_eq!((first.every, first.damage, first.paralysis), (36, 16, 30));
        assert_eq!((last.every, last.damage, last.paralysis), (27, 22, 60));
        assert_eq!(
            (first.whirl_every, at(3).whirl_every, last.whirl_every),
            (None, Some(3), Some(2))
        );
    }

    #[test]
    fn a_whirlwind_turns_from_the_way_faced_down_to_right_to_up_to_left() {
        let whirl = |age| Thrust {
            way: Facing::Down.way(),
            reach: 3.0,
            age,
            whirl: true,
        };
        let facings: Vec<Facing> = [0, 3, 6, 9]
            .into_iter()
            .map(|age| facing_at(whirl(age).whirl_angle().unwrap()))
            .collect();
        assert_eq!(
            facings,
            [Facing::Down, Facing::Right, Facing::Up, Facing::Left]
        );
        assert_eq!(whirl(WHIRL_TICKS).whirl_angle(), None, "over");
    }
}
