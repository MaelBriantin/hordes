//! What the player picks on levelling up: weapons and passives, each up to level 5. A level-up
//! offers three of what can still be picked, at random.

use rand::{Rng, RngExt};

pub const MAX_LEVEL: u8 = 5;
const MAX_WEAPONS: usize = 4;
const MAX_PASSIVES: usize = 4;
/// How many choices a level-up offers.
const CHOICES: usize = 3;
/// The player's HP before Vitality.
pub const BASE_HP: u32 = 100;
/// The HP a heal gives back: the choice left when nothing else can be picked.
pub const HEAL: u32 = 20;
/// What each level of Vitality adds to the player's HP.
pub const VITALITY_HP: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weapon {
    Machete,
    Pistol,
    MachineGun,
    Drones,
}

impl Weapon {
    pub const ALL: [Weapon; 4] = [
        Weapon::Machete,
        Weapon::Pistol,
        Weapon::MachineGun,
        Weapon::Drones,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Passive {
    /// +20 HP.
    Vitality,
    /// +8% speed.
    Swiftness,
    /// +1 tile of magnet radius.
    Magnet,
    /// +10% damage.
    Might,
    /// −8% to weapons' cooldowns.
    Haste,
    /// +2 tiles of light around the player.
    Flashlight,
}

impl Passive {
    pub const ALL: [Passive; 6] = [
        Passive::Vitality,
        Passive::Swiftness,
        Passive::Magnet,
        Passive::Might,
        Passive::Haste,
        Passive::Flashlight,
    ];
}

/// What a level-up offers: a weapon or passive, new or its next level; or a heal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Weapon(Weapon),
    Passive(Passive),
    Heal,
}

/// The weapons and passives the player has, with their levels, in the order picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    weapons: Vec<(Weapon, u8)>,
    passives: Vec<(Passive, u8)>,
}

impl Build {
    /// What a run starts with: the machete.
    pub fn new() -> Self {
        Self {
            weapons: vec![(Weapon::Machete, 1)],
            passives: Vec::new(),
        }
    }

    /// No weapon at all: to watch enemies without them falling.
    #[cfg(test)]
    pub fn unarmed() -> Self {
        Self {
            weapons: Vec::new(),
            passives: Vec::new(),
        }
    }

    /// A weapon's level, 0 without it.
    pub fn weapon_level(&self, weapon: Weapon) -> u8 {
        level_of(&self.weapons, weapon)
    }

    /// A passive's level, 0 without it.
    pub fn passive_level(&self, passive: Passive) -> u8 {
        level_of(&self.passives, passive)
    }

    /// The level a choice brings its weapon or passive to: 1 for a new one, 0 for a heal.
    pub fn next_level(&self, choice: Choice) -> u8 {
        match choice {
            Choice::Weapon(weapon) => self.weapon_level(weapon) + 1,
            Choice::Passive(passive) => self.passive_level(passive) + 1,
            Choice::Heal => 0,
        }
    }

    /// Three choices at random among what can still be picked, all different; a heal if
    /// nothing can.
    pub fn offers(&self, rng: &mut impl Rng) -> Vec<Choice> {
        let mut pool: Vec<Choice> = Vec::new();
        for weapon in Weapon::ALL {
            if can_pick(&self.weapons, weapon, MAX_WEAPONS) {
                pool.push(Choice::Weapon(weapon));
            }
        }
        for passive in Passive::ALL {
            if can_pick(&self.passives, passive, MAX_PASSIVES) {
                pool.push(Choice::Passive(passive));
            }
        }
        if pool.is_empty() {
            return vec![Choice::Heal];
        }
        let count = pool.len().min(CHOICES);
        for i in 0..count {
            let j = rng.random_range(i..pool.len());
            pool.swap(i, j);
        }
        pool.truncate(count);
        pool
    }

    /// Takes a weapon or passive: new at level 1, or its next level. A heal changes nothing here.
    pub fn take(&mut self, choice: Choice) {
        match choice {
            Choice::Weapon(weapon) => raise(&mut self.weapons, weapon),
            Choice::Passive(passive) => raise(&mut self.passives, passive),
            Choice::Heal => {}
        }
    }

    pub fn max_hp(&self) -> u32 {
        BASE_HP + VITALITY_HP * u32::from(self.passive_level(Passive::Vitality))
    }

    /// The player's speed, against their base speed, in percent.
    pub fn speed_percent(&self) -> u16 {
        100 + 8 * u16::from(self.passive_level(Passive::Swiftness))
    }

