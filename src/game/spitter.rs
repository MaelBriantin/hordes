//! The spitter: a zombie, its skin acid, keeping off the player. Every so often, the player in
//! its reach, it pulls back and spits a glob of acid, lobbed at where they stand: it lands a
//! second on, hurting them if they are still there, and leaves a puddle of acid that burns them
//! standing in it, not the enemies.

use super::{TICKS_PER_SECOND, Vec2};

/// How near the player it comes, in tiles; nearer than this, it backs off.
pub const KEEP: f32 = 8.0;
pub const BACK: f32 = 6.0;
/// How near the player it spits, in tiles.
pub const REACH: f32 = 10.0;
/// The ticks between two spits: 2.5 s; the last of them, it pulls back: 0.6 s.
pub const EVERY_TICKS: u32 = 75;
pub const WIND_TICKS: u32 = 18;
/// The ticks a glob flies: 1 s.
pub const FLY_TICKS: u32 = 30;
/// How near where it lands a glob hurts the player, in tiles, and what it takes from them.
pub const SPLASH: f32 = 1.0;
pub const DAMAGE: u32 = 10;
/// How far its puddle spreads, in tiles, and how long it lasts.
pub const PUDDLE_RADIUS: f32 = 1.0;
const PUDDLE_SECONDS: u32 = 3;

/// The ticks its puddle lasts.
pub fn puddle_lasts() -> u32 {
    PUDDLE_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// A glob of acid in the air, lobbed from where the spitter stood to where the player was,
/// over everything; the ticks left before it lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glob {
    pub from: Vec2,
    pub to: Vec2,
    pub left: u32,
}

impl Glob {
    /// How far along its flight, from 0 to 1.
    pub fn progress(&self) -> f32 {
        let gone = u16::try_from(FLY_TICKS.saturating_sub(self.left)).unwrap_or(0);
        let all = u16::try_from(FLY_TICKS).unwrap_or(1);
        f32::from(gone) / f32::from(all)
    }
}
