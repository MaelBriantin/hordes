//! The combat knife at work: its quick stabs at the nearest enemies.

use super::super::build::Weapon;
use super::super::enemy::EnemyKind;
use super::super::knife::{self, STAB_TICKS, Stab};
use super::World;
use super::weapons::against;

impl World {
    /// The combat knife: its last stabs fade; once it can, it stabs the nearest enemies within
    /// its reach, whichever way. In the dark too: it strikes what is near.
    pub(super) fn stab_knife(&mut self) {
        for stab in &mut self.stabs {
            stab.age += 1;
        }
        self.stabs.retain(|stab| stab.age < STAB_TICKS);
        let level = self.player.build.weapon_level(Weapon::Knife);
        if level == 0 {
            return;
        }
        self.knife_wait = self.knife_wait.saturating_sub(1);
        if self.knife_wait > 0 {
            return;
        }
        let stats = knife::at(level);
        let player = self.player.position;
        let mut near: Vec<(f32, usize)> = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.hp > 0)
            .map(|(index, enemy)| {
                let apart = (enemy.position - player).length() - enemy.kind.stats().half_width;
                (apart, index)
            })
            .filter(|(apart, _)| *apart <= stats.reach)
            .collect();
        if near.is_empty() {
            return;
        }
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        for (_, index) in near.into_iter().take(stats.targets) {
            let enemy = &mut self.enemies[index];
            let standing = enemy.hp > 0;
            enemy.hit(against(damage, enemy, hunter));
            if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                self.abominations_smashed += 1;
            }
            self.stabs.push(Stab {
                at: enemy.position,
                age: 0,
            });
        }
        self.knife_wait = self.player.build.cooldown(stats.every);
    }
}
