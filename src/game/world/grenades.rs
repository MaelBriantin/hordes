//! The grenade at work: thrown at the biggest crowds near the player, blowing where it lands.

use super::super::build::Weapon;
use super::super::c4::{BLAST_TICKS, Blast};
use super::super::enemy::EnemyKind;
use super::super::grenade::{self, FLIGHT_TICKS, KNOCKBACK, Thrown, WAVE_TICKS};
use super::super::napalm;
use super::World;
use super::weapons::against;

impl World {
    /// The grenade: its blasts fade; the grenades in the air fly on, each blowing where it
    /// lands, hurting every enemy around and throwing them back; once it can, a crowd within
    /// reach, a throw at it, two crowds at the last level. Not in the dark: nothing to aim at.
    pub(super) fn throw_grenades(&mut self) {
        for blast in &mut self.grenade_blasts {
            blast.age += 1;
        }
        self.grenade_blasts.retain(|blast| blast.age < BLAST_TICKS);
        let level = self.player.build.weapon_level(Weapon::Grenade);
        if level == 0 {
            self.grenades.clear();
            return;
        }
        let stats = grenade::at(level);
        for thrown in &mut self.grenades {
            thrown.left = thrown.left.saturating_sub(1);
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let ticks = f32::from(u16::try_from(WAVE_TICKS).unwrap_or(1));
        for thrown in self.grenades.iter().filter(|thrown| thrown.left == 0) {
            for enemy in &mut self.enemies {
                let apart = enemy.position - thrown.to;
                if apart.length() > stats.radius + enemy.kind.stats().half_width {
                    continue;
                }
                let standing = enemy.hp > 0;
                enemy.hit(against(damage, enemy, hunter));
                if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                    self.abominations_smashed += 1;
                }
                enemy.knock = apart.normalized() * (KNOCKBACK / ticks);
                enemy.knocked = WAVE_TICKS;
            }
            self.grenade_blasts.push(Blast {
                at: thrown.to,
                radius: stats.radius,
                age: 0,
            });
        }
        self.grenades.retain(|thrown| thrown.left > 0);
        self.grenade_wait = self.grenade_wait.saturating_sub(1);
        let dark = self.torch_on().is_some() && !self.torch_lit();
        if self.grenade_wait > 0 || dark {
            return;
        }
        let player = self.player.position;
        let mut enemies: Vec<_> = self.enemies.iter().map(|enemy| enemy.position).collect();
        let mut thrown = false;
        for _ in 0..stats.count {
            let Some(target) = napalm::crowd(player, &enemies) else {
                break;
            };
            self.grenades.push(Thrown {
                from: player,
                to: target,
                left: FLIGHT_TICKS,
            });
            thrown = true;
            // The next one at another crowd: none of this one's.
            enemies.retain(|at| (*at - target).length() > stats.radius);
        }
        if thrown {
            self.grenade_wait = self.player.build.cooldown(stats.every);
        }
    }
}
