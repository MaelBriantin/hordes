//! The heroes: how they look, and the weapon each starts a run with, theirs alone. The survivor
//! and the classic hero from the start; the others once an achievement is reached.

use super::build::Weapon;
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

    /// The damage taken, against what it would be, in percent.
    pub fn hurt_percent(self) -> u32 {
        if self == Perk::Unshakable { 90 } else { 100 }
    }

    /// Whether a burst throws the hero.
    pub fn thrown(self) -> bool {
        self != Perk::Unshakable
    }

    /// How long a gun takes to reload, against how long it would, in percent.
    pub fn reload_percent(self) -> u32 {
        if self == Perk::Reflexes { 80 } else { 100 }
    }
}

/// A hero, as they look: a setting, the survivor at first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skin {
    /// In a leather jacket and jeans.
    Survivor,
    /// The hero of the first versions, in a red headband.
    Classic,
    /// In a yellow hazmat suit, a dark grey gas mask and boots, a red tank on the back: once
    /// Cleaner is reached.
    Exterminator,
    /// Bald and bearded, in a grey tank top and jeans: once Demolisher is reached.
    Demolisher,
    /// A young police officer, fair hair, a blue uniform: once Cold blood is reached.
    Rookie,
}

impl Skin {
    pub const ALL: [Skin; 5] = [
        Skin::Survivor,
        Skin::Classic,
        Skin::Exterminator,
        Skin::Demolisher,
        Skin::Rookie,
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
            Skin::Rookie => Weapon::Knife,
        }
    }

    /// Their HP before Vitality.
    pub fn hp(self) -> u32 {
        match self {
            Skin::Survivor | Skin::Exterminator => 100,
            Skin::Classic => 90,
            Skin::Demolisher => 140,
            Skin::Rookie => 80,
        }
    }

    /// How much faster than the survivor they go, in percent: less than 0, slower.
    pub fn speed_percent(self) -> i16 {
        match self {
            Skin::Survivor => 0,
            Skin::Classic => 5,
            Skin::Exterminator => -5,
            Skin::Demolisher => -15,
            Skin::Rookie => 10,
        }
    }

    /// What sets them apart besides: the classic hero's damage, the exterminator's fire, the
    /// demolisher's hide, the rookie's reloads; none for the survivor.
    pub fn perk(self) -> Perk {
        match self {
            Skin::Survivor => Perk::Versatile,
            Skin::Classic => Perk::Commando,
            Skin::Exterminator => Perk::Pyromaniac,
            Skin::Demolisher => Perk::Unshakable,
            Skin::Rookie => Perk::Reflexes,
        }
    }

    /// The achievement they wait for; none for those there from the start.
    pub fn achievement(self) -> Option<Achievement> {
        match self {
            Skin::Survivor | Skin::Classic => None,
            Skin::Exterminator => Some(Achievement::Cleaner),
            Skin::Demolisher => Some(Achievement::Demolisher),
            Skin::Rookie => Some(Achievement::ColdBlood),
        }
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Skin::Survivor => "survivor",
            Skin::Classic => "classic",
            Skin::Exterminator => "exterminator",
            Skin::Demolisher => "demolisher",
            Skin::Rookie => "rookie",
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
