//! The fists at work: punches at the nearest enemy within arm's reach.

use super::super::build::Weapon;
use super::super::enemy::EnemyKind;
use super::super::fists::{self, KNOCK_TICKS, KNOCKBACK, PUNCH_TICKS, Punch};
use super::World;
use super::weapons::against;

impl World {
    /// The fists: the last punch fades; once they can, a punch at the nearest enemy within their
    /// reach, whichever way; every third a heavier one, twice as hard, throwing it back, a hook
    /// stunning it at the last level. In the dark too: they strike what is near.
    pub(super) fn throw_punches(&mut self) {
        if let Some(punch) = &mut self.punch {
            punch.age += 1;
            if punch.age >= PUNCH_TICKS {
                self.punch = None;
            }
        }
        let level = self.player.build.weapon_level(Weapon::Fists);
        if level == 0 {
            return;
        }
        self.fists_wait = self.fists_wait.saturating_sub(1);
        if self.fists_wait > 0 {
            return;
        }
        let stats = fists::at(level);
        let player = self.player.position;
        let Some(enemy) = self
            .enemies
            .iter_mut()
            .filter(|enemy| {
                enemy.hp > 0
                    && (enemy.position - player).length()
                        <= stats.reach + enemy.kind.stats().half_width
            })
            .min_by(|a, b| {
                (a.position - player)
                    .length()
                    .total_cmp(&(b.position - player).length())
            })
        else {
            return;
        };
        self.punches += 1;
        let build = &self.player.build;
        let (damage, heavy) = fists::hit(self.punches, build.damage(stats.damage));
        let standing = enemy.hp > 0;
        enemy.hit(against(damage, enemy, build.horde_bonus()));
        if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
            self.abominations_smashed += 1;
        }
        if heavy {
            let ticks = f32::from(u16::try_from(KNOCK_TICKS).unwrap_or(1));
            enemy.knock = (enemy.position - player).normalized() * (KNOCKBACK / ticks);
            enemy.knocked = KNOCK_TICKS;
            enemy.stunned = enemy.stunned.max(stats.stun);
        }
        self.punch = Some(Punch {
            at: enemy.position,
            heavy,
            age: 0,
        });
        self.fists_wait = self.player.build.cooldown(stats.every);
    }
}
