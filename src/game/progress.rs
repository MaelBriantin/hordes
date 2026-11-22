//! The player's progress, from one run to the next: the achievements reached, and the cards and
//! rerolls they unlock. At first, few cards are dealt; each achievement adds one, or a reroll.

use std::collections::BTreeSet;

use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize};

use super::build::{Action, Choice, Legendary, Passive, Rarity, Weapon};
use super::skin::Skin;

/// The rerolls of a run before any achievement.
const REROLLS: u8 = 1;
/// The goals counted over every run: the enemies down, the Shadows down, the runs played.
const KILLS_GOAL: u64 = 1_000;
const SHADOWS_GOAL: u64 = 50;
const RUNS_GOAL: u32 = 25;
/// The enemies down in a single run for Carnage; the hordes wiped out in a single run for Swarm.
pub const CARNAGE_KILLS: u32 = 300;
pub const SWARM_HORDES: u32 = 2;
/// The abominations to fell by a melee blow in a single run for Demolisher.
pub const DEMOLISHER_ABOMINATIONS: u32 = 10;
/// The share of their HP the player stays under, in percent, and the seconds they do in a single
/// run, all counted, for Cold blood.
pub const COLD_BLOOD_PERCENT: u32 = 30;
pub const COLD_BLOOD_SECONDS: u32 = 60;
/// The level to reach in a single run for Veteran.
pub const VETERAN_LEVEL: u32 = 25;
/// The seconds without a hit, the night fallen, for Untouchable.
pub const UNTOUCHABLE_SECONDS: u32 = 120;

/// Every card but the heroes' own weapons, and the reroll, as the collection shows them: those there from the start, then
/// those unlocked, from the easiest to the hardest to unlock, those of hidden achievements last. Not the heal, no card of a build.
pub const COLLECTION: [Unlock; 22] = [
    Unlock::Card(Choice::Weapon(Weapon::Pistol)),
    Unlock::Card(Choice::Weapon(Weapon::Shotgun)),
    Unlock::Card(Choice::Passive(Passive::Vitality)),
    Unlock::Card(Choice::Passive(Passive::Swiftness)),
    Unlock::Card(Choice::Passive(Passive::Magnet)),
    Unlock::Card(Choice::Passive(Passive::Flashlight)),
    Unlock::Card(Choice::Passive(Passive::Loot)),
    Unlock::Card(Choice::Passive(Passive::Wisdom)),
    Unlock::Card(Choice::Passive(Passive::HordeHunter)),
    Unlock::Card(Choice::Passive(Passive::Haste)),
    Unlock::Card(Choice::Passive(Passive::Might)),
    Unlock::Card(Choice::Weapon(Weapon::Drones)),
    Unlock::Card(Choice::Weapon(Weapon::MachineGun)),
    Unlock::Card(Choice::Weapon(Weapon::Molotov)),
    Unlock::Card(Choice::Action(Action::Barricade)),
    Unlock::Card(Choice::Action(Action::Dash)),
    Unlock::Reroll,
    Unlock::Card(Choice::Passive(Passive::Glare)),
    Unlock::Card(Choice::Passive(Passive::Regeneration)),
    Unlock::Card(Choice::Weapon(Weapon::Flamethrower)),
    Unlock::Card(Choice::Legendary(Legendary::Illumination)),
    Unlock::Arcade,
];

