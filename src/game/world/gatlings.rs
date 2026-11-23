//! The gatling burst at work, the veteran's own action.

use super::super::build::Action;
use super::super::gatling::{self, CASE_TICKS, Case, INTERVAL};
use super::super::guns::Bullet;
use super::super::vec2::Vec2;
use super::World;

/// A spent case: how far ahead it leaves the gun, how far aside it lands at most, how far down
/// at the veteran's feet, in tiles.
const CASE_OUT: f32 = 1.0;
const CASE_ASIDE: f32 = 0.5;
const CASE_DROP: f32 = 1.0;

impl World {
    /// The veteran starts their gatling burst, if it is ready; then it waits its recharge.
    pub(super) fn start_gatling(&mut self) {
        let level = self.player.build.action_level(Action::Gatling);
        if self.announcement.is_some() || level == 0 || self.gatling_wait > 0 {
            return;
        }
        let stats = gatling::at(level);
        self.gatling_left = stats.burst;
        self.gatling_wait = self.player.build.cooldown(stats.recharge).max(1);
    }

    /// While it fires, a bullet every few ticks straight ahead, the way the veteran faces, a
    /// case falling at their feet; its recharge runs down, the cases lie a while.
    pub(super) fn fire_gatling(&mut self) {
        self.gatling_wait = self.gatling_wait.saturating_sub(1);
        for case in &mut self.cases {
            case.age += 1;
        }
        self.cases.retain(|case| case.age < CASE_TICKS);
        if self.gatling_left == 0 {
            return;
        }
        self.gatling_left -= 1;
        if !self.gatling_left.is_multiple_of(INTERVAL) {
            return;
        }
        let stats = gatling::at(self.player.build.action_level(Action::Gatling).max(1));
        let gun = gatling::gun(stats);
        let damage = self.player.build.damage(stats.damage);
        let (from, way) = (self.player.position, self.player.facing.way());
        self.bullets
            .push(Bullet::toward(from, from + way, &gun, damage));
        // Out of the gun, down at the veteran's feet, a little to one side or the other.
        let aside =
            (f32::from(u16::try_from(self.ticks % 7).unwrap_or(0)) / 3.0 - 1.0) * CASE_ASIDE;
        let side = Vec2::new(-way.y, way.x);
        self.cases.push(Case {
            from: from + way * CASE_OUT + Vec2::new(0.0, CASE_DROP / 2.0),
            to: from + side * aside + Vec2::new(0.0, CASE_DROP),
            age: 0,
        });
    }

    /// Whether the veteran is firing their gatling: they go slower meanwhile.
    pub fn firing_gatling(&self) -> bool {
        self.gatling_left > 0
    }
}
