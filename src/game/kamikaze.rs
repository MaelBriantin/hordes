//! The kamikaze: a zombie in the crowd as any, until it comes into the player's light (near
//! them, the torch not lit): then its head catches fire, a green flame, and it rushes at them, faster than they go; on them, or its
//! rush over, it stops a moment, the player's last chance to get away, then it blows: a
//! fireball that hurts everything around, the player and the enemies alike, and fires left
//! burning on the ground that burn the player standing in them, not the enemies. Felled
//! before, it blows at once where it falls.

use super::{TICKS_PER_SECOND, Vec2};

/// How near the player it lights its fuse, the torch not lit, in tiles, from its middle to
/// theirs; lit, as far as its light shows.
pub const DARK_REACH: f32 = 4.0;
/// The ticks it rushes at most, its fuse lit: 3 s; then the ticks it stands still before it
/// blows, once on the player or its rush over: 0.6 s.
pub const RUSH_TICKS: u32 = 90;
pub const LAST_TICKS: u32 = 18;
/// How much faster it rushes at the player: 7.5 tiles a second, faster than them.
pub const RUSH_PACE: f32 = 2.5;
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

/// The ticks its fires burn.
pub fn fire_lasts() -> u32 {
    FIRE_SECONDS * u32::from(TICKS_PER_SECOND)
}