/// A goal reached once, kept from one run to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Achievement {
    /// Hunter: a horde wiped out to the last.
    Hunter,
    /// First drops: hold out until the drizzle.
    Drizzle,
    /// Carnage: 300 enemies down in a single run.
    Carnage,
    /// Swarm: 2 hordes wiped out in a single run.
    Swarm,
    /// Rain of lead: hold out until the rain.
    Rain,
    /// In the eye: hold out until the storm.
    Storm,
    /// Untouchable: 2 minutes without a hit, the night fallen.
    Untouchable,
    /// Struck by lightning, and standing.
    Struck,
    /// A thousand down: 1,000 enemies down, every run counted.
    ThousandDown,
    /// Raw light: 50 Shadows down, every run counted.
    Shadows,
    /// Hold out until dawn.
    Dawn,
    /// Cleaner: every horde of a standard run wiped out.
    Cleaner,
    /// Demolisher: 10 abominations felled by a melee blow in a single run.
    Demolisher,
    /// Cold blood: a minute under 30% of HP in a single run, all counted, and standing.
    ColdBlood,
    /// A regular: 25 runs played.
    Regular,
    /// Veteran: level 25 reached in a single run.
    Veteran,
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
    /// A hero, to be played from then on, with their own weapon.
    Skin(Skin),
}

impl Achievement {
    pub const ALL: [Achievement; 16] = [
        Achievement::Hunter,
        Achievement::Drizzle,
        Achievement::Carnage,
        Achievement::Swarm,
        Achievement::Rain,
        Achievement::Storm,
        Achievement::Untouchable,
        Achievement::Struck,
        Achievement::ThousandDown,
        Achievement::Shadows,
        Achievement::Cleaner,
        Achievement::Demolisher,
        Achievement::ColdBlood,
        Achievement::Regular,
        Achievement::Veteran,
        Achievement::Dawn,
    ];

    /// What it unlocks.
    pub fn unlocks(self) -> Unlock {
        match self {
            Achievement::Hunter => Unlock::Card(Choice::Passive(Passive::HordeHunter)),
            Achievement::Drizzle => Unlock::Card(Choice::Passive(Passive::Haste)),
            Achievement::Carnage => Unlock::Card(Choice::Passive(Passive::Might)),
            Achievement::Swarm => Unlock::Card(Choice::Weapon(Weapon::Drones)),
            Achievement::Rain => Unlock::Card(Choice::Weapon(Weapon::MachineGun)),
            Achievement::Storm => Unlock::Card(Choice::Weapon(Weapon::Molotov)),
            Achievement::Untouchable => Unlock::Card(Choice::Action(Action::Barricade)),
            Achievement::Struck => Unlock::Card(Choice::Action(Action::Dash)),
            Achievement::ThousandDown => Unlock::Card(Choice::Passive(Passive::Regeneration)),
            Achievement::Shadows => Unlock::Card(Choice::Passive(Passive::Glare)),
            Achievement::Dawn => Unlock::Arcade,
            Achievement::Cleaner => Unlock::Card(Choice::Weapon(Weapon::Flamethrower)),
            Achievement::Demolisher => Unlock::Skin(Skin::Demolisher),
            Achievement::ColdBlood => Unlock::Skin(Skin::Rookie),
            Achievement::Regular => Unlock::Reroll,
            Achievement::Veteran => Unlock::Card(Choice::Legendary(Legendary::Illumination)),
        }
    }

    /// How rare it is: as the card it unlocks; a reroll, rare.
    pub fn rarity(self) -> Rarity {
        self.unlocks().rarity()
    }

    /// Whether it is hidden until reached, what it unlocks a surprise: the dawn's (the arcade),
    /// the veteran's (Illumination).
    pub fn hidden(self) -> bool {
        matches!(self, Achievement::Dawn | Achievement::Veteran)
    }
}

