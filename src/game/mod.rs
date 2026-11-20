//! The game: its rules and its state. No terminal and no clock here: the app moves it on one tick
//! at a time with what the player holds down, and the ui draws what it holds.

mod announce;
mod body;
mod build;
mod burst;
mod dash;
mod drones;
mod enemy;
mod facing;
mod gem;
mod guns;
mod heal;
mod horde;
mod leaderboard;
mod level;
mod machete;
mod map;
mod mode;
mod molotov;
mod progress;
mod score;
mod spawn;
mod stain;
mod vec2;
mod weather;
mod world;

pub use announce::{News, WeatherNews};
pub use build::{Choice, Legendary, MAX_LEVEL, MAX_PASSIVES, MAX_WEAPONS, Passive, Rarity, Weapon};
pub use burst::{Blast, REACH as BURST_REACH, Swelling};
pub use enemy::{Enemy, EnemyKind};
pub use facing::Facing;
pub use guns::Bullet;
pub use heal::light as campfire_light;
pub use leaderboard::{Entry, Leaderboard};
pub use level::xp_to_next;
pub use machete::{SWING_TICKS, Swing};
pub use map::{Feature, Map, OBSTACLE_SIZE, Obstacle};
pub use mode::Mode;
pub use progress::{Achievement, Progress, Unlock};

pub use stain::{Stain, lasts as stain_lasts};
pub use vec2::Vec2;
pub use weather::{STRIKE_RADIUS, STRIKE_WARNING_TICKS, Weather};
pub use world::{Input, Lightning, TORCH_FADE, World};

/// How many ticks the world moves on per second.
pub const TICKS_PER_SECOND: u16 = 30;

/// The seconds one tick lasts.
fn tick_seconds() -> f32 {
    1.0 / f32::from(TICKS_PER_SECOND)
}
