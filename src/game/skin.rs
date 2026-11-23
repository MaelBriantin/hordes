//! The heroes: how they look, and the weapon each starts a run with, theirs alone. The survivor
//! from the start; the others once an achievement is reached.

use serde::{Deserialize, Serialize};

use super::build::{Action, Weapon};
use super::progress::Achievement;

/// What sets a hero apart, besides their HP, their speed and their weapon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perk {
    /// No weakness, and nothing more: the measure of the others.
    Versatile,
    /// Every weapon hurts 10% more.
    Commando,
    /// The enemies set on fire burn twice as long.
    Pyromaniac,
    /// 10% less damage taken, and never thrown by a burst.
    Unshakable,
    /// The guns reload 20% faster.
    Reflexes,
    /// A blow in seven dodged, nothing taken.
    Dodge,
    /// Healing heals half again as much, and HP come back of themselves, one every 5 s.
    Healer,
    /// The torch out, they see around them all the same, in green; the enemies lose them at
    /// once, but those right on them; the Shadows come for them a while later.
    Ghost,
    /// Their shield takes a share of the blows of the enemies ahead of them.
    Shield,
}

impl Perk {
    /// What the weapons hurt, against what they would, in percent.
    pub fn damage_percent(self) -> u32 {
        if self == Perk::Commando { 110 } else { 100 }
    }

    /// How long an enemy set on fire burns, against how long it would.
    pub fn burn_times(self) -> u32 {
        if self == Perk::Pyromaniac { 2 } else { 1 }
    }

    /// Whether a burst throws the hero.
    pub fn thrown(self) -> bool {
        self != Perk::Unshakable
    }

    /// What healing gives back, against what it would, in percent: herbs, campfires, the heal.
    pub fn heal_percent(self) -> u32 {
        if self == Perk::Healer { 150 } else { 100 }
    }

    /// The chance of dodging a blow, in percent.
    pub fn dodge_percent(self) -> u32 {
        if self == Perk::Dodge { 15 } else { 0 }
    }

    /// Whether they see in the dark, the torch out, the enemies losing them, the Shadows slow
    /// to come.
    pub fn ghost(self) -> bool {
        self == Perk::Ghost
    }

    /// The damage taken from an enemy ahead of them, against what it would be, in percent.
    pub fn front_hurt_percent(self) -> u32 {
        if self == Perk::Shield { 70 } else { 100 }
    }

    /// How long a gun takes to reload, against how long it would, in percent.
    pub fn reload_percent(self) -> u32 {
        if self == Perk::Reflexes { 80 } else { 100 }
    }
}

/// A hero, as they look: a setting, the survivor at first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Skin {
    /// In a leather jacket and jeans.
    Survivor,
    /// The hero of the first versions, in a red headband: once Legend is reached.
    Classic,
    /// In a yellow hazmat suit, a dark grey gas mask and boots, a red tank on the back: once
    /// Cleaner is reached.
    Exterminator,
    /// Bald and bearded, in a grey tank top and jeans: once Demolisher is reached.
    Demolisher,
    /// A special agent, fair hair, a brown leather jacket; in their second colours a young police
    /// officer's blue uniform: once Cold blood is reached.
    Agent,
    /// A ninja, all in dark, the eyes alone showing: once Shadow hunter is reached.
    Ninja,
    /// A medic: once First aid is reached.
    Medic,
    /// An infiltrator, all in dark, a balaclava, green goggles on the forehead, over the eyes
    /// once the torch is out: once Shadow is reached.
    Infiltrator,
    /// A riot officer, a helmet, a red visor, a shield on the arm: once Hold the line is reached.
    Riot,
}

impl Skin {
    pub const ALL: [Skin; 9] = [
        Skin::Survivor,
        Skin::Classic,
        Skin::Exterminator,
        Skin::Demolisher,
        Skin::Agent,
        Skin::Ninja,
        Skin::Medic,
        Skin::Infiltrator,
        Skin::Riot,
    ];

    /// The hero a weapon belongs to, if it is one's own: never dealt to the others.
    pub fn owning(weapon: Weapon) -> Option<Skin> {
        Skin::ALL.into_iter().find(|skin| skin.weapon() == weapon)
    }

