//! The stun baton at work: its thrusts straight ahead of the player, and its whirlwinds all
//! around.

use super::super::Facing;
use super::super::baton::{self, Thrust};
use super::super::build::Weapon;
use super::super::enemy::EnemyKind;
use super::World;
use super::weapons::against;

impl World {
    /// The stun baton: its last strike fades; once it can, and an enemy stands on its line, it
    /// thrusts straight ahead, the way the player faces, hurting every enemy on the line and
    /// paralysing them a while; a strike now and then, from its third level, is a whirlwind,
    /// once an enemy stands within it: every enemy around is hit. In the dark too: it strikes
    /// what is near.
    pub(super) fn thrust_baton(&mut self) {
        if let Some(thrust) = &mut self.thrust {
            thrust.age += 1;
            if thrust.age >= thrust.lasts() {
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
        let strike = self.baton_strikes + 1;
        let whirl = stats
            .whirl_every
            .is_some_and(|every| strike.is_multiple_of(every));
        let on_line = |enemy: &super::super::enemy::Enemy| {
            if enemy.hp == 0 {
                return false;
            }
            let (offset, half) = (enemy.position - player, enemy.kind.stats().half_width);
            if whirl {
                baton::in_whirl(offset, half)
            } else {
                baton::on_line(offset, half, way, stats.reach)
            }
        };
        if !self.enemies.iter().any(on_line) {
            return;
        }
        self.baton_strikes = strike;
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        for enemy in self.enemies.iter_mut().filter(|enemy| on_line(enemy)) {
            let standing = enemy.hp > 0;
            enemy.hit(against(damage, enemy, hunter));
            if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                self.abominations_smashed += 1;
            }
            enemy.stunned = enemy.stunned.max(stats.paralysis);
            enemy.shocked = enemy.shocked.max(stats.paralysis);
        }
        self.thrust = Some(Thrust {
            way,
            reach: stats.reach,
            age: 0,
            whirl,
        });
        self.baton_wait = self.player.build.cooldown(stats.every);
    }

    /// The way the player shows facing: round as they spin in a whirlwind, the way they face
    /// otherwise.
    pub fn hero_facing(&self) -> Facing {
        self.thrust
            .and_then(|thrust| thrust.whirl_angle())
            .map_or(self.player.facing, baton::facing_at)
    }
}
