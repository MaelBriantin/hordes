//! The shield ram at work: the player charging ahead, the crowd parting in a V.

use super::super::body::touch;
use super::super::build::Action;
use super::super::dash::Dash;
use super::super::ram::{self, REACH, THROW, THROW_TICKS, WINDUP_TICKS};
use super::weapons::against;
use super::{PLAYER_HALF_WIDTH, World};

impl World {
    /// The player braces a moment, standing still, then charges straight ahead the way they
    /// faced, shield first, if they have the ram and it is ready: nothing hurts them while they
    /// charge.
    pub(super) fn ram(&mut self) {
        let level = self.player.build.action_level(Action::Ram);
        if self.announcement.is_some()
            || level == 0
            || self.ram_wait > 0
            || self.player.dash.is_some()
            || self.ram_windup.is_some()
        {
            return;
        }
        let stats = ram::at(level);
        self.ram_windup = Some((self.player.facing.way(), WINDUP_TICKS));
        self.ram_wait = self.player.build.cooldown(stats.recharge);
    }

    /// Whether the player stands braced, about to charge.
    pub fn bracing(&self) -> bool {
        self.ram_windup.is_some()
    }

    /// The ram's wait goes down; charging, every enemy the player runs into is hurt, stunned a
    /// moment and thrown aside, to its side of the charge and a little on, each once.
    pub(super) fn charge(&mut self) {
        self.ram_wait = self.ram_wait.saturating_sub(1);
        // Braced long enough, off they go.
        if let Some((way, left)) = &mut self.ram_windup {
            *left -= 1;
            if *left == 0 {
                let tiles = ram::at(self.player.build.action_level(Action::Ram).max(1)).tiles;
                self.player.dash = Some(Dash::of(*way, tiles));
                self.ramming = true;
                self.rammed.clear();
                self.ram_windup = None;
            }
            return;
        }
        let Some(dash) = self.player.dash.filter(|_| self.ramming) else {
            self.ramming = false;
            return;
        };
        let stats = ram::at(self.player.build.action_level(Action::Ram).max(1));
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let player = self.player.position;
        let ticks = f32::from(u16::try_from(THROW_TICKS).unwrap_or(1));
        for enemy in &mut self.enemies {
            if self.rammed.contains(&enemy.id)
                || !touch(
                    enemy.position,
                    enemy.kind.stats().half_width + REACH,
                    player,
                    PLAYER_HALF_WIDTH,
                )
            {
                continue;
            }
            self.rammed.push(enemy.id);
            enemy.hit(against(damage, enemy, hunter));
            enemy.stunned = enemy.stunned.max(stats.stun);
            enemy.knock = ram::throw(dash.way, enemy.position - player) * (THROW / ticks);
            enemy.knocked = THROW_TICKS;
        }
    }
}
