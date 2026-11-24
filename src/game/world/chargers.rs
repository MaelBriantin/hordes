//! The chargers: scraping the ground near the player, then charging straight; running into the
//! player, a barricade, an obstacle.

use super::super::Vec2;
use super::super::body::{fits, touch};
use super::super::charger::{
    AHEAD, CHARGE_PACE, CHARGE_TICKS, Charge, DAMAGE, PUSH, REACH, REST_TICKS, SCRAPE_TICKS,
    STUN_TICKS,
};
use super::super::enemy::{Enemy, EnemyKind};
use super::enemies::shove;
use super::{PLAYER_HALF_WIDTH, World};

impl World {
    /// The chargers charging run into what is just ahead: the player, hurt and thrown back, the
    /// charge over; a wooden barricade, smashed, the charge going on; any other obstacle, the
    /// charge over, stunned.
    pub(super) fn ram_chargers(&mut self) {
        for index in 0..self.enemies.len() {
            let enemy = &self.enemies[index];
            let Charge::Charging { way, .. } = enemy.charge else {
                continue;
            };
            if enemy.hp == 0 {
                continue;
            }
            let (at, half) = (enemy.position, enemy.kind.stats().half_width);
            if touch(at, half, self.player.position, PLAYER_HALF_WIDTH) {
                self.enemies[index].charge = Charge::Resting(REST_TICKS);
                self.charged(way);
                continue;
            }
            let ahead = at + way * AHEAD;
            if fits(&self.map, ahead, half, false)
                || (self.smash_barricades(ahead, half) && fits(&self.map, ahead, half, false))
            {
                continue;
            }
            let enemy = &mut self.enemies[index];
            enemy.charge = Charge::Resting(REST_TICKS);
            enemy.stunned = STUN_TICKS;
            enemy.dazed = STUN_TICKS;
        }
    }

    /// A charger charging the `way` given runs into the player: they are hurt, and thrown back.
    fn charged(&mut self, way: Vec2) {
        if self.untouchable() {
            return;
        }
        if !self.unhurt && !self.daybreak() && self.player.invincible == 0 {
            self.wound_player(DAMAGE);
        }
        let push = if self.player.build.hero().perk().thrown() {
            PUSH
        } else {
            0.0
        };
        self.player.position = shove(
            &self.map,
            self.player.position,
            way * push,
            PLAYER_HALF_WIDTH,
            false,
        );
    }
}

/// A charger, `apart` tiles from the player at `target`, `hunts` them: having found them, it
/// scrapes the ground once within its reach, facing them, then charges straight the way they
/// were, then rests a while. Its heading and its pace this tick, while it scrapes or charges.
pub(super) fn charge(
    enemy: &mut Enemy,
    apart: f32,
    target: Vec2,
    hunts: bool,
) -> Option<(Vec2, f32)> {
    if enemy.kind != EnemyKind::Charger || enemy.hp == 0 {
        return None;
    }
    match enemy.charge {
        Charge::Ready if hunts && apart <= REACH => {
            enemy.charge = Charge::Scraping(SCRAPE_TICKS - 1);
            Some((enemy.heading(target), 0.0))
        }
        Charge::Ready => None,
        Charge::Scraping(_) if !hunts => {
            enemy.charge = Charge::Ready;
            None
        }
        Charge::Scraping(left) => {
            enemy.charge = if left > 1 {
                Charge::Scraping(left - 1)
            } else {
                Charge::Charging {
                    way: enemy.heading(target),
                    left: CHARGE_TICKS,
                }
            };
            Some((enemy.heading(target), 0.0))
        }
        Charge::Charging { way, left } => {
            enemy.charge = if left > 1 {
                Charge::Charging {
                    way,
                    left: left - 1,
                }
            } else {
                Charge::Resting(REST_TICKS)
            };
            Some((way, CHARGE_PACE))
        }
        Charge::Resting(left) => {
            enemy.charge = if left > 1 {
                Charge::Resting(left - 1)
            } else {
                Charge::Ready
            };
            None
        }
    }
}
