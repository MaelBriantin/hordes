//! The smoke bomb at work, the scout's own action.

use super::super::build::Action;
use super::super::smoke::{self, Cloud};
use super::World;

impl World {
    /// The player throws their smoke bomb at their feet, if it is ready; then it waits its
    /// recharge.
    pub(super) fn throw_smoke(&mut self) {
        let level = self.player.build.action_level(Action::Smoke);
        if self.announcement.is_some() || level == 0 || self.smoke_wait > 0 {
            return;
        }
        let stats = smoke::at(level);
        self.smoke = Some(Cloud {
            at: self.player.position,
            radius: stats.radius,
            left: stats.lasts,
            lasts: stats.lasts,
            slows: stats.slows,
        });
        self.smoke_wait = self.player.build.cooldown(stats.recharge).max(1);
    }

    /// The smoke bomb's recharge runs down; its cloud thins out, then clears.
    pub(super) fn thin_smoke(&mut self) {
        self.smoke_wait = self.smoke_wait.saturating_sub(1);
        if let Some(cloud) = &mut self.smoke {
            cloud.left = cloud.left.saturating_sub(1);
            if cloud.left == 0 {
                self.smoke = None;
            }
        }
    }

    /// Whether the player is in their smoke, lost to the enemies.
    pub fn smoked(&self) -> bool {
        self.smoke
            .is_some_and(|cloud| cloud.holds(self.player.position))
    }
}
