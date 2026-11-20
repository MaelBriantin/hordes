//! The player's weapons at work: the melee swings, the guns and their bullets, the drones, the
//! Molotov's bottles and puddles.

use rand::RngExt;

use super::super::body::{slide, touch};
use super::super::build::Weapon;
use super::super::enemy::Enemy;
use super::super::guns::{self, Bullet, Magazine};
use super::super::machete::{self, Melee, Swing};
use super::super::molotov::{self, Bottle, Puddle};
use super::super::{Vec2, drones};
use super::{Swinging, World};

/// The weapons that are guns, in the order they show.
const GUNS: [Weapon; 3] = [Weapon::Pistol, Weapon::MachineGun, Weapon::Shotgun];

impl World {
    /// The Molotov: bottles fly on and burst into puddles; puddles burn on, setting every enemy
    /// in them on fire; once it can, and an enemy is near, bottles are thrown again. Not in the
    /// dark: nothing to aim at.
    pub(super) fn throw_molotovs(&mut self) {
        let mut landed = Vec::new();
        self.bottles.retain_mut(|bottle| {
            bottle.left = bottle.left.saturating_sub(1);
            if bottle.left == 0 {
                landed.push(Puddle {
                    centre: bottle.to,
                    radius: bottle.radius,
                    left: bottle.lasts,
                    lasts: bottle.lasts,
                });
            }
            bottle.left > 0
        });
        self.puddles.extend(landed);
        for puddle in &mut self.puddles {
            puddle.left = puddle.left.saturating_sub(1);
            for enemy in &mut self.enemies {
                if (enemy.position - puddle.centre).length() <= puddle.radius {
                    enemy.ignite();
                }
            }
        }
        self.puddles.retain(|puddle| puddle.left > 0);
        let level = self.player.build.weapon_level(Weapon::Molotov);
        if level == 0 {
            return;
        }
        self.molotov_wait = self.molotov_wait.saturating_sub(1);
        let dark = self.torch_on().is_some() && !self.torch_lit();
        if self.molotov_wait > 0 || dark {
            return;
        }
        let player = self.player.position;
        let mut near: Vec<Vec2> = self
            .enemies
            .iter()
            .map(|enemy| enemy.position)
            .filter(|at| (*at - player).length() <= molotov::RANGE)
            .collect();
        if near.is_empty() {
            return;
        }
        let throw = molotov::at(level);
        for _ in 0..throw.bottles.min(near.len()) {
            let to = near.swap_remove(self.rng.random_range(0..near.len()));
            self.bottles.push(Bottle {
                from: player,
                to,
                left: molotov::FLIGHT_TICKS,
                radius: throw.radius,
                lasts: throw.lasts,
            });
        }
        self.molotov_wait = self.player.build.cooldown(throw.every);
    }

    /// The melee weapons swing once they can and an enemy is in reach; the slashes of a swing
    /// follow each other, ahead then behind in turn.
    pub(super) fn swing_melee(&mut self) {
        if let Some(swing) = &mut self.swing {
            swing.left -= 1;
            if swing.left == 0 {
                self.swing = None;
            }
        }
        for weapon in [Weapon::Crowbar, Weapon::Machete] {
            let level = self.player.build.weapon_level(weapon);
            let Some(stats) = machete::melee(weapon, level).filter(|_| level > 0) else {
                continue;
            };
            let mut swinging = self.melee.get(&weapon).copied().unwrap_or_default();
            self.swing_once(weapon, stats, &mut swinging);
            self.melee.insert(weapon, swinging);
        }
    }

