//! The stun baton at work: its thrusts straight ahead of the player.

use super::super::baton::{self, THRUST_TICKS, Thrust};
use super::super::build::Weapon;
use super::World;
use super::weapons::against;

impl World {
    /// The stun baton: its last thrust fades; once it can, and an enemy stands on its line, it
    /// thrusts straight ahead, the way the player faces, hurting every enemy on the line and
    /// paralysing them a while. In the dark too: it strikes what is near.
    pub(super) fn thrust_baton(&mut self) {
        if let Some(thrust) = &mut self.thrust {
            thrust.age += 1;
            if thrust.age >= THRUST_TICKS {
                self.thrust = None;
            }
        }
        let level = self.player.build.weapon_level(Weapon::Baton);
        if level == 0 {
            return;
        }
        self.baton_wait = self.baton_wait.saturating_sub(1);
        if self.baton_wait > 0 {
            return;
        }
        let stats = baton::at(level);
        let (player, way) = (self.player.position, self.player.facing.way());
        let on_line = |enemy: &super::super::enemy::Enemy| {
            baton::on_line(
                enemy.position - player,
                enemy.kind.stats().half_width,
                way,
                stats.reach,
            )
        };
        if !self.enemies.iter().any(on_line) {
            return;
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        for enemy in self.enemies.iter_mut().filter(|enemy| on_line(enemy)) {
            enemy.hit(against(damage, enemy, hunter));
            enemy.stunned = enemy.stunned.max(stats.paralysis);
            enemy.shocked = enemy.shocked.max(stats.paralysis);
        }
        self.thrust = Some(Thrust {
            way,
            reach: stats.reach,
            age: 0,
        });
        self.baton_wait = self.player.build.cooldown(stats.every);
    }
}
