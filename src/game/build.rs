//! What the player picks on levelling up: weapons and passives, each up to level 5. A level-up
//! offers three of what can still be picked, at random.

use rand::{Rng, RngExt};

pub const MAX_LEVEL: u8 = 5;
const MAX_WEAPONS: usize = 4;
const MAX_PASSIVES: usize = 4;
/// How many choices a level-up offers.
const CHOICES: usize = 3;
/// The chance, in percent, that a card of a level-up is a rare one, when there are rare ones to
/// offer.
pub const RARE_PERCENT: u32 = 25;
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
    Shotgun,
}

impl Weapon {
    pub const ALL: [Weapon; 5] = [
        Weapon::Machete,
        Weapon::Pistol,
        Weapon::MachineGun,
        Weapon::Drones,
        Weapon::Shotgun,
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
    /// Points to the nearest healing on the map.
    Compass,
    /// +10% XP from the gems.
    Wisdom,
    /// +25% to the torch's battery: it lasts longer, and charges faster.
    Batteries,
    /// +10% chance for an enemy to leave a gem.
    Loot,
}

impl Passive {
    pub const ALL: [Passive; 10] = [
        Passive::Vitality,
        Passive::Swiftness,
        Passive::Magnet,
        Passive::Might,
        Passive::Haste,
        Passive::Flashlight,
        Passive::Compass,
        Passive::Wisdom,
        Passive::Batteries,
        Passive::Loot,
    ];
}

/// A rare bonus, found on a golden card now and then: one level only, and no room taken among the
/// passives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Legendary {
    /// +1 XP a second.
    Illumination,
}

impl Legendary {
    pub const ALL: [Legendary; 1] = [Legendary::Illumination];
}

impl Choice {
    /// How rare its card is.
    pub fn rarity(self) -> Rarity {
        match self {
            Choice::Passive(
                Passive::Might
                | Passive::Haste
                | Passive::Compass
                | Passive::Wisdom
                | Passive::Batteries,
            ) => Rarity::Rare,
            Choice::Legendary(_) => Rarity::Legendary,
            Choice::Weapon(_) | Choice::Passive(_) | Choice::Heal => Rarity::Common,
        }
    }
}

/// How rare a card is: rare cards come a time in four, legendaries now and then.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
    Legendary,
}

/// What a level-up offers: a weapon or passive, new or its next level; or a heal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Weapon(Weapon),
    Passive(Passive),
    Legendary(Legendary),
    Heal,
}

/// The weapons and passives the player has, with their levels, in the order picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    weapons: Vec<(Weapon, u8)>,
    passives: Vec<(Passive, u8)>,
    legendaries: Vec<Legendary>,
}

impl Build {
    /// What a run starts with: the machete.
    pub fn new() -> Self {
        Self {
            weapons: vec![(Weapon::Machete, 1)],
            passives: Vec::new(),
            legendaries: Vec::new(),
        }
    }

    /// No weapon at all: to watch enemies without them falling.
    #[cfg(test)]
    pub fn unarmed() -> Self {
        Self {
            weapons: Vec::new(),
            passives: Vec::new(),
            legendaries: Vec::new(),
        }
    }

    /// The weapons the player has, with their levels, in the order picked.
    pub fn weapons(&self) -> &[(Weapon, u8)] {
        &self.weapons
    }

    /// The passives the player has, with their levels, in the order picked.
    pub fn passives(&self) -> &[(Passive, u8)] {
        &self.passives
    }

    /// The legendaries the player has found, in the order found.
    pub fn legendaries(&self) -> &[Legendary] {
        &self.legendaries
    }

    /// Whether the player has found a legendary.
    pub fn has(&self, legendary: Legendary) -> bool {
        self.legendaries.contains(&legendary)
    }

    /// A weapon's level, 0 without it.
    pub fn weapon_level(&self, weapon: Weapon) -> u8 {
        level_of(&self.weapons, weapon)
    }

    /// A passive's level, 0 without it.
    pub fn passive_level(&self, passive: Passive) -> u8 {
        level_of(&self.passives, passive)
    }

    /// The level a choice brings its weapon or passive to: 1 for a new one or a legendary, 0
    /// for a heal.
    pub fn next_level(&self, choice: Choice) -> u8 {
        match choice {
            Choice::Weapon(weapon) => self.weapon_level(weapon) + 1,
            Choice::Passive(passive) => self.passive_level(passive) + 1,
            Choice::Legendary(_) => 1,
            Choice::Heal => 0,
        }
    }

