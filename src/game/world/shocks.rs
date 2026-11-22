//! The defibrillator at work: its shocks leaping through the crowd, healing the medic.

use super::super::build::Weapon;
use super::super::defibrillator::{self, LEAP, REACH, SHOCK_TICKS, Shock};
use super::super::enemy::EnemyKind;
use super::World;
use super::weapons::against;

impl World {
    /// The defibrillator: its last shock fades; once it can, it shocks the nearest enemy within
    /// its reach, then leaps on to the nearest to that one, and so on, as many as it chains;
    /// each enemy it fells gives the player back some HP. In the dark too: it strikes what is
    /// near.
    pub(super) fn shock_defibrillator(&mut self) {
        if let Some(shock) = &mut self.shock {
            shock.age += 1;
            if shock.age >= SHOCK_TICKS {
                self.shock = None;
            }
        }
        let level = self.player.build.weapon_level(Weapon::Defibrillator);
        if level == 0 {
            return;
        }
        self.defibrillator_wait = self.defibrillator_wait.saturating_sub(1);
        if self.defibrillator_wait > 0 {
            return;
        }
        let stats = defibrillator::at(level);
        let mut path = vec![self.player.position];
        let mut shocked: Vec<usize> = Vec::new();
        while shocked.len() < stats.chain {
            let (from, reach) = (
                path[path.len() - 1],
                if shocked.is_empty() { REACH } else { LEAP },
            );
            let next = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(index, enemy)| !shocked.contains(index) && enemy.hp > 0)
                .map(|(index, enemy)| {
                    let apart = (enemy.position - from).length() - enemy.kind.stats().half_width;
                    (apart, index)
                })
                .filter(|(apart, _)| *apart <= reach)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((_, index)) = next else {
                break;
            };
            shocked.push(index);
            path.push(self.enemies[index].position);
        }
        if shocked.is_empty() {
            return;
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let mut felled = 0;
        for index in shocked {
            let enemy = &mut self.enemies[index];
            let standing = enemy.hp > 0;
            enemy.hit(against(damage, enemy, hunter));
            if standing && enemy.hp == 0 {
                felled += 1;
                if enemy.kind == EnemyKind::Abomination {
                    self.abominations_smashed += 1;
                }
            }
        }
        if felled > 0 && self.heal(felled * stats.heals) > 0 {
            self.healed_at = Some(self.ticks);
        }
        self.shock = Some(Shock { path, age: 0 });
        self.defibrillator_wait = self.player.build.cooldown(stats.every);
    }
}
