//! What the player picks on levelling up: weapons and passives, each up to level 5. A level-up
//! offers three of what can still be picked, at random.

use rand::{Rng, RngExt};

use super::skin::Skin;

pub const MAX_LEVEL: u8 = 5;
pub const MAX_WEAPONS: usize = 4;
pub const MAX_PASSIVES: usize = 4;
/// The action cards a run can hold: one, played with Space or Enter.
pub const MAX_ACTIONS: usize = 1;
/// How many choices a level-up offers.
const CHOICES: usize = 3;
/// The chance, in percent, that a card of a level-up is a rare one, when there are rare ones to
/// offer.
pub const RARE_PERCENT: u32 = 25;
/// The survivor's HP before Vitality: the measure of the others'.
pub const BASE_HP: u32 = 100;
/// How long a dash takes to come back, in ticks, by the Dash card's level: 8 s, 6 s, 6 s,
/// 4.5 s, 3 s.
const DASH_RECHARGES: [u32; 5] = [240, 180, 180, 135, 90];
/// The HP a heal gives back: the choice left when nothing else can be picked.
pub const HEAL: u32 = 20;
/// What each level of Vitality adds to the player's HP.
pub const VITALITY_HP: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weapon {
    Crowbar,
    Machete,
    Pistol,
    MachineGun,
    Drones,
    Shotgun,
    Molotov,
    Flamethrower,
    Hammer,
    Baton,
    Knife,
    Katana,
    Shuriken,
    Defibrillator,
    Chainsaw,
    C4,
    Napalm,
    Fists,
}

impl Weapon {
    pub const ALL: [Weapon; 18] = [
        Weapon::Crowbar,
        Weapon::Machete,
        Weapon::Pistol,
        Weapon::MachineGun,
        Weapon::Drones,
        Weapon::Shotgun,
        Weapon::Molotov,
        Weapon::Flamethrower,
        Weapon::Hammer,
        Weapon::Baton,
        Weapon::Knife,
        Weapon::Katana,
        Weapon::Shuriken,
        Weapon::Defibrillator,
        Weapon::Chainsaw,
        Weapon::C4,
        Weapon::Napalm,
        Weapon::Fists,
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
    /// +10% XP from the gems.
    Wisdom,
    /// +10% chance for an enemy to leave a gem.
    Loot,
    /// +0.4 HP a second, of itself.
    Regeneration,
    /// The enemies in the torch's light slowed, 10% per level.
    Glare,
    /// A chance for each enemy felled to give back 1 HP: 10%, 5% more per level.
    Drain,
    /// The torch out, 10% faster per level.
    Stealth,
    /// +10% damage to the hordes per level; their last ones targeted in red, more with the
    /// levels.
    HordeHunter,
}

impl Passive {
    pub const ALL: [Passive; 13] = [
        Passive::Vitality,
        Passive::Swiftness,
        Passive::Magnet,
        Passive::Might,
        Passive::Haste,
        Passive::Flashlight,
        Passive::Wisdom,
        Passive::Loot,
        Passive::Regeneration,
        Passive::Glare,
        Passive::Drain,
        Passive::Stealth,
        Passive::HordeHunter,
    ];
}

/// A card played with Space or Enter, when the player wants: one a run, no more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// A dash; a second at level 3, each back sooner and sooner.
    Dash,
    /// A crate set down just ahead, stopping the enemies that walk until they break it.
    Barricade,
}

impl Action {
    pub const ALL: [Action; 2] = [Action::Dash, Action::Barricade];
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
            Choice::Weapon(
                Weapon::Flamethrower
                | Weapon::Hammer
                | Weapon::Baton
                | Weapon::Chainsaw
                | Weapon::C4
                | Weapon::Napalm,
            )
            | Choice::Passive(
                Passive::Might
                | Passive::Haste
                | Passive::Wisdom
                | Passive::Regeneration
                | Passive::Glare
                | Passive::Drain,
            ) => Rarity::Rare,
            Choice::Legendary(_) => Rarity::Legendary,
            Choice::Weapon(_) | Choice::Passive(_) | Choice::Action(_) | Choice::Heal => {
                Rarity::Common
            }
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

/// What a level-up offers: a weapon, passive or action, new or its next level; or a heal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Weapon(Weapon),
    Passive(Passive),
    Action(Action),
    Legendary(Legendary),
    Heal,
}

