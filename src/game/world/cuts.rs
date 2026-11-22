//! The katana at work: its chains of cuts through the crowd.

use super::super::build::Weapon;
use super::super::enemy::EnemyKind;
use super::super::katana::{self, CUT_TICKS, Cuts, LEAP, REACH};
use super::World;
use super::weapons::against;

impl World {
    /// The katana: its last cuts fade; once it can, it cuts the nearest enemy within its reach,
    /// then the nearest to that one within a leap, and so on, as many as it chains, each cut
    /// taking less than the one before. In the dark too: it strikes what is near.
    pub(super) fn cut_katana(&mut self) {
        if let Some(cuts) = &mut self.cuts {
            cuts.age += 1;
            if cuts.age >= CUT_TICKS {
                self.cuts = None;
            }
        }
        let level = self.player.build.weapon_level(Weapon::Katana);
        if level == 0 {
            return;
        }
        self.katana_wait = self.katana_wait.saturating_sub(1);
        if self.katana_wait > 0 {
            return;
        }
        let stats = katana::at(level);
        let mut path = vec![self.player.position];
        let mut cut: Vec<usize> = Vec::new();
        while cut.len() < stats.chain {
            let (from, reach) = (
                path[path.len() - 1],
                if cut.is_empty() { REACH } else { LEAP },
            );
            let next = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(index, _)| !cut.contains(index))
                .map(|(index, enemy)| {
                    let apart = (enemy.position - from).length() - enemy.kind.stats().half_width;
                    (apart, index)
                })
                .filter(|(apart, _)| *apart <= reach)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((_, index)) = next else {
                break;
            };
            cut.push(index);
            path.push(self.enemies[index].position);
        }
        if cut.is_empty() {
            return;
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        for (step, index) in cut.into_iter().enumerate() {
            let enemy = &mut self.enemies[index];
            let standing = enemy.hp > 0;
            enemy.hit(against(katana::cut_damage(damage, step), enemy, hunter));
            if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                self.abominations_smashed += 1;
            }
        }
        self.cuts = Some(Cuts { path, age: 0 });
        self.katana_wait = self.player.build.cooldown(stats.every);
    }
}
