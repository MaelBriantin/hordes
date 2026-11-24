//! The wall of fire at work, the exterminator's own action.

use super::super::build::Action;
use super::super::fire_wall::{self, STEP};
use super::super::molotov::Puddle;
use super::World;

impl World {
    /// The player lights their wall of fire, if it is ready: the fire follows them a while; then
    /// it waits its recharge.
    pub(super) fn light_fire_wall(&mut self) {
        let level = self.player.build.action_level(Action::FireWall);
        if self.announcement.is_some() || level == 0 || self.fire_wall_wait > 0 {
            return;
        }
        let stats = fire_wall::at(level);
        self.fire_wall_left = stats.trail;
        self.fire_wall_wait = self.player.build.cooldown(stats.recharge).max(1);
        self.trail_from = self.player.position;
        self.lay_fire(stats.radius, stats.lasts);
    }

    /// The wall of fire's recharge runs down; while it follows the player, a stretch of fire
    /// every step they take, burning on a while: it burns the enemies as the Molotov's puddles.
    pub(super) fn follow_fire_wall(&mut self) {
        self.fire_wall_wait = self.fire_wall_wait.saturating_sub(1);
        if self.fire_wall_left == 0 {
            return;
        }
        self.fire_wall_left -= 1;
        if (self.player.position - self.trail_from).length() < STEP {
            return;
        }
        self.trail_from = self.player.position;
        let stats = fire_wall::at(self.player.build.action_level(Action::FireWall).max(1));
        self.lay_fire(stats.radius, stats.lasts);
    }

    /// A stretch of the wall where the player stands.
    fn lay_fire(&mut self, radius: f32, lasts: u32) {
        self.puddles.push(Puddle {
            centre: self.player.position,
            radius,
            left: lasts,
            lasts,
            burst: false,
            wall: true,
            hostile: false,
            acid: false,
        });
    }
}
