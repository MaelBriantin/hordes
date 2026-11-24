//! The spitters: keeping off the player, spitting globs of acid at them, the globs landing.

use super::super::Vec2;
use super::super::enemy::{Enemy, EnemyKind};
use super::super::molotov::Puddle;
use super::super::spitter::{
    self, BACK, DAMAGE, EVERY_TICKS, FLY_TICKS, Glob, KEEP, PUDDLE_RADIUS, REACH, SPLASH,
    WIND_TICKS,
};
use super::{PLAYER_HALF_WIDTH, World};

impl World {
    /// The globs fly on; landing, they hurt the player still there, and leave a puddle of acid.
    /// The spitters with the player in their reach, having found them, count down to their
    /// spit, then spit a glob at where they stand; out of reach, they start again.
    pub(super) fn spit_acid(&mut self) {
        let mut landed = Vec::new();
        self.globs.retain_mut(|glob| {
            glob.left = glob.left.saturating_sub(1);
            if glob.left == 0 {
                landed.push(glob.to);
            }
            glob.left > 0
        });
        let lasts = spitter::puddle_lasts();
        for at in landed {
            self.puddles.push(Puddle {
                centre: at,
                radius: PUDDLE_RADIUS,
                left: lasts,
                lasts,
                burst: false,
                wall: false,
                hostile: true,
                acid: true,
            });
            let near = (self.player.position - at).length() <= SPLASH + PLAYER_HALF_WIDTH;
            if near
                && !self.unhurt
                && !self.daybreak()
                && !self.untouchable()
                && self.player.invincible == 0
            {
                self.wound_player(DAMAGE);
            }
        }
        let target = self.player.position;
        let calm = self.daybreak();
        for enemy in self
            .enemies
            .iter_mut()
            .filter(|enemy| enemy.kind == EnemyKind::Spitter && enemy.hp > 0)
        {
            let in_reach = (target - enemy.position).length() <= REACH;
            if calm || !enemy.noticed || !in_reach || enemy.stunned > 0 {
                enemy.spit_wait = enemy.spit_wait.max(WIND_TICKS + 1);
                continue;
            }
            enemy.spit_wait = enemy.spit_wait.saturating_sub(1);
            if enemy.spit_wait == 0 {
                self.globs.push(Glob {
                    from: enemy.position,
                    to: target,
                    left: FLY_TICKS,
                });
                enemy.spit_wait = EVERY_TICKS;
            }
        }
    }
}

/// The way an enemy heading `heading` faces: a spitter that has found the player faces them,
/// backing off or standing, but at daybreak.
pub(super) fn facing(enemy: &Enemy, heading: Vec2, target: Vec2, flee: bool) -> Vec2 {
    if enemy.kind == EnemyKind::Spitter && enemy.noticed && !flee {
        enemy.heading(target)
    } else {
        heading
    }
}

/// Where a spitter, `apart` tiles from the player and `toward` them (length 1), goes: on to
/// its distance, back off nearer, still between, or as it pulls back to spit. Its heading and
/// its pace, if it is a spitter.
pub(super) fn keep_off(enemy: &Enemy, apart: f32, toward: Vec2) -> Option<(Vec2, f32)> {
    if enemy.kind != EnemyKind::Spitter {
        return None;
    }
    Some(if enemy.spit_wait <= WIND_TICKS {
        (Vec2::ZERO, 0.0)
    } else if apart > KEEP {
        (toward, 1.0)
    } else if apart < BACK {
        (toward * -1.0, 1.0)
    } else {
        (Vec2::ZERO, 0.0)
    })
}
