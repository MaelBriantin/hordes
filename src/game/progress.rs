//! The player's progress, from one run to the next: the achievements reached, and the cards and
//! rerolls they unlock. At first, few cards are dealt; each achievement adds one, or a reroll.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::build::{Action, Choice, Legendary, Passive, Rarity, Weapon};
use super::weather::Weather;

/// How many weathers a run goes through, from the dusk to the tempest.
const WEATHERS: usize = 6;
/// The rerolls of a run before any achievement.
const REROLLS: u8 = 1;
/// The goals counted over every run: the enemies down, the Shadows down, the runs played.
const KILLS_GOAL: u64 = 1_000;
const SHADOWS_GOAL: u64 = 50;
const RUNS_GOAL: u32 = 10;

/// Every card, and the reroll, as the collection shows them: those there from the start, then
/// those unlocked, from the easiest to the hardest to unlock. Not the drones, dealt no more, nor
/// the heal, no card of a build.
pub const COLLECTION: [Unlock; 21] = [
    Unlock::Card(Choice::Weapon(Weapon::Crowbar)),
    Unlock::Card(Choice::Weapon(Weapon::Machete)),
    Unlock::Card(Choice::Weapon(Weapon::Pistol)),
    Unlock::Card(Choice::Weapon(Weapon::Shotgun)),
    Unlock::Card(Choice::Passive(Passive::Vitality)),
    Unlock::Card(Choice::Passive(Passive::Swiftness)),
    Unlock::Card(Choice::Passive(Passive::Magnet)),
    Unlock::Card(Choice::Passive(Passive::Flashlight)),
    Unlock::Card(Choice::Passive(Passive::Loot)),
    Unlock::Card(Choice::Passive(Passive::Might)),
    Unlock::Card(Choice::Passive(Passive::Haste)),
    Unlock::Card(Choice::Passive(Passive::Wisdom)),
    Unlock::Card(Choice::Passive(Passive::HordeHunter)),
    Unlock::Card(Choice::Weapon(Weapon::MachineGun)),
    Unlock::Card(Choice::Weapon(Weapon::Molotov)),
    Unlock::Card(Choice::Action(Action::Dash)),
    Unlock::Reroll,
    Unlock::Card(Choice::Passive(Passive::Glare)),
    Unlock::Card(Choice::Passive(Passive::Regeneration)),
    Unlock::Card(Choice::Legendary(Legendary::Illumination)),
    Unlock::Arcade,
];

/// A goal reached once, kept from one run to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Achievement {
    /// Hunter: a horde wiped out to the last.
    Hunter,
    /// Rain of lead: hold out until the rain.
    Rain,
    /// In the eye: hold out until the storm.
    Storm,
    /// Struck by lightning, and standing.
    Struck,
    /// A thousand down: 1,000 enemies down, every run counted.
    ThousandDown,
    /// Raw light: 50 Shadows down, every run counted.
    Shadows,
    /// Hold out until dawn.
    Dawn,
    /// A regular: 10 runs played.
    Regular,
    /// Come rain or shine: every weather seen once, every run counted, even one lost in it.
    Weathers,
}

/// What an achievement unlocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unlock {
    /// A card, dealt from then on.
    Card(Choice),
    /// One more reroll a run.
    Reroll,
    /// The arcade mode, to be played from then on.
    Arcade,
}

impl Achievement {
    pub const ALL: [Achievement; 9] = [
        Achievement::Hunter,
        Achievement::Rain,
        Achievement::Storm,
        Achievement::Struck,
        Achievement::ThousandDown,
        Achievement::Shadows,
        Achievement::Dawn,
        Achievement::Regular,
        Achievement::Weathers,
    ];

    /// What it unlocks.
    pub fn unlocks(self) -> Unlock {
        match self {
            Achievement::Hunter => Unlock::Card(Choice::Passive(Passive::HordeHunter)),
            Achievement::Rain => Unlock::Card(Choice::Weapon(Weapon::MachineGun)),
            Achievement::Storm => Unlock::Card(Choice::Weapon(Weapon::Molotov)),
            Achievement::Struck => Unlock::Card(Choice::Action(Action::Dash)),
            Achievement::ThousandDown => Unlock::Card(Choice::Passive(Passive::Regeneration)),
            Achievement::Shadows => Unlock::Card(Choice::Passive(Passive::Glare)),
            Achievement::Dawn => Unlock::Card(Choice::Legendary(Legendary::Illumination)),
            Achievement::Regular => Unlock::Reroll,
            Achievement::Weathers => Unlock::Arcade,
        }
    }