/// The weapons and passives the player has, with their levels, in the order picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    weapons: Vec<(Weapon, u8)>,
    passives: Vec<(Passive, u8)>,
    actions: Vec<(Action, u8)>,
    legendaries: Vec<Legendary>,
    /// Every weapon, passive, action and legendary, in the order first picked.
    picked: Vec<Choice>,
    /// The hero playing: their HP, their speed, their perk.
    hero: Skin,
}

impl Build {
    /// What a run starts with, the survivor playing: the crowbar.
    pub fn new() -> Self {
        Self::playing(Skin::Survivor)
    }

    /// What a run starts with, a hero playing: their own weapon at its first level.
    pub fn playing(hero: Skin) -> Self {
        let weapon = hero.weapon();
        Self {
            weapons: vec![(weapon, 1)],
            passives: Vec::new(),
            actions: Vec::new(),
            legendaries: Vec::new(),
            picked: vec![Choice::Weapon(weapon)],
            hero,
        }
    }

    /// The hero playing.
    pub fn hero(&self) -> Skin {
        self.hero
    }

    /// No weapon at all: to watch enemies without them falling.
    #[cfg(test)]
    pub fn unarmed() -> Self {
        Self {
            weapons: Vec::new(),
            passives: Vec::new(),
            actions: Vec::new(),
            legendaries: Vec::new(),
            picked: Vec::new(),
            hero: Skin::Survivor,
        }
    }

    /// How many weapons and passives the player has: against `MAX_WEAPONS` and `MAX_PASSIVES`.
    pub fn counts(&self) -> (usize, usize) {
        (self.weapons.len(), self.passives.len())
    }

