//! The achievements of a run, reached as it goes.

use super::{
    Achievement, CARNAGE_KILLS, COLD_BLOOD_PERCENT, COLD_BLOOD_SECONDS, DEMOLISHER_ABOMINATIONS,
    FIRST_AID_HP, HOLD_THE_LINE_ENEMIES, HOLD_THE_LINE_REACH, HOLD_THE_LINE_SECONDS, LEGEND_KILLS,
    SHADOW_HUNTER_SHADOWS, SHADOW_SECONDS, TICKS_PER_SECOND, VETERAN_LEVEL, Weather, World,
};

impl World {
    /// The achievements of the run reached so far: the weather held out until, and those
    /// counted over every run. The cards they unlock are dealt at once.
    pub(super) fn reach_goals(&mut self) {
        let weather = self.weather();
        if weather >= Weather::Drizzle {
            self.progress.reach(Achievement::Drizzle);
        }
        if matches!(weather, Weather::Rain | Weather::Storm | Weather::Tempest) {
            self.progress.reach(Achievement::Rain);
        }
        if self.kills >= CARNAGE_KILLS {
            self.progress.reach(Achievement::Carnage);
        }
        // Low on HP and standing, a while, every tick counted.
        let (hp, most) = (self.player.hp, self.player.max_hp());
        if hp > 0 && hp * 100 < most * COLD_BLOOD_PERCENT {
            self.low_hp_ticks += 1;
        }
        if self.low_hp_ticks >= COLD_BLOOD_SECONDS * u32::from(TICKS_PER_SECOND) {
            self.progress.reach(Achievement::ColdBlood);
        }
        if self.healed >= FIRST_AID_HP {
            self.progress.reach(Achievement::FirstAid);
        }
        // The torch out, the night fallen, every tick counted.
        if self.torch_on() == Some(false) {
            self.dark_ticks += 1;
            self.dark_streak += 1;
        } else {
            self.dark_streak = 0;
        }
        self.goggles_out = self.goggles_out.saturating_sub(1);
        // Surrounded, every tick counted.
        let player = self.player.position;
        let around = self
            .enemies
            .iter()
            .filter(|enemy| {
                enemy.hp > 0 && (enemy.position - player).length() <= HOLD_THE_LINE_REACH
            })
            .count();
        if around >= HOLD_THE_LINE_ENEMIES {
            self.surrounded_ticks += 1;
        }
        if self.surrounded_ticks >= HOLD_THE_LINE_SECONDS * u32::from(TICKS_PER_SECOND) {
            self.progress.reach(Achievement::HoldTheLine);
        }
        if self.dark_ticks >= SHADOW_SECONDS * u32::from(TICKS_PER_SECOND) {
            self.progress.reach(Achievement::Shadow);
        }
        if self.kills >= LEGEND_KILLS {
            self.progress.reach(Achievement::Legend);
        }
        if self.shadows_down >= SHADOW_HUNTER_SHADOWS {
            self.progress.reach(Achievement::ShadowHunter);
        }
        if self.abominations_smashed >= DEMOLISHER_ABOMINATIONS {
            self.progress.reach(Achievement::Demolisher);
        }
        if self.player.level >= VETERAN_LEVEL {
            self.progress.reach(Achievement::Veteran);
        }
        if matches!(weather, Weather::Storm | Weather::Tempest) {
            self.progress.reach(Achievement::Storm);
        }
        self.progress.count();
    }
}
