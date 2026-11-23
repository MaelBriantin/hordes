//! Revival at work: the hero down to 0 HP comes back.

use super::super::build::{Choice, Legendary};
use super::super::revival::{
    self, PERCENT, SHOW_TICKS, SOUL_TICKS, WAVE_REACH, WAVE_TICKS, WAVE_TILES,
};
use super::World;

impl World {
    /// Down to 0 HP, the hero comes back, if they have Revival not yet used: half their HP
    /// back, untouchable a while. Whether they did.
    pub(super) fn revive(&mut self) -> bool {
        if self.player.hp > 0 || !self.player.build.has(Legendary::Revival) || self.revivals > 0 {
            return false;
        }
        self.revivals += 1;
        let most = self.player.max_hp();
        self.player.hp = (most * PERCENT / 100).max(1);
        self.player.invincible = revival::untouchable();
        self.revived = Some(0);
        true
    }

    /// Come back, the soul back into the body, the crosses rise, a wave throws the enemies around
    /// back.
    pub(super) fn come_back(&mut self) {
        let Some(age) = &mut self.revived else {
            return;
        };
        *age += 1;
        let age = *age;
        if age == SOUL_TICKS {
            self.healed_at = Some(self.ticks);
            let ticks = f32::from(u16::try_from(WAVE_TICKS).unwrap_or(1));
            let at = self.player.position;
            for enemy in &mut self.enemies {
                let away = enemy.position - at;
                if away.length() <= WAVE_REACH {
                    enemy.knock = away.normalized() * (WAVE_TILES / ticks);
                    enemy.knocked = WAVE_TICKS;
                }
            }
        }
        if age >= SHOW_TICKS {
            self.revived = None;
        }
    }

    /// Whether a card is spent: Revival, once it has brought the hero back.
    pub fn spent(&self, choice: Choice) -> bool {
        choice == Choice::Legendary(Legendary::Revival) && self.revivals > 0
    }
}
