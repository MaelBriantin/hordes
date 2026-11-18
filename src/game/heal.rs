//! Healing found on the map: herbs, picked up as the player walks over them, and campfires,
//! which heal the player staying by them until the fire has given all it had and dies out.

use super::TICKS_PER_SECOND;

/// What a herb gives back, in HP.
pub const HERB_HP: u32 = 20;
/// How near a herb's middle the player picks it up, in tiles.
pub const HERB_REACH: f32 = 0.9;
/// All the HP a campfire gives before it dies out: 8 seconds of healing.
pub const CAMPFIRE_HP: u32 = 24;
/// How near a campfire's middle the player is healed by it, in tiles.
pub const CAMPFIRE_REACH: f32 = 2.0;
/// How far a campfire lights the night at its brightest, in tiles; it shrinks as the fire
/// gives its HP.
pub const CAMPFIRE_LIGHT: f32 = 6.0;

/// The ticks for each HP a campfire gives: 3 HP a second.
pub fn ticks_per_hp() -> u32 {
    u32::from(TICKS_PER_SECOND) / 3
}

/// How far a campfire with `left` HP still to give lights the night, in tiles: less as it
/// gives, never under a tile and a half while it burns; nothing once it is out.
pub fn light(left: u32) -> f32 {
    if left == 0 {
        return 0.0;
    }
    let share = f32::from(u16::try_from(left).unwrap_or(u16::MAX))
        / f32::from(u16::try_from(CAMPFIRE_HP).unwrap_or(1));
    (CAMPFIRE_LIGHT * share).max(1.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_campfires_light_shrinks_as_it_gives() {
        assert!((light(CAMPFIRE_HP) - CAMPFIRE_LIGHT).abs() < 1e-4);
        assert!((light(CAMPFIRE_HP / 2) - CAMPFIRE_LIGHT / 2.0).abs() < 1e-4);
        assert!((light(1) - 1.5).abs() < 1e-4);
        assert!(light(0).abs() < f32::EPSILON);
    }
}