impl Unlock {
    /// How rare it is: as its card; a reroll, rare; the arcade, legendary.
    pub fn rarity(self) -> Rarity {
        match self {
            Unlock::Card(choice) => choice.rarity(),
            Unlock::Reroll | Unlock::Skin(_) => Rarity::Rare,
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
    /// Those an older version had and this one no longer has are left out as it is read.
    #[serde(deserialize_with = "known")]
    pub achieved: BTreeSet<Achievement>,
    /// The enemies down, and the Shadows among them.
    pub kills: u64,
    pub shadows: u64,
    /// The runs played to their end.
    pub runs: u32,
}

/// The achievements read, but those no longer in the game: a progress kept by an older version
/// still reads, rather than being lost.
fn known<'de, D: Deserializer<'de>>(reader: D) -> Result<BTreeSet<Achievement>, D::Error> {
    /// An achievement as read: one the game has, or one it no longer has.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Read {
        Known(Achievement),
        Gone(IgnoredAny),
    }
    let read = Vec::<Read>::deserialize(reader)?;
    Ok(read
        .into_iter()
        .filter_map(|achievement| match achievement {
            Read::Known(achievement) => Some(achievement),
            Read::Gone(_) => None,
        })
        .collect())
}

impl Progress {
    /// Every achievement reached: every card dealt, every reroll.
    pub fn everything() -> Self {
        Self {
            achieved: Achievement::ALL.into_iter().collect(),
            ..Self::default()
        }
    }

    /// Whether a card may be dealt: unless an achievement not yet reached unlocks it.
    pub fn deals(&self, choice: Choice) -> bool {
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
        assert!(progress.deals(Choice::Passive(Passive::Wisdom)));
        assert!(progress.deals(Choice::Heal));
        assert!(!progress.deals(Choice::Passive(Passive::Might)));
        assert!(!progress.deals(Choice::Passive(Passive::Haste)));
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
        // Every achievement's card in it; not the heroes, who show among the skins.
        let cards = Achievement::ALL
            .iter()
            .filter(|achievement| !matches!(achievement.unlocks(), Unlock::Skin(_)))
            .count();
        assert_eq!(from_start, COLLECTION.len() - cards);
        for achievement in Achievement::ALL {
            let unlock = achievement.unlocks();
            assert!(matches!(unlock, Unlock::Skin(_)) || COLLECTION.contains(&unlock));
        }
        for skin in Skin::ALL {
            assert!(!COLLECTION.contains(&Unlock::Card(Choice::Weapon(skin.weapon()))));
        }
        assert!(!COLLECTION.contains(&Unlock::Card(Choice::Heal)));
        assert!(Progress::everything().has(Unlock::Reroll));
    }

    #[test]
    fn the_dawn_unlocks_the_arcade_and_veteran_illumination() {
        let mut progress = Progress::default();
        let illumination = Choice::Legendary(Legendary::Illumination);
        assert!(!progress.arcade() && !progress.deals(illumination));
        progress.reach(Achievement::Dawn);
        assert!(progress.arcade());
        progress.reach(Achievement::Veteran);
        assert!(progress.deals(illumination));
    }

    #[test]
    fn the_hidden_achievements_come_last_in_the_collection_and_the_achievements() {
        let hidden = |unlock: &Unlock| unlock.achievement().is_some_and(Achievement::hidden);
        let first_hidden = COLLECTION.iter().position(hidden).unwrap();
        assert!(COLLECTION[first_hidden..].iter().all(hidden));
        let first_hidden = Achievement::ALL.iter().position(|a| a.hidden()).unwrap();
        assert!(Achievement::ALL[first_hidden..].iter().all(|a| a.hidden()));
    }

    #[test]
    fn a_progress_holding_an_achievement_no_longer_in_the_game_still_reads() {
        let text = "(achieved: [Hunter, Weathers, Dawn], kills: 40, weathers: [Dusk, Clear])";
        let progress = ron::from_str::<Progress>(text).unwrap();
        assert_eq!(
            progress.achieved,
            BTreeSet::from([Achievement::Hunter, Achievement::Dawn])
        );
        assert_eq!(progress.kills, 40);
    }

    #[test]
    fn the_drones_are_dealt_once_swarm_is_reached() {
        let mut progress = Progress::default();
        assert!(!progress.deals(Choice::Weapon(Weapon::Drones)));
        progress.reach(Achievement::Swarm);
        assert!(progress.deals(Choice::Weapon(Weapon::Drones)));
        let progress = Progress::everything();
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
