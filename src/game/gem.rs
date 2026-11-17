//! XP gems, left by fallen enemies. Once the player comes near, a gem flies to them.

use super::{Vec2, tick_seconds};

/// How fast a gem flies to the player, in tiles per second.
const PULL_SPEED: f32 = 12.0;
/// How near the player a gem is picked up, in tiles.
const PICKUP_RADIUS: f32 = 0.6;

#[derive(Debug, Clone, PartialEq)]
pub struct Gem {
    pub position: Vec2,
    /// The XP it gives.
    pub value: u32,
    /// Whether it flies to the player: once it does, it keeps on until picked up.
    pulled: bool,
}

impl Gem {
    pub fn new(position: Vec2, value: u32) -> Self {
        Self {
            position,
            value,
            pulled: false,
        }
    }

    /// Moves it on by one tick, the player at `player`, gems flying to them from within
    /// `magnet` tiles. Whether the player picks it up.
    pub fn update(&mut self, player: Vec2, magnet: f32) -> bool {
        let apart = player - self.position;
        if apart.length() <= magnet {
            self.pulled = true;
        }
        if self.pulled {
            self.position += apart.clamp_length(PULL_SPEED * tick_seconds());
        }
        (player - self.position).length() <= PICKUP_RADIUS
    }
}