    /// Three choices at random among what can still be picked, all different; a heal if
    /// nothing can. Each card is a rare one `rare_percent` of the time, a common one otherwise (or
    /// the other, if none is left of its rarity). At `dusk`, no Flashlight nor Batteries: there is still light enough.
    /// `legendary_percent` of the time, the last is a legendary not found yet.
    pub fn offers(
        &self,
        rng: &mut impl Rng,
        dusk: bool,
        rare_percent: u32,
        legendary_percent: u32,
    ) -> Vec<Choice> {
        let mut pool: Vec<Choice> = Vec::new();
        for weapon in Weapon::ALL {
            if can_pick(&self.weapons, weapon, MAX_WEAPONS) {
                pool.push(Choice::Weapon(weapon));
            }
        }
        for passive in Passive::ALL {
            if dusk && matches!(passive, Passive::Flashlight | Passive::Batteries) {
                continue;
            }
            if can_pick(&self.passives, passive, MAX_PASSIVES) {
                pool.push(Choice::Passive(passive));
            }
        }
        if pool.is_empty() {
            return vec![Choice::Heal];
        }
        let (mut rare, mut common): (Vec<Choice>, Vec<Choice>) = pool
            .into_iter()
            .partition(|choice| choice.rarity() == Rarity::Rare);
        let mut offer = Vec::new();
        while offer.len() < CHOICES && !(rare.is_empty() && common.is_empty()) {
            let wants_rare = rng.random_range(0..100) < rare_percent;
            let from = if (wants_rare && !rare.is_empty()) || common.is_empty() {
                &mut rare
            } else {
                &mut common
            };
            let index = rng.random_range(0..from.len());
            offer.push(from.swap_remove(index));
        }
        let missing: Vec<Legendary> = Legendary::ALL
            .into_iter()
            .filter(|legendary| !self.has(*legendary))
            .collect();
        if !missing.is_empty() && rng.random_range(0..100) < legendary_percent {
            let legendary = missing[rng.random_range(0..missing.len())];
            if let Some(last) = offer.last_mut() {
                *last = Choice::Legendary(legendary);
            }
        }
        offer
    }

    /// Takes a weapon or passive: new at level 1, or its next level. A heal changes nothing here.
    pub fn take(&mut self, choice: Choice) {
        match choice {
            Choice::Weapon(weapon) => raise(&mut self.weapons, weapon),
            Choice::Passive(passive) => raise(&mut self.passives, passive),
            Choice::Legendary(legendary) => {
                if !self.has(legendary) {
                    self.legendaries.push(legendary);
                }
            }
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

    /// How far the compass finds healing, in tiles: 150, and 50 more per level after the first;
    /// none without it. Healing is rare: within 150 tiles, there is nearly always some.
    pub fn compass_range(&self) -> Option<f32> {
        match self.passive_level(Passive::Compass) {
            0 => None,
            level => Some(100.0 + 50.0 * f32::from(level)),
        }
    }

    /// The torch's battery, against its own, in percent: how long it lasts, and how fast it
    /// charges; 25 more per level of Batteries.
    pub fn battery_percent(&self) -> u32 {
        100 + 25 * u32::from(self.passive_level(Passive::Batteries))
    }

    /// How much likelier an enemy is to leave a gem, in points of percent: 10 per level of Loot.
    pub fn drop_bonus(&self) -> u32 {
        10 * u32::from(self.passive_level(Passive::Loot))
    }

    /// The XP the gems give, against their own, in percent: 10 more per level of Wisdom.
    pub fn xp_percent(&self) -> u32 {
        100 + 10 * u32::from(self.passive_level(Passive::Wisdom))
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
            let offers = build.offers(&mut rng, false, RARE_PERCENT, 5);
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
                    .offers(&mut rng, false, RARE_PERCENT, 5)
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
                    .offers(&mut rng, false, RARE_PERCENT, 5)
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
        assert_eq!(
            build.offers(&mut rng(), false, RARE_PERCENT, 5),
            [Choice::Heal]
        );
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

    #[test]
    fn no_flashlight_is_offered_at_dusk() {
        let mut rng = rng();
        let build = Build::new();
        for _ in 0..200 {
            assert!(
                !build
                    .offers(&mut rng, true, RARE_PERCENT, 5)
                    .contains(&Choice::Passive(Passive::Flashlight))
            );
        }
    }

    #[test]
    fn a_legendary_comes_now_and_then_and_only_once() {
        let mut build = Build::new();
        let mut rng = rng();
        let legendary = Choice::Legendary(Legendary::Illumination);
        let found = (0..1_000)
            .filter(|_| {
                build
                    .offers(&mut rng, false, RARE_PERCENT, 5)
                    .contains(&legendary)
            })
            .count();
        assert!((20..=80).contains(&found), "{found} in 1000");
        build.take(legendary);
        assert!(build.has(Legendary::Illumination));
        assert!((0..500).all(|_| {
            !build
                .offers(&mut rng, false, RARE_PERCENT, 5)
                .contains(&legendary)
        }));
    }

    #[test]
    fn wisdom_raises_the_xp_of_gems() {
        let mut build = Build::new();
        assert_eq!(build.xp_percent(), 100);
        build.take(Choice::Passive(Passive::Wisdom));
        build.take(Choice::Passive(Passive::Wisdom));
        assert_eq!(build.xp_percent(), 120);
    }

    #[test]
    fn rare_cards_come_a_time_in_four() {
        let build = Build::new();
        let mut rng = rng();
        let cards: Vec<Choice> = (0..1_000)
            .flat_map(|_| build.offers(&mut rng, false, RARE_PERCENT, 0))
            .collect();
        let rare = cards
            .iter()
            .filter(|card| card.rarity() == Rarity::Rare)
            .count();
        assert!(
            (600..=900).contains(&rare),
            "{rare} rare of {}",
            cards.len()
        );
    }
}
