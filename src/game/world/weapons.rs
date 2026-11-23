//! The player's weapons at work: the melee swings, the guns and their bullets, the drones, the
//! Molotov's bottles and puddles.

use rand::RngExt;

use super::super::body::{in_sight, slide, touch};
use super::super::build::Weapon;
use super::super::enemy::{Enemy, EnemyKind};
use super::super::flashball::{self, Burst};
use super::super::guns::{self, Bullet, Magazine};
use super::super::machete::{self, Melee, Swing};
use super::super::molotov::{self, Bottle, Puddle};
use super::super::{
    Vec2, baton, chainsaw, defibrillator, drones, fists, flamethrower, grenade, hammer, katana,
    knife, napalm, sniper,
};
use super::{Swinging, World};

/// The weapons that are guns, in the order they show.
const GUNS: [Weapon; 5] = [
    Weapon::Pistol,
    Weapon::MachineGun,
    Weapon::Shotgun,
    Weapon::Shuriken,
    Weapon::Flashball,
];

/// The ticks a spark shows where a bullet hit a tree or a rock.
const SPARK_TICKS: u32 = 4;

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
                    burst: true,
                    wall: false,
                });
            }
            bottle.left > 0
        });
        self.puddles.extend(landed);
        let burns = self.player.build.burn_ticks();
        for puddle in &mut self.puddles {
            puddle.left = puddle.left.saturating_sub(1);
            for enemy in &mut self.enemies {
                if (enemy.position - puddle.centre).length() <= puddle.radius {
                    enemy.ignite_for(burns);
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
        for weapon in [Weapon::Crowbar, Weapon::Machete, Weapon::Truncheon] {
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
            let standing = enemy.hp > 0;
            enemy.hit(against(damage, enemy, hunter));
            if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                self.abominations_smashed += 1;
            }
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
            map,
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
            // Only those in sight: no shooting through trees and rocks.
            let targets = guns::targets(enemies, player.position, gun.bullets, gun.range, |at| {
                in_sight(map, player.position, at)
            });
            if targets.is_empty() {
                continue;
            }
            let damage = build.damage(gun.damage);
            for target in targets {
                bullets.extend(Bullet::burst(player.position, target, &gun, damage, rng));
            }
            magazine.rounds = magazine.rounds.saturating_sub(1);
            magazine.wait = build.cooldown(gun.interval).max(1);
            if magazine.rounds == 0 {
                // Reloaded, the gun shoots at once: the reload is its wait.
                magazine.wait = 0;
                magazine.reloading = build.reload(gun.reload).max(1);
            }
        }
    }

    /// Bullets fly on, hitting each enemy they touch once; gone once they have gone through as
    /// many as they can, or flown their range.
    pub(super) fn move_bullets(&mut self) {
        let enemies = &mut self.enemies;
        let (map, sparks, now) = (&self.map, &mut self.sparks, self.ticks);
        let hunter = self.player.build.horde_bonus();
        let (bursts, dazes) = (
            &mut self.flash_bursts,
            flashball::stun(self.player.build.weapon_level(Weapon::Flashball)),
        );
        sparks.retain(|&(_, at)| now.saturating_sub(at) < SPARK_TICKS);
        self.bullets.retain_mut(|bullet| {
            if !bullet.advance() {
                return false;
            }
            // A tree or a rock stops it, a spark where it hits.
            if map.stops_shots(bullet.position.tile()) {
                sparks.push((bullet.position, now));
                return false;
            }
            let mut ricochet_from = None;
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
                // The flashball throws its enemy back, stunned; felled, it bursts.
                if bullet.weapon == Weapon::Flashball {
                    let way = bullet.way();
                    let ticks = f32::from(u16::try_from(flashball::THROW_TICKS).unwrap_or(1));
                    enemy.knock = way * (flashball::THROW_TILES / ticks);
                    enemy.knocked = flashball::THROW_TICKS;
                    enemy.stunned = enemy.stunned.max(dazes);
                    enemy.dazed = dazes;
                    if enemy.hp == 0 {
                        bursts.push(Burst {
                            at: enemy.position,
                            way,
                            age: 0,
                        });
                    }
                }
                if bullet.pierce > 0 {
                    bullet.pierce -= 1;
                    continue;
                }
                if !bullet.ricochets() {
                    return false;
                }
                ricochet_from = Some(enemy.position);
                break;
            }
            // A star ricochets on the nearest enemy left near the one it hit, if any.
            if let Some(from) = ricochet_from {
                let next = enemies
                    .iter()
                    .filter(|enemy| enemy.hp > 0 && !bullet.hit.contains(&enemy.id))
                    .map(|enemy| enemy.position)
                    .filter(|at| (*at - from).length() <= guns::RICOCHET_RANGE)
                    .min_by(|a, b| (*a - from).length().total_cmp(&(*b - from).length()));
                let Some(next) = next else {
                    return false;
                };
                bullet.ricochet(next);
            }
            true
        });
    }

    /// The weapons the player has that wait between two blows, the guns apart: the melee
    /// weapons between two swings, the Molotov between two throws, the C4 between two charges,
    /// the napalm between two strikes, the drones while one is broken. Each with the ticks still to wait, out of a whole wait.
    pub fn waits(&self) -> Vec<(Weapon, u32, u32)> {
        let build = &self.player.build;
        let mut waits = Vec::new();
        for weapon in [Weapon::Crowbar, Weapon::Machete, Weapon::Truncheon] {
            let level = build.weapon_level(weapon);
            if let Some(stats) = machete::melee(weapon, level).filter(|_| level > 0) {
                let left = self.melee.get(&weapon).map_or(0, |swinging| swinging.wait);
                waits.push((weapon, left, build.cooldown(stats.cooldown).max(1)));
            }
        }
        let level = build.weapon_level(Weapon::Molotov);
        if level > 0 {
            let every = build.cooldown(molotov::at(level).every).max(1);
            waits.push((Weapon::Molotov, self.molotov_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Defibrillator);
        if level > 0 {
            let every = build.cooldown(defibrillator::at(level).every).max(1);
            waits.push((
                Weapon::Defibrillator,
                self.defibrillator_wait.min(every),
                every,
            ));
        }
        let level = build.weapon_level(Weapon::Katana);
        if level > 0 {
            let every = build.cooldown(katana::at(level).every).max(1);
            waits.push((Weapon::Katana, self.katana_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Knife);
        if level > 0 {
            let every = build.cooldown(knife::at(level).every).max(1);
            waits.push((Weapon::Knife, self.knife_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Fists);
        if level > 0 {
            let every = build.cooldown(fists::at(level).every).max(1);
            waits.push((Weapon::Fists, self.fists_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Baton);
        if level > 0 {
            let every = build.cooldown(baton::at(level).every).max(1);
            waits.push((Weapon::Baton, self.baton_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Hammer);
        if level > 0 {
            let every = build.cooldown(hammer::at(level).every).max(1);
            waits.push((Weapon::Hammer, self.hammer_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Sniper);
        if level > 0 {
            let every = build.cooldown(sniper::at(level).every).max(1);
            waits.push((Weapon::Sniper, self.sniper_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Grenade);
        if level > 0 {
            let every = build.cooldown(grenade::at(level).every).max(1);
            waits.push((Weapon::Grenade, self.grenade_wait.min(every), every));
        }
        let level = build.weapon_level(Weapon::Napalm);
        if level > 0 {
            let every = build.cooldown(napalm::at(level).every).max(1);
            waits.push((Weapon::Napalm, self.napalm_wait.min(every), every));
        }
        waits
    }

    /// The guns the player has, each with its magazine, how many rounds it holds full, and the
    /// ticks a whole reload takes.
    pub fn ammo(&self) -> Vec<(Weapon, Magazine, u32, u32)> {
        GUNS.into_iter()
            .chain([Weapon::Drones])
            .filter_map(|weapon| {
                let level = self.player.build.weapon_level(weapon);
                let gun = if weapon == Weapon::Drones {
                    Some(drones::gun(level))
                } else {
                    guns::gun(weapon, level)
                }
                .filter(|_| level > 0)?;
                let magazine = self.magazines.get(&weapon).copied().unwrap_or(Magazine {
                    rounds: gun.magazine,
                    ..Magazine::default()
                });
                let reload = self.player.build.reload(gun.reload).max(1);
                Some((weapon, magazine, gun.magazine, reload))
            })
            .chain(self.tanks())
            .collect()
    }

    /// The tanks as guns' magazines, the flamethrower's and the chainsaw's, those the player
    /// has: their fuel in ticks, and their filling as a reload.
    fn tanks(&self) -> Vec<(Weapon, Magazine, u32, u32)> {
        let build = &self.player.build;
        let flamethrower = build.weapon_level(Weapon::Flamethrower);
        let chainsaw = build.weapon_level(Weapon::Chainsaw);
        [
            (flamethrower > 0).then(|| {
                (
                    Weapon::Flamethrower,
                    flamethrower::at(flamethrower).tank,
                    flamethrower::refill(),
                )
            }),
            (chainsaw > 0).then(|| (Weapon::Chainsaw, chainsaw::tank(), chainsaw::refill())),
        ]
        .into_iter()
        .flatten()
        .map(|(weapon, full, refill)| {
            let tank = self.magazines.get(&weapon).copied().unwrap_or(Magazine {
                rounds: full,
                ..Magazine::default()
            });
            (weapon, tank, full, build.cooldown(refill).max(1))
        })
        .collect()
    }

    /// Where the drones are, if the player has them.
    pub fn drones(&self) -> Vec<Vec2> {
        let level = self.player.build.weapon_level(Weapon::Drones);
        drones::positions(self.player.position, level, self.drone_angle)
    }

    /// The drones turn on; each shoots at the nearest enemy within its reach, as often as it
    /// can, but in the dark: nothing to aim at; nothing breaks them. Their shots come from one
    /// magazine; empty, they reload it.
    pub(super) fn turn_drones(&mut self) {
        let build = &self.player.build;
        let level = build.weapon_level(Weapon::Drones);
        if level == 0 {
            return;
        }
        self.drone_angle =
            (self.drone_angle + drones::turn_per_tick(level)) % std::f32::consts::TAU;
        let gun = drones::gun(level);
        let damage = build.damage(gun.damage);
        let interval = build.cooldown(gun.interval).max(1);
        let reload = build.reload(gun.reload).max(1);
        let mut magazine = self
            .magazines
            .get(&Weapon::Drones)
            .copied()
            .unwrap_or(Magazine {
                rounds: gun.magazine,
                ..Magazine::default()
            });
        if magazine.reloading > 0 {
            magazine.reloading -= 1;
            if magazine.reloading == 0 {
                magazine.rounds = gun.magazine;
            }
            self.magazines.insert(Weapon::Drones, magazine);
            return;
        }
        let dark = self.torch_on().is_some() && !self.torch_lit();
        let drones = self.drones();
        self.drone_waits.resize(drones.len(), 0);
        for (index, position) in drones.into_iter().enumerate() {
            let wait = &mut self.drone_waits[index];
            *wait = wait.saturating_sub(1);
            if *wait > 0 || dark || magazine.rounds == 0 {
                continue;
            }
            if let Some(&target) =
                guns::targets(&self.enemies, position, 1, drones::reach(), |at| {
                    in_sight(&self.map, position, at)
                })
                .first()
            {
                self.bullets
                    .push(Bullet::toward(position, target, &gun, damage));
                *wait = interval;
                magazine.rounds -= 1;
                if magazine.rounds == 0 {
                    magazine.reloading = reload;
                }
            }
        }
        self.magazines.insert(Weapon::Drones, magazine);
    }
}

/// A weapon's `damage` to `enemy`: more to one of a horde, by `hunter` percent, as Horde Hunter
/// has it.
pub(super) fn against(damage: u32, enemy: &Enemy, hunter: u32) -> u32 {
    if enemy.horde.is_some() {
        super::super::percent_of(damage, 100 + hunter)
    } else {
        damage
    }
}