    /// One melee weapon on by a tick: its next slash, or a new swing.
    pub(super) fn swing_once(&mut self, weapon: Weapon, stats: Melee, swinging: &mut Swinging) {
        swinging.wait = swinging.wait.saturating_sub(1);
        if swinging.left > 0 {
            if swinging.gap > 0 {
                swinging.gap -= 1;
            } else {
                self.slash(weapon, stats, swinging);
                swinging.left -= 1;
                swinging.gap = machete::SLASH_GAP - 1;
            }
            return;
        }
        if swinging.wait > 0 {
            return;
        }
        let player = self.player.position;
        let near = self.enemies.iter().any(|enemy| {
            machete::in_reach(
                enemy.position - player,
                enemy.kind.stats().half_width,
                stats.reach,
            )
        });
        if !near {
            return;
        }
        swinging.done = 0;
        self.slash(weapon, stats, swinging);
        swinging.left = stats.slashes - 1;
        swinging.gap = machete::SLASH_GAP - 1;
        swinging.wait = self.player.build.cooldown(stats.cooldown);
    }

    /// A slash: it hits every enemy in its arc, the way the player faces for a swing's first and
    /// third slashes, behind them for its second; pushing them back, stunning them, as the
    /// weapon does.
    pub(super) fn slash(&mut self, weapon: Weapon, stats: Melee, swinging: &mut Swinging) {
        let player = self.player.position;
        let facing = if swinging.done % 2 == 1 {
            self.player.facing.opposite()
        } else {
            self.player.facing
        };
        swinging.done += 1;
        let swing = Swing {
            weapon,
            facing,
            left: machete::SWING_TICKS,
            reach: stats.reach,
            half_arc: stats.half_arc,
        };
        let ahead = swing.facing.way();
        let damage = self.player.build.damage(stats.damage);
        let hunter = self.player.build.horde_bonus();
        for enemy in &mut self.enemies {
            let offset = enemy.position - player;
            let enemy_stats = enemy.kind.stats();
            let hit = machete::in_arc(
                offset,
                enemy_stats.half_width,
                swing.reach,
                ahead,
                swing.half_arc,
            );
            if !hit {
                continue;
            }
            enemy.hit(against(damage, enemy, hunter));
            enemy.stunned = enemy.stunned.max(stats.stun);
            if stats.knockback > 0.0 {
                let away = offset.normalized() * stats.knockback;
                enemy.position = slide(
                    &self.map,
                    enemy.position,
                    away,
                    enemy_stats.half_width,
                    enemy_stats.flies,
                );
            }
        }
        self.swing = Some(swing);
    }

    /// Each gun the player has shoots at the nearest enemies in its range, while its magazine
    /// holds rounds; empty, it reloads.
    pub(super) fn fire_guns(&mut self) {
        // In the dark, the torch out, nothing to aim at: the guns hold their fire.
        let dark = self.torch_on().is_some() && !self.torch_lit();
        let Self {
            player,
            enemies,
            bullets,
            magazines,
            rng,
            ..
        } = self;
        let build = &player.build;
        for weapon in GUNS {
            let level = build.weapon_level(weapon);
            let Some(gun) = guns::gun(weapon, level).filter(|_| level > 0) else {
                continue;
            };
            let magazine = magazines.entry(weapon).or_insert(Magazine {
                rounds: gun.magazine,
                ..Magazine::default()
            });
            if magazine.reloading > 0 {
                magazine.reloading -= 1;
                if magazine.reloading == 0 {
                    magazine.rounds = gun.magazine;
                }
                continue;
            }
            if magazine.wait > 0 {
                magazine.wait -= 1;
                continue;
            }
            if dark {
                continue;
            }
            let targets = guns::targets(enemies, player.position, gun.bullets, gun.range);
            if targets.is_empty() {
                continue;
            }
            let damage = build.damage(gun.damage);
            for target in targets {
                bullets.extend(Bullet::burst(player.position, target, &gun, damage, rng));
            }
            magazine.rounds = magazine.rounds.saturating_sub(1);
            magazine.wait = gun.interval;
            if magazine.rounds == 0 {
                // Reloaded, the gun shoots at once: the reload is its wait.
                magazine.wait = 0;
                magazine.reloading = build.cooldown(gun.reload).max(1);
            }
        }
    }

