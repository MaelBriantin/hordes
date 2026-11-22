//! The game: its rules and its state. No terminal and no clock here: the app moves it on one tick
//! at a time with what the player holds down, and the ui draws what it holds.

mod announce;
mod barricade;
mod baton;
mod body;
mod build;
mod burst;
mod c4;
mod chainsaw;
mod dash;
mod defibrillator;
mod drones;
mod enemy;
mod facing;
mod fists;
mod flamethrower;
mod gem;
mod guns;
mod hammer;
mod heal;
mod horde;
mod katana;
mod knife;
mod leaderboard;
mod level;
mod machete;
mod map;
mod mode;
mod molotov;
mod napalm;
mod progress;
pub mod ram;
mod score;
mod skin;
mod spawn;
mod stain;
mod vec2;
mod weather;
mod world;

pub use announce::{News, WeatherNews};
pub use barricade::{Barricade, recharge as barricade_recharge};
pub use build::{
    Action, Choice, HEAL, Legendary, MAX_LEVEL, MAX_PASSIVES, MAX_WEAPONS, Passive, Rarity, Weapon,
    percent_of,
};
pub use burst::{Blast, REACH as BURST_REACH, Swelling};
pub use enemy::{Enemy, EnemyKind};
pub use facing::Facing;
pub use guns::Bullet;
pub use hammer::{Blow, WAVE_TICKS as HAMMER_WAVE_TICKS};
pub use heal::light as campfire_light;
pub use leaderboard::{Entry, Leaderboard};
pub use level::xp_to_next;
pub use machete::{SWING_TICKS, Swing};
pub use map::{Feature, Map, OBSTACLE_SIZE, Obstacle};
pub use mode::Mode;
pub use progress::{Achievement, COLLECTION, Progress, Unlock};

/// The ticks the ram takes to come back, by its level, before Haste.
pub fn ram_recharge(level: u8) -> u32 {
    ram::at(level).recharge
}

pub use skin::{Perk, Skin};
pub use stain::{Stain, lasts as stain_lasts};
pub use vec2::Vec2;
pub use weather::{STRIKE_RADIUS, STRIKE_WARNING_TICKS, Weather};
pub use world::{Input, Lightning, TORCH_FADE, Veil, World};

/// How many ticks the world moves on per second.
pub const TICKS_PER_SECOND: u16 = 30;

/// The seconds one tick lasts.
fn tick_seconds() -> f32 {
    1.0 / f32::from(TICKS_PER_SECOND)
}
