//! The demolition hammer: now and then, once an enemy stands where it would land, a blow on the
//! ground just ahead of the player; its shock hurts every enemy around where it lands, throwing
//! them back; from its third level, more shocks follow one after the other, on ahead in a line;
//! to the left and the right too at the last level, the enemies stunned.

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
/// The shocks following the blow: how far apart, in tiles; the ticks between two; how far each
/// reaches, in tiles.
pub const SHOCK_GAP: f32 = 2.0;
pub const SHOCK_EVERY: u32 = 4;
pub const SHOCK_RADIUS: f32 = 1.25;

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
    /// How many shocks follow the blow, on ahead; and whether as many go left and right too.
    pub shocks: u8,
    pub sideways: bool,
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
        damage: if level >= 2 { 40 } else { 28 },
        radius: if level >= 2 { 2.5 } else { 2.0 },
        stun: if level >= 5 { second } else { 0 },
        shocks: if level >= 3 { 3 } else { 0 },
        sideways: level >= 5,
    }
}

/// A shock to come after a blow: where, and the ticks left before it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quake {
    pub at: Vec2,
    pub left: u32,
}

/// The shocks following a blow landed at `at`, the player facing `way` (length 1): `shocks` on
/// ahead, one after the other; as many to the left and the right too, `sideways`. Never back
/// towards the player.
pub fn quakes(at: Vec2, way: Vec2, shocks: u8, sideways: bool) -> Vec<Quake> {
    let across = Vec2::new(-way.y, way.x);
    let ways = if sideways {
        vec![way, across, across * -1.0]
    } else {
        vec![way]
    };
    ways.into_iter()
        .flat_map(|line| {
            (1..=shocks).map(move |shock| Quake {
                at: at + line * (SHOCK_GAP * f32::from(shock)),
                left: SHOCK_EVERY * u32::from(shock),
            })
        })
        .collect()
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
    fn its_shocks_go_on_ahead_one_after_the_other_and_aside_at_the_last_level_never_back() {
        let (at, way) = (Vec2::new(10.0, 0.0), Vec2::new(1.0, 0.0));
        let ahead = quakes(at, way, 3, false);
        let places: Vec<f32> = ahead.iter().map(|quake| quake.at.x).collect();
        assert_eq!(places, [12.0, 14.0, 16.0]);
        assert!(ahead.windows(2).all(|two| two[0].left < two[1].left));
        let all = quakes(at, way, 3, true);
        assert_eq!(all.len(), 9);
        assert!(all.iter().all(|quake| quake.at.x >= 10.0), "never back");
        assert!(
            all.iter().any(|quake| quake.at.y > 0.0) && all.iter().any(|quake| quake.at.y < 0.0)
        );
    }

    #[test]
    fn the_hammer_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!(
            (first.every, first.damage, first.radius, first.stun),
            (60, 28, 2.0, 0)
        );
        assert_eq!((first.shocks, at(2).shocks, at(3).shocks), (0, 0, 3));
        assert_eq!(
            (last.every, last.damage, last.radius, last.stun),
            (45, 40, 2.5, 30)
        );
    }
}