    /// How rare it is: as the card it unlocks; a reroll, rare.
    pub fn rarity(self) -> Rarity {
        self.unlocks().rarity()
    }
}

impl Unlock {
    /// How rare it is: as its card; a reroll, rare; the arcade, legendary.
    pub fn rarity(self) -> Rarity {
        match self {
            Unlock::Card(choice) => choice.rarity(),
            Unlock::Reroll => Rarity::Rare,
            Unlock::Arcade => Rarity::Legendary,
        }
    }

    /// The achievement that unlocks it; none for what is there from the start.
    pub fn achievement(self) -> Option<Achievement> {
        Achievement::ALL
            .into_iter()
            .find(|achievement| achievement.unlocks() == self)
    }
}

/// What the player has reached so far, every run counted.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub achieved: BTreeSet<Achievement>,
    /// The enemies down, and the Shadows among them.
    pub kills: u64,
    pub shadows: u64,
    /// The runs played to their end.
    pub runs: u32,
    /// The weathers seen, every run counted.
    pub weathers: BTreeSet<Weather>,
}

impl Progress {
    /// Every achievement reached: every card dealt, every reroll.
    pub fn everything() -> Self {
        Self {
            achieved: Achievement::ALL.into_iter().collect(),
            ..Self::default()
        }
    }

    /// Whether a card may be dealt: unless an achievement not yet reached unlocks it. The
    /// drones are dealt no more, until they are made worth taking.
    pub fn deals(&self, choice: Choice) -> bool {
        if choice == Choice::Weapon(Weapon::Drones) {
            return false;
        }
        Achievement::ALL
            .into_iter()
            .find(|achievement| achievement.unlocks() == Unlock::Card(choice))
            .is_none_or(|achievement| self.achieved.contains(&achievement))
    }

    /// Whether it is there to be had: from the start, or its achievement reached.
    pub fn has(&self, unlock: Unlock) -> bool {
        unlock
            .achievement()
            .is_none_or(|achievement| self.achieved.contains(&achievement))
    }

    /// Where an achievement counted over every run stands: its count so far, and its goal.
    pub fn count_of(&self, achievement: Achievement) -> Option<(u64, u64)> {
        match achievement {
            Achievement::ThousandDown => Some((self.kills, KILLS_GOAL)),
            Achievement::Shadows => Some((self.shadows, SHADOWS_GOAL)),
            Achievement::Regular => Some((u64::from(self.runs), u64::from(RUNS_GOAL))),
            _ => None,
        }
    }

    /// Reaches an achievement, if not yet.
    pub fn reach(&mut self, achievement: Achievement) {
        self.achieved.insert(achievement);
    }

    /// A weather seen: once every one has, the achievement it makes.
    pub fn see(&mut self, weather: Weather) {
        if self.weathers.insert(weather) && self.weathers.len() == WEATHERS {
            self.reach(Achievement::Weathers);
        }
    }

    /// Whether the arcade mode can be played: its achievement reached.
    pub fn arcade(&self) -> bool {
        self.has(Unlock::Arcade)
    }

    /// Reaches the achievements counted over every run, once their counts are.
    pub fn count(&mut self) {
        if self.kills >= KILLS_GOAL {
            self.reach(Achievement::ThousandDown);
        }
        if self.shadows >= SHADOWS_GOAL {
            self.reach(Achievement::Shadows);
        }
        if self.runs >= RUNS_GOAL {
            self.reach(Achievement::Regular);
        }
    }

