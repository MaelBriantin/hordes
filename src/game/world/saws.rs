//! The chainsaw at work: what touches it ground on and on, slowed.

use super::super::build::Weapon;
use super::super::chainsaw::{self, BITE_TICKS, SLOW_TICKS};
use super::super::enemy::EnemyKind;
use super::super::machete;
use super::World;
use super::weapons::against;

impl World {
    /// The chainsaw: every enemy within its reach ahead of the player, the way they face, is
    /// slowed while it is, and bitten every few ticks. In the dark too: it grinds what is near.
    pub(super) fn grind_chainsaw(&mut self) {
        self.sawing = false;
        let level = self.player.build.weapon_level(Weapon::Chainsaw);
        if level == 0 {
            return;
        }
        let stats = chainsaw::at(level);
        let (player, way) = (self.player.position, self.player.facing.way());
        let build = &self.player.build;
        self.chainsaw_wait = self.chainsaw_wait.saturating_sub(1);
        let bite = self.chainsaw_wait == 0;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        for enemy in &mut self.enemies {
            let half = enemy.kind.stats().half_width;
            if !machete::in_arc(
                enemy.position - player,
                half,
                stats.reach,
                way,
                stats.half_arc,
            ) {
                continue;
            }
            self.sawing = true;
            enemy.slowed = enemy.slowed.max(SLOW_TICKS);
            if bite {
                let standing = enemy.hp > 0;
                enemy.hit(against(damage, enemy, hunter));
                if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                    self.abominations_smashed += 1;
                }
            }
        }
        if self.sawing && bite {
            self.chainsaw_wait = self.player.build.cooldown(BITE_TICKS).max(1);
        }
    }
}
