//! Bolt, the weapon every run starts with: sparks shot at the nearest enemies, every 2 seconds.
//! A spark flies over obstacles and stops at the first enemy it hits.

use super::enemy::Enemy;
use super::{TICKS_PER_SECOND, Vec2, tick_seconds};

/// In tiles per second.
const SPEED: f32 = 15.0;
/// How far a spark flies, and how close an enemy must be to be shot at, in tiles.
const RANGE: f32 = 12.0;
/// Half the width of a spark, in tiles.
pub const HALF_WIDTH: f32 = 0.3;

/// What a spark takes from the enemy it hits, in HP, at a level of Bolt: 10, 20 from level 5.
pub fn damage(level: u8) -> u32 {
    if level >= 5 { 20 } else { 10 }
}

/// The sparks a shot holds at a level: 1, 2 from level 2, 3 from level 4.
pub fn sparks(level: u8) -> usize {
    match level {
        0 | 1 => 1,
        2 | 3 => 2,
        _ => 3,
    }
}

/// The ticks between two shots at a level: 2 seconds, 1.5 from level 3.
pub fn cooldown(level: u8) -> u32 {
    let ticks = u32::from(TICKS_PER_SECOND);
    if level >= 3 { ticks * 3 / 2 } else { ticks * 2 }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spark {
    pub position: Vec2,
    /// Its move each tick.
    velocity: Vec2,
    /// How far it has flown, in tiles.
    flown: f32,
    /// What it takes from the enemy it hits, in HP.
    pub damage: u32,
}

impl Spark {
    /// A spark leaving `from` towards `target`.
    pub fn toward(from: Vec2, target: Vec2, damage: u32) -> Self {
        Self {
            position: from,
            velocity: (target - from).normalized() * (SPEED * tick_seconds()),
            flown: 0.0,
            damage,
        }
    }

    /// Moves it on by one tick. Whether it is still flying.
    pub fn advance(&mut self) -> bool {
        self.position += self.velocity;
        self.flown += self.velocity.length();
        self.flown < RANGE
    }
}

/// Where the `count` enemies nearest to `from` are, those in range, nearest first.
pub fn targets(enemies: &[Enemy], from: Vec2, count: usize) -> Vec<Vec2> {
    let mut in_range: Vec<Vec2> = enemies
        .iter()
        .map(|enemy| enemy.position)
        .filter(|position| (*position - from).length() <= RANGE)
        .collect();
    in_range.sort_by(|a, b| (*a - from).length().total_cmp(&(*b - from).length()));
    in_range.truncate(count);
    in_range
}
