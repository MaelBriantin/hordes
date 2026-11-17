//! The game: its rules and its state. No terminal and no clock here: the app moves it on one tick
//! at a time with what the player holds down, and the ui draws what it holds.

mod body;
mod bolt;
mod build;
mod enemy;
mod facing;
mod gem;
mod level;
mod map;
mod spawn;
mod vec2;
mod world;

pub use build::{Choice, MAX_LEVEL, Passive, Weapon};
pub use enemy::{Enemy, EnemyKind};
pub use facing::Facing;
pub use level::xp_to_next;
pub use map::{Map, OBSTACLE_SIZE, Obstacle};
pub use vec2::Vec2;
pub use world::{Input, World};

/// How many ticks the world moves on per second.
pub const TICKS_PER_SECOND: u16 = 30;

/// The seconds one tick lasts.
fn tick_seconds() -> f32 {
    1.0 / f32::from(TICKS_PER_SECOND)
}