    /// Every weapon, passive and legendary the player has, in the order first picked.
    pub fn picked(&self) -> &[Choice] {
        &self.picked
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

    /// An action's level, 0 without it.
    pub fn action_level(&self, action: Action) -> u8 {
        level_of(&self.actions, action)
    }

    /// The level a choice brings its weapon or passive to: 1 for a new one or a legendary, 0
    /// for a heal.
    pub fn next_level(&self, choice: Choice) -> u8 {
        match choice {
            Choice::Weapon(weapon) => self.weapon_level(weapon) + 1,
            Choice::Passive(passive) => self.passive_level(passive) + 1,
            Choice::Action(action) => self.action_level(action) + 1,
            Choice::Legendary(_) => 1,
            Choice::Heal => 0,
        }
    }

    /// Three choices at random among what can still be picked, all different; a heal if
    /// nothing can. Each card is a rare one `rare_percent` of the time, a common one otherwise (or
    /// the other, if none is left of its rarity). At `dusk`, no Flashlight nor Glare: the torch isn't lit yet.
    /// `legendary_percent` of the time, the last is a legendary not found yet.
    pub fn offers(
        &self,
        rng: &mut impl Rng,
        dusk: bool,
        rare_percent: u32,
        legendary_percent: u32,
        deals: impl Fn(Choice) -> bool,
    ) -> Vec<Choice> {
        let mut pool: Vec<Choice> = Vec::new();
        for weapon in Weapon::ALL {
            if can_pick(&self.weapons, weapon, MAX_WEAPONS) && deals(Choice::Weapon(weapon)) {
                pool.push(Choice::Weapon(weapon));
            }
        }
        for passive in Passive::ALL {
            if dusk && matches!(passive, Passive::Flashlight | Passive::Glare) {
                continue;
            }
            if can_pick(&self.passives, passive, MAX_PASSIVES) && deals(Choice::Passive(passive)) {
                pool.push(Choice::Passive(passive));
            }
        }
        // An action once the run has none, or the next level of the one it has.
        for action in Action::ALL {
            if can_pick(&self.actions, action, MAX_ACTIONS) && deals(Choice::Action(action)) {
                pool.push(Choice::Action(action));
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
            .filter(|legendary| !self.has(*legendary) && deals(Choice::Legendary(*legendary)))
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
        if choice != Choice::Heal && !self.picked.contains(&choice) {
            self.picked.push(choice);
        }
        match choice {
            Choice::Weapon(weapon) => raise(&mut self.weapons, weapon),
            Choice::Passive(passive) => raise(&mut self.passives, passive),
            Choice::Action(action) => raise(&mut self.actions, action),
            Choice::Legendary(legendary) => {
                if !self.has(legendary) {
                    self.legendaries.push(legendary);
                }
            }
            Choice::Heal => {}
        }
    }

    /// The player's HP at most: the hero's, and Vitality's.
    pub fn max_hp(&self) -> u32 {
        self.hero.hp() + VITALITY_HP * u32::from(self.passive_level(Passive::Vitality))
    }

    /// The player's speed, against the survivor's, in percent: the hero's, and Swiftness's.
    /// How much faster the player goes with the torch out, in points of percent: 10 per level of
    /// Stealth.
    pub fn stealth_percent(&self) -> u16 {
        10 * u16::from(self.passive_level(Passive::Stealth))
    }

    pub fn speed_percent(&self) -> u16 {
        let swiftness = 8 * i16::from(self.passive_level(Passive::Swiftness));
        u16::try_from(100 + self.hero.speed_percent() + swiftness).unwrap_or(100)
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

    /// The HP the player gets back of themselves each second, in tenths: 4 per level of
    /// Regeneration.
    pub fn regeneration_tenths(&self) -> u32 {
        4 * u32::from(self.passive_level(Passive::Regeneration))
    }

    /// The chance for each enemy felled to give back 1 HP, in percent: none without Drain, 10
    /// at its first level and 5 more each level after.
    pub fn drain_percent(&self) -> u32 {
        match self.passive_level(Passive::Drain) {
            0 => 0,
            level => 5 + 5 * u32::from(level),
        }
    }

    /// How much likelier an enemy is to leave a gem, in points of percent: 10 per level of Loot.
    pub fn drop_bonus(&self) -> u32 {
        10 * u32::from(self.passive_level(Passive::Loot))
    }

    /// The XP the gems give, against their own, in percent: 10 more per level of Wisdom.
    pub fn xp_percent(&self) -> u32 {
        100 + 10 * u32::from(self.passive_level(Passive::Wisdom))
    }

    /// A weapon's damage, with Might and the hero's perk.
    pub fn damage(&self, base: u32) -> u32 {
        let might = 100 + 10 * u32::from(self.passive_level(Passive::Might));
        base * might * self.hero.perk().damage_percent() / 10_000
    }

    /// A gun's reload in ticks, with Haste and the hero's perk.
    pub fn reload(&self, base: u32) -> u32 {
        self.cooldown(base) * self.hero.perk().reload_percent() / 100
    }

    /// How long an enemy set on fire burns, in ticks, with the hero's perk.
    pub fn burn_ticks(&self) -> u32 {
        super::enemy::burn_ticks() * self.hero.perk().burn_times()
    }

    /// How much more the weapons hurt an enemy of a horde, in percent: 10 per level of Horde
    /// Hunter.
    pub fn horde_bonus(&self) -> u32 {
        10 * u32::from(self.passive_level(Passive::HordeHunter))
    }

    /// The share of a horde, its last ones still standing, targeted in red, in percent: none
    /// without Horde Hunter, then a tenth, a quarter (seen in the dark from level 3), a half,
    /// all of it.
    pub fn horde_veil_percent(&self) -> u32 {
        [0, 10, 25, 25, 50, 100][usize::from(self.passive_level(Passive::HordeHunter).min(5))]
    }

    /// Whether the veiled show in the dark too: from level 3 of Horde Hunter.
    pub fn horde_veil_in_dark(&self) -> bool {
        self.passive_level(Passive::HordeHunter) >= 3
    }

    /// How much slower the enemies in the torch's light go, in percent: 10 per level of Glare.
    pub fn glare_percent(&self) -> u32 {
        10 * u32::from(self.passive_level(Passive::Glare))
    }

    /// How many dashes the player can have ready: none without the Dash card, one from its
    /// level 1, two from its level 3, no more.
    pub fn dashes(&self) -> u8 {
        match self.action_level(Action::Dash) {
            0 => 0,
            1 | 2 => 1,
            _ => 2,
        }
    }

    /// How long a dash takes to come back, in ticks, by the Dash card's level.
    pub fn dash_recharge(&self) -> u32 {
        let level = usize::from(self.action_level(Action::Dash).max(1));
        DASH_RECHARGES[(level - 1).min(DASH_RECHARGES.len() - 1)]
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
    fn a_run_starts_with_the_crowbar_and_no_passive() {
        let build = Build::new();
        assert_eq!(build.weapon_level(Weapon::Crowbar), 1);
        assert_eq!(build.max_hp(), BASE_HP);
        assert_eq!(build.next_level(Choice::Passive(Passive::Might)), 1);
        assert_eq!(build.next_level(Choice::Weapon(Weapon::Crowbar)), 2);
        assert_eq!(build.next_level(Choice::Weapon(Weapon::Pistol)), 1);
    }

    #[test]
    fn a_level_up_offers_three_different_choices() {
        let build = Build::new();
        let mut rng = rng();
        for _ in 0..50 {
            let offers = build.offers(&mut rng, false, RARE_PERCENT, 5, |_| true);
            assert_eq!(offers.len(), 3);
            assert!(offers[0] != offers[1] && offers[1] != offers[2] && offers[0] != offers[2]);
        }
    }

    #[test]
    fn nothing_at_the_top_level_is_offered() {
        let mut build = Build::new();
        for _ in 0..10 {
            build.take(Choice::Weapon(Weapon::Crowbar));
        }
        assert_eq!(build.weapon_level(Weapon::Crowbar), MAX_LEVEL);
        let mut rng = rng();
        for _ in 0..50 {
            assert!(
                !build
                    .offers(&mut rng, false, RARE_PERCENT, 5, |_| true)
                    .contains(&Choice::Weapon(Weapon::Crowbar))
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
                    .offers(&mut rng, false, RARE_PERCENT, 5, |_| true)
                    .contains(&Choice::Passive(Passive::Haste))
            );
        }
    }

    #[test]
    fn one_action_card_a_run_its_levels_alone_dealt_once_taken() {
        let mut build = Build::new();
        let actions = |build: &Build| {
            (0..200)
                .flat_map(|seed| {
                    build.offers(
                        &mut StdRng::seed_from_u64(seed),
                        false,
                        RARE_PERCENT,
                        0,
                        |_| true,
                    )
                })
                .filter(|choice| matches!(choice, Choice::Action(_)))
                .collect::<Vec<_>>()
        };
        assert!(actions(&build).contains(&Choice::Action(Action::Dash)));
        build.take(Choice::Action(Action::Dash));
        assert!(
            actions(&build)
                .iter()
                .all(|choice| *choice == Choice::Action(Action::Dash))
        );
        assert_eq!(build.action_level(Action::Dash), 1);
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
            build.take(Choice::Action(Action::Dash));
        }
        assert_eq!(
            build.offers(&mut rng(), false, RARE_PERCENT, 5, |_| true),
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
                    .offers(&mut rng, true, RARE_PERCENT, 5, |_| true)
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
                    .offers(&mut rng, false, RARE_PERCENT, 5, |_| true)
                    .contains(&legendary)
            })
            .count();
        assert!((20..=80).contains(&found), "{found} in 1000");
        build.take(legendary);
        assert!(build.has(Legendary::Illumination));
        assert!((0..500).all(|_| {
            !build
                .offers(&mut rng, false, RARE_PERCENT, 5, |_| true)
                .contains(&legendary)
        }));
    }

    #[test]
    fn the_dash_card_brings_a_dash_slow_to_come_back_then_two_at_most() {
        let mut build = Build::new();
        assert_eq!(build.dashes(), 0);
        let mut levels = Vec::new();
        for _ in 0..5 {
            build.take(Choice::Action(Action::Dash));
            levels.push((build.dashes(), build.dash_recharge()));
        }
        assert_eq!(levels, [(1, 240), (1, 180), (2, 180), (2, 135), (2, 90)]);
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
            .flat_map(|_| build.offers(&mut rng, false, RARE_PERCENT, 0, |_| true))
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
