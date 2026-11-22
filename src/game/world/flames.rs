//! The flamethrower at work: its cone of fire ahead of the player, its tank, and the trail of
//! fires its tank leaves at the last level.

use super::super::Vec2;
use super::super::body::in_sight;
use super::super::build::Weapon;
use super::super::flamethrower::{self, BURN_EVERY, NOZZLE, TRAIL_RADIUS, TRAIL_STEP};
use super::super::guns::Magazine;
use super::super::molotov::Puddle;
use super::World;
use super::weapons::against;

impl World {
    /// The flamethrower's fire while it burns: where it leaves the nozzle, the way it goes
    /// (length 1), and how far it flies from there, in tiles.
    pub fn flame_cone(&self) -> Option<(Vec2, Vec2, f32)> {
        let level = self.player.build.weapon_level(Weapon::Flamethrower);
        (self.flaming && level > 0).then(|| {
            let way = self.player.facing.way();
            let range = flamethrower::at(level).range;
            (self.player.position + way * NOZZLE, way, range - NOZZLE)
        })
    }

    /// Lit by an enemy in its cone, the flamethrower burns its whole tank ahead of the player,
    /// the way they face, enemies or not: every enemy in the cone, in sight, takes its damage
    /// and catches fire, a few times a second. Empty, the tank fills again. In the dark too: its
    /// fire lights what it burns.
    pub(super) fn throw_flames(&mut self) {
        let lit = self.flaming;
        self.flaming = false;
        let level = self.player.build.weapon_level(Weapon::Flamethrower);
        if level == 0 {
            return;
        }
        let stats = flamethrower::at(level);
        self.leak(stats.leaks);
        let Self {
            player,
            enemies,
            magazines,
            map,
            ..
        } = self;
        let build = &player.build;
        let tank = magazines.entry(Weapon::Flamethrower).or_insert(Magazine {
            rounds: stats.tank,
            ..Magazine::default()
        });
        if tank.reloading > 0 {
            tank.reloading -= 1;
            if tank.reloading == 0 {
                tank.rounds = stats.tank;
            }
            return;
        }
        let (from, way) = (player.position, player.facing.way());
        let burning: Vec<usize> = enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| {
                let half = enemy.kind.stats().half_width;
                flamethrower::in_cone(from, way, stats.range, enemy.position, half)
                    && in_sight(map, from, enemy.position)
            })
            .map(|(index, _)| index)
            .collect();
        if !lit && burning.is_empty() {
            return;
        }
        self.flaming = true;
        tank.rounds = tank.rounds.saturating_sub(1);
        if tank.rounds == 0 {
            tank.reloading = build.cooldown(flamethrower::refill()).max(1);
        }
        // Its first burst at once, then every few ticks for as long as it burns.
        let burst = self.flame_ticks.is_multiple_of(BURN_EVERY);
        self.flame_ticks += 1;
        if !burst {
            return;
        }
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        for index in burning {
            let enemy = &mut enemies[index];
            enemy.hit(against(damage, enemy, hunter));
            enemy.ignite_for(build.burn_ticks());
        }
    }

    /// At the last level, the tank leaks: a small fire where the player was, every few steps.
    fn leak(&mut self, leaks: bool) {
        let at = self.player.position;
        if !leaks {
            self.trail_from = at;
            return;
        }
        if (at - self.trail_from).length() < TRAIL_STEP {
            return;
        }
        self.trail_from = at;
        let lasts = flamethrower::trail_lasts();
        self.puddles.push(Puddle {
            centre: at,
            radius: TRAIL_RADIUS,
            left: lasts,
            lasts,
            burst: false,
        });
    }
}