    /// Bullets fly on, hitting each enemy they touch once; gone once they have gone through as
    /// many as they can, or flown their range.
    pub(super) fn move_bullets(&mut self) {
        let enemies = &mut self.enemies;
        let hunter = self.player.build.horde_bonus();
        self.bullets.retain_mut(|bullet| {
            if !bullet.advance() {
                return false;
            }
            for enemy in enemies.iter_mut() {
                let touched = touch(
                    bullet.position,
                    guns::BULLET_HALF_WIDTH,
                    enemy.position,
                    enemy.kind.stats().half_width,
                );
                if enemy.hp == 0 || !touched || bullet.hit.contains(&enemy.id) {
                    continue;
                }
                enemy.hit(against(bullet.damage, enemy, hunter));
                bullet.hit.push(enemy.id);
                if bullet.pierce == 0 {
                    return false;
                }
                bullet.pierce -= 1;
            }
            true
        });
    }

    /// The guns the player has, each with its magazine, how many rounds it holds full, and the
    /// ticks a whole reload takes.
    pub fn ammo(&self) -> Vec<(Weapon, Magazine, u32, u32)> {
        GUNS.into_iter()
            .filter_map(|weapon| {
                let level = self.player.build.weapon_level(weapon);
                let gun = guns::gun(weapon, level).filter(|_| level > 0)?;
                let magazine = self.magazines.get(&weapon).copied().unwrap_or(Magazine {
                    rounds: gun.magazine,
                    ..Magazine::default()
                });
                let reload = self.player.build.cooldown(gun.reload).max(1);
                Some((weapon, magazine, gun.magazine, reload))
            })
            .collect()
    }

    /// Where the drones are, if the player has them, and whether each is broken.
    pub fn drones(&self) -> Vec<(Vec2, bool)> {
        let level = self.player.build.weapon_level(Weapon::Drones);
        drones::positions(self.player.position, level, self.drone_angle)
            .into_iter()
            .enumerate()
            .map(|(index, position)| {
                let broken = self.drone_repairs.get(index).is_some_and(|left| *left > 0);
                (position, broken)
            })
            .collect()
    }

    /// The drones turn on; each that works hits the first enemy it runs into, pushes it back
    /// from the player, and breaks. A broken drone repairs with time.
    pub(super) fn turn_drones(&mut self) {
        let build = &self.player.build;
        let level = build.weapon_level(Weapon::Drones);
        if level == 0 {
            return;
        }
        self.drone_angle =
            (self.drone_angle + drones::turn_per_tick(level)) % std::f32::consts::TAU;
        let damage = build.damage(drones::damage(level));
        let hunter = build.horde_bonus();
        let repair = build.cooldown(drones::repair());
        let player = self.player.position;
        let drones = self.drones();
        self.drone_repairs.resize(drones.len(), 0);
        for (index, (position, broken)) in drones.into_iter().enumerate() {
            if broken {
                self.drone_repairs[index] -= 1;
                continue;
            }
            let hit = self.enemies.iter_mut().find(|enemy| {
                enemy.hp > 0
                    && touch(
                        position,
                        drones::HALF_WIDTH,
                        enemy.position,
                        enemy.kind.stats().half_width,
                    )
            });
            let Some(enemy) = hit else {
                continue;
            };
            let stats = enemy.kind.stats();
            enemy.hit(against(damage, enemy, hunter));
            let away = (enemy.position - player).normalized() * drones::KNOCKBACK;
            enemy.position = slide(
                &self.map,
                enemy.position,
                away,
                stats.half_width,
                stats.flies,
            );
            self.drone_repairs[index] = repair.max(1);
        }
    }
}

/// A weapon's `damage` to `enemy`: more to one of a horde, by `hunter` percent, as Horde Hunter
/// has it.
pub(super) fn against(damage: u32, enemy: &Enemy, hunter: u32) -> u32 {
    if enemy.horde.is_some() {
        damage * (100 + hunter) / 100
    } else {
        damage
    }
}
