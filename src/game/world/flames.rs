//! The flamethrower at work: its cone of fire after the nearest enemy, and its tank.

use super::super::Vec2;
use super::super::body::in_sight;
use super::super::build::Weapon;
use super::super::flamethrower::{self, BURN_EVERY, NOZZLE, TURN_DEGREES};
use super::super::guns;
use super::super::guns::Magazine;
use super::World;
use super::weapons::against;

impl World {
    /// The flamethrower's fire while it burns: where it leaves the nozzle, the way it goes
    /// (length 1), and how far it flies from there, in tiles.
    pub fn flame_cone(&self) -> Option<(Vec2, Vec2, f32)> {
        let level = self.player.build.weapon_level(Weapon::Flamethrower);
        (self.flaming && level > 0).then(|| {
            let way = self.flame_way;
            let range = flamethrower::at(level).range;
            (self.player.position + way * NOZZLE, way, range - NOZZLE)
        })
    }

    /// Lit at the nearest enemy in its reach and in sight, the flamethrower burns its whole
    /// tank, enemies or not, its fire turning after the nearest enemy as it burns: every enemy
    /// in the cone, in sight, takes its damage and catches fire, a few times a second. Empty,
    /// the tank fills again. In the dark too: its fire lights what it burns.
    pub(super) fn throw_flames(&mut self) {
        let lit = self.flaming;
        self.flaming = false;
        let level = self.player.build.weapon_level(Weapon::Flamethrower);
        if level == 0 {
            return;
        }
        let stats = flamethrower::at(level);
        let Self {
            player,
            enemies,
            magazines,
            map,
            flame_way,
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
        let from = player.position;
        let nearest = guns::targets(enemies, from, 1, stats.range, |at| in_sight(map, from, at));
        if let Some(target) = nearest
            .first()
            .filter(|target| (**target - from).length() > f32::EPSILON)
        {
            // Lit, right at it; burning, it turns after it.
            *flame_way = if lit {
                flamethrower::turn(*flame_way, *target - from, TURN_DEGREES.to_radians())
            } else {
                (*target - from).normalized()
            };
        }
        let way = *flame_way;
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
}