    /// The rerolls of a run: one, and one more for each achievement unlocking one.
    pub fn rerolls(&self) -> u8 {
        let more = self
            .achieved
            .iter()
            .filter(|achievement| achievement.unlocks() == Unlock::Reroll)
            .count();
        REROLLS + u8::try_from(more).unwrap_or(u8::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_first_four_weapons_the_plain_passives_and_the_heal_are_dealt() {
        let progress = Progress::default();
        for weapon in [Weapon::Crowbar, Weapon::Pistol, Weapon::Shotgun] {
            assert!(progress.deals(Choice::Weapon(weapon)));
        }
        assert!(progress.deals(Choice::Passive(Passive::Vitality)));
        assert!(progress.deals(Choice::Heal));
        assert!(!progress.deals(Choice::Weapon(Weapon::MachineGun)));
        assert!(!progress.deals(Choice::Passive(Passive::Regeneration)));
        assert!(!progress.deals(Choice::Action(Action::Dash)));
        assert!(!progress.deals(Choice::Legendary(Legendary::Illumination)));
        assert_eq!(progress.rerolls(), 1);
    }

    #[test]
    fn an_achievement_unlocks_its_card_or_a_reroll() {
        let mut progress = Progress::default();
        progress.achieved.insert(Achievement::Rain);
        progress.achieved.insert(Achievement::Regular);
        assert!(progress.deals(Choice::Weapon(Weapon::MachineGun)));
        assert_eq!(progress.rerolls(), 2);
    }

    #[test]
    fn the_counted_achievements_are_reached_at_their_counts() {
        let mut progress = Progress {
            kills: KILLS_GOAL - 1,
            shadows: SHADOWS_GOAL,
            runs: RUNS_GOAL - 1,
            ..Progress::default()
        };
        progress.count();
        assert_eq!(progress.achieved, BTreeSet::from([Achievement::Shadows]));
        progress.kills += 1;
        progress.runs += 1;
        progress.count();
        assert!(progress.achieved.contains(&Achievement::ThousandDown));
        assert!(progress.achieved.contains(&Achievement::Regular));
    }

    #[test]
    fn an_achievement_is_as_rare_as_what_it_unlocks() {
        assert_eq!(Achievement::Rain.rarity(), Rarity::Common);
        assert_eq!(Achievement::ThousandDown.rarity(), Rarity::Rare);
        assert_eq!(Achievement::Regular.rarity(), Rarity::Rare);
        assert_eq!(Achievement::Dawn.rarity(), Rarity::Legendary);
    }

    #[test]
    fn the_collection_holds_every_card_dealt_and_the_reroll() {
        let progress = Progress::default();
        let from_start = COLLECTION
            .iter()
            .filter(|unlock| progress.has(**unlock))
            .count();
        assert_eq!(from_start, COLLECTION.len() - Achievement::ALL.len());
        for achievement in Achievement::ALL {
            assert!(COLLECTION.contains(&achievement.unlocks()));
        }
        assert!(!COLLECTION.contains(&Unlock::Card(Choice::Weapon(Weapon::Drones))));
        assert!(!COLLECTION.contains(&Unlock::Card(Choice::Heal)));
        assert!(Progress::everything().has(Unlock::Reroll));
    }

    #[test]
    fn every_weather_seen_over_the_runs_unlocks_the_arcade() {
        let mut progress = Progress::default();
        assert!(!progress.arcade());
        for weather in [
            Weather::Clear,
            Weather::Drizzle,
            Weather::Rain,
            Weather::Storm,
        ] {
            progress.see(weather);
        }
        // Another run, the dusk and the tempest at last.
        progress.see(Weather::Dusk);
        assert!(!progress.arcade());
        progress.see(Weather::Tempest);
        assert!(progress.arcade());
    }

    #[test]
    fn the_drones_are_dealt_no_more() {
        let progress = Progress::everything();
        assert!(!progress.deals(Choice::Weapon(Weapon::Drones)));
        assert!(progress.deals(Choice::Weapon(Weapon::Molotov)));
    }

    #[test]
    fn progress_reads_back_what_it_wrote_and_an_old_file_still_reads() {
        let mut progress = Progress::everything();
        progress.kills = 734;
        let text = ron::to_string(&progress).unwrap();
        assert_eq!(ron::from_str::<Progress>(&text).unwrap(), progress);
        assert_eq!(ron::from_str::<Progress>("(kills: 12)").unwrap().kills, 12);
    }
}
