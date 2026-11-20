//! Debug tools, for the console of debug builds: what comes late in a run, at once.

use super::super::TICKS_PER_SECOND;
use super::super::horde;
use super::super::level::xp_to_next;
use super::super::map::Feature;
use super::super::progress::Achievement;
use super::super::weather;
use super::World;

/// How near the player the debug console sets enemies on fire, in tiles.
const DEBUG_FIRE_REACH: f32 = 8.0;

impl World {
    /// A debug tool: the run jumps to just before the next weather, which is then told; in the
    /// tempest, to the sunrise.
    pub fn debug_next_weather(&mut self) {
        self.ticks = match weather::next_stage_at(self.seconds(), self.mode) {
            Some(next) => next * u32::from(TICKS_PER_SECOND) - 1,
            None => self.ticks.max(weather::sunrise()),
        };
        self.after_a_jump();
    }

    /// After a debug jump in time: no horde owed from the time skipped, the next one in the
    /// middle of its stage.
    pub(super) fn after_a_jump(&mut self) {
        self.horde_warning = None;
        self.next_horde = horde::due_after(self.ticks + horde::warning(), self.mode);
    }

    /// A debug tool: the next horde is announced at once.
    pub fn debug_horde(&mut self) {
        if self.horde_warning.is_none() {
            self.next_horde = self.ticks + horde::warning();
        }
    }

    /// A debug tool: the run jumps to 10 seconds before the dawn.
    pub fn debug_dawn(&mut self) {
        let before = weather::dawn() - 10 * u32::from(TICKS_PER_SECOND);
        self.ticks = self.ticks.max(before);
        self.after_a_jump();
    }

    /// A debug tool: the player gets the XP left to their next level, and its cards.
    pub fn debug_level_up(&mut self) {
        let missing = xp_to_next(self.player.level).saturating_sub(self.player.xp);
        self.pending_levels += self.player.gain(missing);
    }

    /// A debug tool: the enemies within a few tiles of the player catch fire.
    pub fn debug_fire(&mut self) {
        let player = self.player.position;
        for enemy in &mut self.enemies {
            if (enemy.position - player).length() <= DEBUG_FIRE_REACH {
                enemy.ignite();
            }
        }
    }

    /// A debug tool: a burning campfire a few tiles to the player's right.
    pub fn debug_campfire(&mut self) {
        let (x, y) = self.player.position.tile();
        let tile = (x + 4, y);
        self.map.put_feature(tile, Feature::Campfire);
        self.burnt.remove(&tile);
    }

    /// A debug tool: every achievement reached, every card unlocked, from the next run on.
    pub fn debug_unlock_all(&mut self) {
        for achievement in Achievement::ALL {
            self.progress.reach(achievement);
        }
    }

    /// A debug tool: the player's HP all back.
    pub fn debug_heal(&mut self) {
        self.player.hp = self.player.max_hp();
    }
}