    /// The skin worn: the survivor in place of one still locked.
    pub fn worn(self, unlocked: bool) -> Self {
        if unlocked { self } else { Skin::Survivor }
    }

    /// The weapon they start a run with: theirs alone, never dealt to another hero.
    pub fn weapon(self) -> Weapon {
        match self {
            Skin::Survivor => Weapon::Crowbar,
            Skin::Classic => Weapon::Machete,
            Skin::Exterminator => Weapon::Baton,
            Skin::Demolisher => Weapon::Hammer,
            Skin::Agent => Weapon::Knife,
            Skin::Ninja => Weapon::Katana,
            Skin::Medic => Weapon::Defibrillator,
            Skin::Infiltrator => Weapon::Fists,
            Skin::Riot => Weapon::Truncheon,
        }
    }

    /// Their own action, coming at the run's level 3: theirs alone, never dealt to another hero.
    pub fn action(self) -> Action {
        match self {
            Skin::Survivor => Action::Barricade,
            Skin::Ninja => Action::Dash,
            Skin::Riot => Action::Ram,
            Skin::Demolisher => Action::C4,
            Skin::Medic => Action::Kit,
            Skin::Exterminator => Action::FireWall,
            Skin::Infiltrator => Action::Smoke,
            Skin::Classic => Action::Gatling,
            Skin::Agent => Action::Vault,
        }
    }

    /// Their HP before Vitality.
    pub fn hp(self) -> u32 {
        match self {
            Skin::Survivor | Skin::Exterminator => 100,
            Skin::Classic => 120,
            Skin::Medic | Skin::Infiltrator => 90,
            Skin::Demolisher => 140,
            Skin::Riot => 110,
            Skin::Agent | Skin::Ninja => 80,
        }
    }

    /// How much faster than the survivor they go, in percent: less than 0, slower.
    pub fn speed_percent(self) -> i16 {
        match self {
            Skin::Survivor | Skin::Medic => 0,
            Skin::Classic | Skin::Infiltrator => 5,
            Skin::Exterminator => -5,
            Skin::Demolisher => -15,
            Skin::Riot => -10,
            Skin::Agent => 10,
            Skin::Ninja => 15,
        }
    }

    /// What sets them apart besides: the classic hero's damage, the exterminator's fire, the
    /// demolisher's hide, the special agent's reloads; none for the survivor.
    pub fn perk(self) -> Perk {
        match self {
            Skin::Survivor => Perk::Versatile,
            Skin::Classic => Perk::Commando,
            Skin::Exterminator => Perk::Pyromaniac,
            Skin::Demolisher => Perk::Unshakable,
            Skin::Agent => Perk::Reflexes,
            Skin::Ninja => Perk::Dodge,
            Skin::Medic => Perk::Healer,
            Skin::Infiltrator => Perk::Ghost,
            Skin::Riot => Perk::Shield,
        }
    }

    /// The achievement they wait for; none for those there from the start.
    pub fn achievement(self) -> Option<Achievement> {
        match self {
            Skin::Survivor => None,
            Skin::Classic => Some(Achievement::Legend),
            Skin::Exterminator => Some(Achievement::Cleaner),
            Skin::Demolisher => Some(Achievement::Demolisher),
            Skin::Agent => Some(Achievement::ColdBlood),
            Skin::Ninja => Some(Achievement::ShadowHunter),
            Skin::Medic => Some(Achievement::FirstAid),
            Skin::Infiltrator => Some(Achievement::Shadow),
            Skin::Riot => Some(Achievement::HoldTheLine),
        }
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Skin::Survivor => "survivor",
            Skin::Classic => "classic",
            Skin::Exterminator => "exterminator",
            Skin::Demolisher => "demolisher",
            Skin::Agent => "agent",
            Skin::Ninja => "ninja",
            Skin::Medic => "medic",
            Skin::Infiltrator => "infiltrator",
            Skin::Riot => "riot",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skin_still_locked_is_worn_as_the_survivor() {
        assert_eq!(Skin::Exterminator.worn(false), Skin::Survivor);
        assert_eq!(Skin::Demolisher.worn(true), Skin::Demolisher);
    }
}
