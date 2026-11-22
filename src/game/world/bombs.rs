//! The napalm at work: its bombs falling on the biggest crowd near the player, and where they
//! land.

use super::super::build::Weapon;
use super::super::c4::{BLAST_TICKS, Blast};
use super::super::molotov::Puddle;
use super::super::napalm::{self, BOMB_RADIUS, Bomb, FALL_TICKS, FIRE_RADIUS};
use super::World;
use super::weapons::against;

impl World {
    /// The napalm: its blasts fade; its bombs fall, each landing hurting and setting on fire
    /// every enemy around, the ground left burning; once it can, a crowd within reach, a new
    /// strike on it. In the dark too: it strikes from the sky.
    pub(super) fn drop_napalm(&mut self) {
        for blast in &mut self.napalm_blasts {
            blast.age += 1;
        }
        self.napalm_blasts.retain(|blast| blast.age < BLAST_TICKS);
        let level = self.player.build.weapon_level(Weapon::Napalm);
        if level == 0 {
            self.bombs.clear();
            return;
        }
        let stats = napalm::at(level);
        for bomb in &mut self.bombs {
            bomb.left = bomb.left.saturating_sub(1);
        }
        let build = &self.player.build;
        let (damage, hunter, burns) = (
            build.damage(stats.damage),
            build.horde_bonus(),
            build.burn_ticks(),
        );
        let lasts = napalm::fire_lasts();
        for bomb in self.bombs.iter().filter(|bomb| bomb.left == 0) {
            for enemy in &mut self.enemies {
                if (enemy.position - bomb.at).length() > BOMB_RADIUS + enemy.kind.stats().half_width
                {
                    continue;
                }
                enemy.hit(against(damage, enemy, hunter));
                enemy.ignite_for(burns);
            }
            self.napalm_blasts.push(Blast {
                at: bomb.at,
                radius: BOMB_RADIUS,
                age: 0,
            });
            self.puddles.push(Puddle {
                centre: bomb.at,
                radius: FIRE_RADIUS,
                left: lasts,
                lasts,
                burst: false,
            });
        }
        self.bombs.retain(|bomb| bomb.left > 0);
        self.napalm_wait = self.napalm_wait.saturating_sub(1);
        if self.napalm_wait > 0 {
            return;
        }
        let player = self.player.position;
        let enemies: Vec<_> = self.enemies.iter().map(|enemy| enemy.position).collect();
        if let Some(target) = napalm::crowd(player, &enemies) {
            self.bombs
                .extend(napalm::strike(player, target, stats.length, stats.crossed));
            self.napalm_wait = self.player.build.cooldown(stats.every);
        }
    }

    /// The napalm's bombs in the last of their fall: their streaks show.
    pub fn falling_bombs(&self) -> impl Iterator<Item = &Bomb> {
        self.bombs.iter().filter(|bomb| bomb.left <= FALL_TICKS)
    }

    /// The fireballs showing: the C4's blasts and the napalm's.
    pub fn fireballs(&self) -> impl Iterator<Item = &Blast> {
        self.c4_blasts.iter().chain(&self.napalm_blasts)
    }
}
