//! The kamikaze: a zombie, its skin orange, coming at the player as any; on them, it stops a
//! moment, its skin lit, the player's last chance to get away, then it blows: a fireball that
//! hurts everything around, the player and the enemies alike, and fires left burning on the
//! ground that burn the player standing in them, not the enemies. Felled before, it blows at
//! once where it falls.

use super::{TICKS_PER_SECOND, Vec2};

/// The ticks it stands still before it blows, once on the player: 0.6 s.
pub const LAST_TICKS: u32 = 18;
/// How far around it its blast reaches, in tiles; what it takes from each it reaches, the
/// player and the enemies alike, in HP; how far it throws them, in tiles.
pub const RADIUS: f32 = 2.5;
pub const DAMAGE: u32 = 20;
pub const PUSH: f32 = 0.5;
/// The fires its blast leaves on the ground, around where it blew, in tiles; how far each
/// spreads, in tiles, and how long it burns.
pub const FIRES: [Vec2; 4] = [
    Vec2::new(-1.5, -0.5),
    Vec2::new(1.0, -1.5),
    Vec2::new(1.5, 1.0),
    Vec2::new(-0.5, 1.5),
];
pub const FIRE_RADIUS: f32 = 0.6;
const FIRE_SECONDS: u32 = 3;
/// What standing in its fire costs the player, in HP, each time they can be hurt again.
pub const FIRE_DAMAGE: u32 = 2;

/// Whether a kamikaze, its fuse `fuse` ticks from blowing, shows lit: its fuse burning.
pub fn lit(fuse: u32) -> bool {
    fuse > 0
}

/// The ticks its fires burn.
pub fn fire_lasts() -> u32 {
    FIRE_SECONDS * u32::from(TICKS_PER_SECOND)
}
