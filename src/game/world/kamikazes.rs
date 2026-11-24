//! The kamikazes: their fuses lit near the player, their blasts, the fires they leave burning.

use super::super::c4::{BLAST_TICKS, Blast};
use super::super::enemy::{Enemy, EnemyKind};
use super::super::kamikaze::{
    self, DAMAGE, DARK_REACH, FIRE_RADIUS, FIRES, LAST_TICKS, PUSH, RADIUS, RUSH_PACE, RUSH_TICKS,
};
use super::super::molotov::Puddle;
use super::super::Vec2;
use super::enemies::shove;
use super::{PLAYER_HALF_WIDTH, TORCH_FADE, World};

impl World {
    /// The kamikazes' blasts fade; those felled blow, each blast felling others that blow in
    /// turn, all in the same tick.
    pub(super) fn blow_kamikazes(&mut self) {
        for blast in &mut self.kamikaze_blasts {
            blast.age += 1;
        }
        self.kamikaze_blasts.retain(|blast| blast.age < BLAST_TICKS);
        let mut blown = vec![false; self.enemies.len()];
        loop {
            let blowing: Vec<usize> = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(index, enemy)| {
                    enemy.kind == EnemyKind::Kamikaze && enemy.hp == 0 && !blown[*index]
                })
                .map(|(index, _)| index)
                .collect();
            if blowing.is_empty() {
                return;
            }
            for index in blowing {
                blown[index] = true;
                let at = self.enemies[index].position;
                self.blow(at);
            }
        }
    }

    /// A kamikaze blows at `at`: everything around hurt and thrown a little, the player too,
    /// fires left burning on the ground.
    fn blow(&mut self, at: Vec2) {
        self.kamikaze_blasts.push(Blast {
            at,
            radius: RADIUS,
            age: 0,
        });
        for enemy in &mut self.enemies {
            let away = enemy.position - at;
            let stats = enemy.kind.stats();
            if away.length() > RADIUS + stats.half_width {
                continue;
            }
            enemy.hit(DAMAGE);
            enemy.position = shove(
                &self.map,
                enemy.position,
                away.normalized() * PUSH,
                stats.half_width,
                stats.flies,
            );
        }
        let lasts = kamikaze::fire_lasts();
        for offset in FIRES {
            self.puddles.push(Puddle {
                centre: at + offset,
                radius: FIRE_RADIUS,
                left: lasts,
                lasts,
                burst: false,
                wall: false,
                hostile: true,
            });
        }
        let away = self.player.position - at;
        if away.length() > RADIUS + PLAYER_HALF_WIDTH || self.untouchable() {
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
            away.normalized() * push,
            PLAYER_HALF_WIDTH,
            false,
        );
    }

    /// How near the player a kamikaze lights its fuse, in tiles: as far as the torch's light
    /// shows, fading included, as Glare slows; the torch not lit, a few tiles.
    pub(super) fn kamikaze_reach(&self) -> f32 {
        if self.torch_lit() {
            self.light_radius() + TORCH_FADE
        } else {
            DARK_REACH
        }
    }

    /// Whether the player stands in a kamikaze's fire.
    pub(super) fn in_hostile_fire(&self) -> bool {
        self.puddles.iter().any(|puddle| {
            puddle.hostile && (self.player.position - puddle.centre).length() <= puddle.radius
        })
    }
}

/// A kamikaze within `reach` of the player lights its fuse and rushes at them; `touching` them,
/// or its rush over, it stands still its last moment, then blows. Its pace this tick against
/// its speed, while its fuse burns.
pub(super) fn fuse(
    enemy: &mut Enemy,
    (apart, reach): (f32, f32),
    lights: bool,
    touching: bool,
) -> Option<f32> {
    if enemy.kind != EnemyKind::Kamikaze || enemy.hp == 0 {
        return None;
    }
    if enemy.fuse == 0 && lights && apart <= reach {
        enemy.fuse = RUSH_TICKS + LAST_TICKS;
    }
    if enemy.fuse == 0 {
        return None;
    }
    if touching {
        enemy.fuse = enemy.fuse.min(LAST_TICKS);
    }
    enemy.fuse -= 1;
    if enemy.fuse == 0 {
        enemy.hp = 0;
        enemy.blew = true;
    }
    Some(if enemy.fuse > LAST_TICKS { RUSH_PACE } else { 0.0 })
}
