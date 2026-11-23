//! The sniper rifle at work: its red dot on an enemy far off, then the shot.

use super::super::body::in_sight;
use super::super::build::Weapon;
use super::super::enemy::EnemyKind;
use super::super::sniper::{self, AIM_TICKS, FLIGHT_TICKS, IMPACT_TICKS, Impact, Shot};
use super::World;
use super::weapons::against;

/// How wide the line a piercing bullet sweeps is, each side, in tiles.
const LINE: f32 = 0.5;

impl World {
    /// The sniper rifle: once it can, it aims at the toughest enemy within its range, in sight
    /// past trees and rocks, light or dark, the furthest of those, a red dot on it; a moment
    /// later, the shot, hitting it, and at the last level every enemy on its line. The enemy
    /// gone meanwhile, it aims anew.
    pub(super) fn snipe(&mut self) {
        for impact in &mut self.impacts {
            impact.age += 1;
        }
        self.impacts.retain(|impact| impact.age < IMPACT_TICKS);
        let level = self.player.build.weapon_level(Weapon::Sniper);
        if level == 0 {
            self.aim = None;
            self.shot = None;
            return;
        }
        let stats = sniper::at(level);
        self.sniper_wait = self.sniper_wait.saturating_sub(1);
        let player = self.player.position;
        // The bullet flies on; striking, it hurts.
        if let Some(shot) = &mut self.shot {
            shot.left -= 1;
            if shot.left == 0 {
                let shot = *shot;
                self.shot = None;
                self.bullet_strikes(shot, stats);
            }
            return;
        }
        if let Some((target, left)) = self.aim {
            let Some(at) = self
                .enemies
                .iter()
                .find(|enemy| enemy.id == target && enemy.hp > 0)
                .map(|enemy| enemy.position)
            else {
                self.aim = None;
                return;
            };
            if left > 1 {
                self.aim = Some((target, left - 1));
                return;
            }
            self.aim = None;
            self.shot = Some(Shot {
                from: player,
                to: at,
                target,
                left: FLIGHT_TICKS,
            });
            self.sniper_wait = self.player.build.cooldown(stats.every).max(1);
            return;
        }
        if self.sniper_wait > 0 {
            return;
        }
        let map = &self.map;
        self.aim = self
            .enemies
            .iter()
            .filter(|enemy| {
                enemy.hp > 0
                    && (enemy.position - player).length() <= stats.range
                    && in_sight(map, player, enemy.position)
            })
            .max_by(|a, b| {
                a.hp.cmp(&b.hp).then(
                    (a.position - player)
                        .length()
                        .total_cmp(&(b.position - player).length()),
                )
            })
            .map(|enemy| (enemy.id, AIM_TICKS));
    }

    /// A bullet striking: the enemy aimed at, and at the last level every enemy on its line;
    /// where it strikes, its impact shows.
    fn bullet_strikes(&mut self, shot: Shot, stats: sniper::Sniper) {
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let way = (shot.to - shot.from).normalized();
        let mut struck_at = None;
        for enemy in &mut self.enemies {
            let struck = if stats.pierces {
                let offset = enemy.position - shot.from;
                let along = offset.x * way.x + offset.y * way.y;
                let across = (offset.x * way.y - offset.y * way.x).abs();
                along > 0.0
                    && along <= stats.range
                    && across <= LINE + enemy.kind.stats().half_width
            } else {
                enemy.id == shot.target
            };
            if !struck || enemy.hp == 0 {
                continue;
            }
            if enemy.id == shot.target {
                struck_at = Some(enemy.position);
            }
            let standing = enemy.hp > 0;
            enemy.hit(against(damage, enemy, hunter));
            if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                self.abominations_smashed += 1;
            }
        }
        if let Some(at) = struck_at {
            self.impacts.push(Impact { at, way, age: 0 });
        }
    }

    /// Where the sniper's red dot is, if it is aiming: on the enemy it aims at.
    pub fn sniper_dot(&self) -> Option<super::super::Vec2> {
        let (target, _) = self.aim?;
        self.enemies
            .iter()
            .find(|enemy| enemy.id == target)
            .map(|enemy| enemy.position)
    }
}
