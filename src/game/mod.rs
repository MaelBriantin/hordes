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
mod fire_wall;
mod fists;
mod flamethrower;
mod flashball;
mod gatling;
mod gem;
mod grenade;
mod guns;
mod hammer;
mod heal;
mod horde;
mod kamikaze;
mod katana;
mod kit;
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
mod revival;
mod score;
mod skin;
mod smoke;
mod sniper;
mod spawn;
mod spitter;
mod stain;
mod vault;
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
pub use c4::Blast as Fireball;
pub use enemy::{Enemy, EnemyKind};
pub use facing::Facing;
pub use flashball::{BURST_TICKS as FLASH_BURST_TICKS, Burst as FlashBurst};
pub use gatling::CASE_TICKS;
pub use grenade::ARC as GRENADE_ARC;
pub use guns::Bullet;
pub use hammer::{Blow, WAVE_TICKS as HAMMER_WAVE_TICKS};
pub use heal::light as campfire_light;
pub use kamikaze::LAST_TICKS as KAMIKAZE_LAST_TICKS;
pub use leaderboard::{Entry, Leaderboard};
pub use level::xp_to_next;
pub use machete::{SWING_TICKS, Swing};
pub use map::{Feature, Map, OBSTACLE_SIZE, Obstacle};
pub use mode::Mode;
pub use molotov::Puddle;
pub use progress::{Achievement, COLLECTION, Progress, Unlock};
pub use revival::SOUL_TICKS as REVIVAL_SOUL_TICKS;
pub use smoke::{CLEAR_TICKS as SMOKE_CLEAR_TICKS, SPREAD_TICKS as SMOKE_SPREAD_TICKS};
pub use sniper::IMPACT_TICKS as SNIPER_IMPACT_TICKS;
pub use spitter::WIND_TICKS as SPIT_WIND_TICKS;

/// The ticks the C4 takes to come back after a blast, by its level, before Haste.
pub fn c4_recharge(level: u8) -> u32 {
    c4::at(level).recharge
}

pub use kit::at as kit_at;

/// The ticks the vault takes to come back, by its level, before Haste.
pub fn vault_recharge(level: u8) -> u32 {
    vault::at(level).recharge
}

/// The ticks the gatling burst takes to come back, by its level, before Haste.
pub fn gatling_recharge(level: u8) -> u32 {
    gatling::at(level).recharge
}

/// The ticks the smoke bomb takes to come back, by its level, before Haste.
pub fn smoke_recharge(level: u8) -> u32 {
    smoke::at(level).recharge
}

/// The ticks the wall of fire takes to come back, by its level, before Haste.
pub fn fire_wall_recharge(level: u8) -> u32 {
    fire_wall::at(level).recharge
}

/// The ticks the first-aid kit takes to come back, by its level, before Haste.
pub fn kit_recharge(level: u8) -> u32 {
    kit::at(level).recharge
}

/// The ticks the ram takes to come back, by its level, before Haste.
pub fn ram_recharge(level: u8) -> u32 {
    ram::at(level).recharge
}

pub use skin::{Perk, Skin};
pub use stain::{Stain, lasts as stain_lasts};
pub use vec2::Vec2;
pub use weather::{DAWN_SECONDS, STRIKE_RADIUS, STRIKE_WARNING_TICKS, Weather};
pub use world::{Input, Lightning, TORCH_FADE, Veil, World};

/// How many ticks the world moves on per second.
pub const TICKS_PER_SECOND: u16 = 30;

/// The seconds one tick lasts.
fn tick_seconds() -> f32 {
    1.0 / f32::from(TICKS_PER_SECOND)
}
