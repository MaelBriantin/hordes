//! The first-aid kit at work, the medic's own action.

use super::super::build::Action;
use super::super::kit;
use super::World;

impl World {
    /// The player opens their first-aid kit, if it is ready and they are hurt: HP back at once;
    /// then it waits its recharge.
    pub(super) fn open_kit(&mut self) {
        let level = self.player.build.action_level(Action::Kit);
        let hurt = self.player.hp < self.player.max_hp();
        if self.announcement.is_some() || level == 0 || self.kit_wait > 0 || !hurt {
            return;
        }
        let stats = kit::at(level);
        self.kit_wait = self.player.build.cooldown(stats.recharge).max(1);
        self.heal(stats.heal);
        self.healed_at = Some(self.ticks);
    }

    /// The first-aid kit's recharge runs down.
    pub(super) fn wait_kit(&mut self) {
        self.kit_wait = self.kit_wait.saturating_sub(1);
    }
}
