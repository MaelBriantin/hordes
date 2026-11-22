//! The C4 at work: its charges set down at the player's feet, and their blasts.

use super::super::build::Weapon;
use super::super::c4::{self, BLAST_TICKS, Blast, Charge, FUSE_TICKS, KNOCKBACK, NEAR, WAVE_TICKS};
use super::World;
use super::weapons::against;

impl World {
    /// The C4: its blasts fade; its charges wait, each blowing once its fuse is out, hurting
    /// every enemy around, throwing them back, stunning them at the last level; once it can, an
    /// enemy near, a new charge is set down at the player's feet. In the dark too.
    pub(super) fn set_charges(&mut self) {
        for blast in &mut self.c4_blasts {
            blast.age += 1;
        }
        self.c4_blasts.retain(|blast| blast.age < BLAST_TICKS);
        let level = self.player.build.weapon_level(Weapon::C4);
        if level == 0 {
            self.charges.clear();
            return;
        }
        let stats = c4::at(level);
        for charge in &mut self.charges {
            charge.left = charge.left.saturating_sub(1);
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let ticks = f32::from(u16::try_from(WAVE_TICKS).unwrap_or(1));
        for charge in self.charges.iter().filter(|charge| charge.left == 0) {
            for enemy in &mut self.enemies {
                let apart = enemy.position - charge.at;
                if apart.length() > stats.radius + enemy.kind.stats().half_width {
                    continue;
                }
                enemy.hit(against(damage, enemy, hunter));
                enemy.stunned = enemy.stunned.max(stats.stun);
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
        self.charges.retain(|charge| charge.left > 0);
        self.c4_wait = self.c4_wait.saturating_sub(1);
        let player = self.player.position;
        let near = self
            .enemies
            .iter()
            .any(|enemy| (enemy.position - player).length() <= NEAR);
        if self.c4_wait == 0 && near {
            self.charges.push(Charge {
                at: player,
                left: FUSE_TICKS,
            });
            self.c4_wait = self.player.build.cooldown(stats.every);
        }
    }
}
