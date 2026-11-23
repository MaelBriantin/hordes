//! The vault, the special agent's own action: over the obstacle just ahead, landing behind it,
//! nothing hurting them on the way; nothing ahead, a short hop. Its levels let them clear an
//! enemy too, then up to three blocks in a row; landing, a kick throws the enemies near back.

use super::TICKS_PER_SECOND;
use super::map::OBSTACLE_SIZE;
use super::vec2::Vec2;

/// The ticks a vault takes: 0.4 s.
pub const TICKS: u32 = 12;
/// How far a hop goes with nothing ahead, in tiles.
pub const HOP_TILES: f32 = 2.0;
/// How near the obstacle must be to vault over it, in tiles: just ahead.
pub const AHEAD: f32 = 1.0;
/// A block's depth, in tiles: an obstacle's.
#[allow(clippy::cast_precision_loss, reason = "a couple of tiles")]
pub const BLOCK: f32 = OBSTACLE_SIZE as f32;
/// The kick on landing: how far around it reaches, how far it throws the enemies back, in
/// tiles, over how many ticks.
pub const KICK_REACH: f32 = 1.5;
pub const KICK_TILES: f32 = 2.0;
pub const KICK_TICKS: u32 = 6;
/// How high the agent rises at the vault's middle, in tiles: a low, quick jump.
const LIFT: f32 = 0.5;

/// The vault at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    /// How many blocks in a row it clears.
    pub blocks: u8,
    /// Whether it clears enemies too.
    pub over_enemies: bool,
    /// Whether a kick throws the enemies near back on landing.
    pub kicks: bool,
    /// The ticks after it before the next, before Haste.
    pub recharge: u32,
}

/// The vault at a level: a kick at 2, over enemies at 3, sooner back at 4, three blocks at 5.
pub fn at(level: u8) -> Stats {
    let second = u32::from(TICKS_PER_SECOND);
    Stats {
        blocks: if level >= 5 { 3 } else { 1 },
        over_enemies: level >= 3,
        kicks: level >= 2,
        recharge: if level >= 4 { 4 } else { 6 } * second,
    }
}

/// A vault under way: from where to where, the ticks it has gone, and whether it ends in a
/// kick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vault {
    pub from: Vec2,
    pub to: Vec2,
    pub age: u32,
    pub kicks: bool,
}

impl Vault {
    /// How far along it is, from 0 to 1.
    pub fn share(&self) -> f32 {
        f32::from(u16::try_from(self.age.min(TICKS)).unwrap_or(0))
            / f32::from(u16::try_from(TICKS).unwrap_or(1))
    }

    /// Where the agent is now.
    pub fn position(&self) -> Vec2 {
        self.from + (self.to - self.from) * self.share()
    }

    /// How high they are now, in tiles: up, then down.
    pub fn lift(&self) -> f32 {
        let share = self.share();
        LIFT * 4.0 * share * (1.0 - share)
    }

    /// The way it goes.
    pub fn way(&self) -> Vec2 {
        (self.to - self.from).normalized()
    }
}
