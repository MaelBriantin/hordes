//! The C4 at work, the demolisher's own action: its charges set down, blown from afar or by
//! themselves, and their blasts.

use super::super::build::Action;
use super::super::c4::{self, BLAST_TICKS, Blast, Charge, FUSE_TICKS, KNOCKBACK, WAVE_TICKS};
use super::super::enemy::EnemyKind;
use super::World;
use super::weapons::against;

impl World {
    /// The player plays the C4: a charge set down at their feet, if one is ready and room is
    /// left for it; with as many down as can stand, they all blow, from afar.
    pub(super) fn c4(&mut self) {
        let level = self.player.build.action_level(Action::C4);
        if self.announcement.is_some() || level == 0 {
            return;
        }
        if self.charges.len() < c4::at(level).charges && self.c4_wait == 0 {
            self.charges.push(Charge {
                at: self.player.position,
                left: FUSE_TICKS,
            });
        } else {
            for charge in &mut self.charges {
                charge.left = 0;
            }
        }
    }

    /// The C4: its blasts fade; its charges wait; once one is blown, from afar or its fuse out,
    /// they all blow together, each hurting every enemy around, throwing them back; then the
    /// next charge waits its recharge. In the dark too.
    pub(super) fn set_charges(&mut self) {
        for blast in &mut self.c4_blasts {
            blast.age += 1;
        }
        self.c4_blasts.retain(|blast| blast.age < BLAST_TICKS);
        self.c4_wait = self.c4_wait.saturating_sub(1);
        let level = self.player.build.action_level(Action::C4);
        if level == 0 {
            self.charges.clear();
            return;
        }
        for charge in &mut self.charges {
            charge.left = charge.left.saturating_sub(1);
        }
        if self.charges.iter().all(|charge| charge.left > 0) {
            return;
        }
        let stats = c4::at(level);
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let ticks = f32::from(u16::try_from(WAVE_TICKS).unwrap_or(1));
        for charge in &self.charges {
            for enemy in &mut self.enemies {
                let apart = enemy.position - charge.at;
                if apart.length() > stats.radius + enemy.kind.stats().half_width {
                    continue;
                }
                let standing = enemy.hp > 0;
                enemy.hit(against(damage, enemy, hunter));
                if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                    self.abominations_smashed += 1;
                }
                // Thrown out over the wave's ticks, as the blast goes.
                enemy.knock = apart.normalized() * (KNOCKBACK / ticks);
                enemy.knocked = WAVE_TICKS;
            }
            self.c4_blasts.push(Blast {
                at: charge.at,
                radius: stats.radius,
                age: 0,
            });
        }
        self.charges.clear();
        self.c4_wait = self.player.build.cooldown(stats.recharge).max(1);
    }
}