    /// How near the player gems start to fly to them, in tiles.
    pub fn magnet_radius(&self) -> f32 {
        2.0 + f32::from(self.passive_level(Passive::Magnet))
    }

    /// How far around the player the night is lit, in tiles: 9, and 2 more per level of
    /// Flashlight.
    pub fn light_radius(&self) -> f32 {
        9.0 + 2.0 * f32::from(self.passive_level(Passive::Flashlight))
    }

    /// A weapon's damage, with Might.
    pub fn damage(&self, base: u32) -> u32 {
        base * (100 + 10 * u32::from(self.passive_level(Passive::Might))) / 100
    }

    /// A weapon's cooldown in ticks, with Haste.
    pub fn cooldown(&self, base: u32) -> u32 {
        base * (100 - 8 * u32::from(self.passive_level(Passive::Haste))) / 100
    }
}

fn level_of<T: PartialEq + Copy>(owned: &[(T, u8)], item: T) -> u8 {
    owned
        .iter()
        .find(|(owned, _)| *owned == item)
        .map_or(0, |(_, level)| *level)
}

/// Whether an item can be picked: owned below the top level, or new with room left.
fn can_pick<T: PartialEq + Copy>(owned: &[(T, u8)], item: T, room: usize) -> bool {
    match level_of(owned, item) {
        0 => owned.len() < room,
        level => level < MAX_LEVEL,
    }
}

fn raise<T: PartialEq>(owned: &mut Vec<(T, u8)>, item: T) {
    match owned.iter_mut().find(|(owned, _)| *owned == item) {
        Some((_, level)) => *level = (*level + 1).min(MAX_LEVEL),
        None => owned.push((item, 1)),
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(2)
    }

    #[test]
    fn a_run_starts_with_the_machete_and_no_passive() {
        let build = Build::new();
        assert_eq!(build.weapon_level(Weapon::Machete), 1);
        assert_eq!(build.max_hp(), BASE_HP);
        assert_eq!(build.next_level(Choice::Passive(Passive::Might)), 1);
        assert_eq!(build.next_level(Choice::Weapon(Weapon::Machete)), 2);
        assert_eq!(build.next_level(Choice::Weapon(Weapon::Pistol)), 1);
    }

    #[test]
    fn a_level_up_offers_three_different_choices() {
        let build = Build::new();
        let mut rng = rng();
        for _ in 0..50 {
            let offers = build.offers(&mut rng);
            assert_eq!(offers.len(), 3);
            assert!(offers[0] != offers[1] && offers[1] != offers[2] && offers[0] != offers[2]);
        }
    }

    #[test]
    fn nothing_at_the_top_level_is_offered() {
        let mut build = Build::new();
        for _ in 0..10 {
            build.take(Choice::Weapon(Weapon::Machete));
        }
        assert_eq!(build.weapon_level(Weapon::Machete), MAX_LEVEL);
        let mut rng = rng();
        for _ in 0..50 {
            assert!(
                !build
                    .offers(&mut rng)
                    .contains(&Choice::Weapon(Weapon::Machete))
            );
        }
    }

    #[test]
    fn with_four_passives_no_new_one_is_offered() {
        let mut build = Build::new();
        for passive in &Passive::ALL[..4] {
            build.take(Choice::Passive(*passive));
        }
        let mut rng = rng();
        for _ in 0..50 {
            assert!(
                !build
                    .offers(&mut rng)
                    .contains(&Choice::Passive(Passive::Haste))
            );
        }
    }

    #[test]
    fn with_everything_at_the_top_a_heal_is_offered() {
        let mut build = Build::new();
        for _ in 0..MAX_LEVEL {
            for weapon in Weapon::ALL {
                build.take(Choice::Weapon(weapon));
            }
            for passive in &Passive::ALL[..4] {
                build.take(Choice::Passive(*passive));
            }
        }
        assert_eq!(build.offers(&mut rng()), [Choice::Heal]);
    }

    #[test]
    fn passives_change_the_numbers() {
        let mut build = Build::new();
        for passive in Passive::ALL {
            build.take(Choice::Passive(passive));
            build.take(Choice::Passive(passive));
        }
        assert_eq!(build.max_hp(), 140);
        assert_eq!(build.speed_percent(), 116);
        assert!((build.magnet_radius() - 4.0).abs() < f32::EPSILON);
        assert_eq!(build.damage(10), 12);
        assert_eq!(build.cooldown(60), 50);
        assert!((build.light_radius() - 13.0).abs() < f32::EPSILON);
    }
}
