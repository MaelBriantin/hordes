//! The demolition hammer: now and then, once an enemy stands where it would land, a blow on the
//! ground just ahead of the player; its shock hurts every enemy around where it lands, throwing
//! them back, stunning them at the last level.

use super::{TICKS_PER_SECOND, Vec2};

/// How far ahead of the player's middle the hammer lands, in tiles.
pub const AHEAD: f32 = 2.5;
/// How far the shock throws the enemies it hits back, away from where the hammer lands, in
/// tiles, over how many ticks: as its wave goes out.
pub const KNOCKBACK: f32 = 1.5;
pub const WAVE_TICKS: u32 = 6;
/// How long a blow shows, in ticks.
pub const BLOW_TICKS: u32 = 20;
/// How long the hammer is raised over the player's head before it falls, in ticks.
pub const RAISE_TICKS: u32 = 6;

/// The hammer at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hammer {
    /// The ticks between two blows, before Haste.
    pub every: u32,
    /// What the shock takes from each enemy it hits, before Might.
    pub damage: u32,
    /// How far around where it lands the shock reaches, in tiles.
    pub radius: f32,
    /// How long the enemies it hits stand stunned, in ticks.
    pub stun: u32,
}

/// The hammer at a level.
pub fn at(level: u8) -> Hammer {
    let second = u32::from(TICKS_PER_SECOND);
    Hammer {
        every: if level >= 4 {
            second * 3 / 2
        } else {
            second * 2
        },
        damage: if level >= 3 { 40 } else { 28 },
        radius: if level >= 2 { 2.5 } else { 2.0 },
        stun: if level >= 5 { second } else { 0 },
    }
}

/// A blow as it shows: where the hammer landed, how far its shock reached, and its age in ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blow {
    pub at: Vec2,
    pub radius: f32,
    pub age: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hammer_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!(
            (first.every, first.damage, first.radius, first.stun),
            (60, 28, 2.0, 0)
        );
        assert_eq!(
            (last.every, last.damage, last.radius, last.stun),
            (45, 40, 2.5, 30)
        );
    }
}
