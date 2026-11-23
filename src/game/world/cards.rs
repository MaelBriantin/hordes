//! The cards: those dealt on a level-up, the one chosen, a reroll.

use super::{
    Choice, HEAL, HORDE_RARE_PERCENT, LEGENDARY_PERCENT, Passive, RARE_PERCENT, Skin, VITALITY_HP,
    Weather, World,
};

impl World {
    /// Picks the choice at `index` of the offer: then the next level's offer, or the world
    /// goes on.
    pub fn choose(&mut self, index: usize) {
        let Some(choice) = self
            .offer
            .as_ref()
            .and_then(|offer| offer.get(index))
            .copied()
        else {
            return;
        };
        let heal = HEAL * self.player.build.hero().perk().heal_percent() / 100;
        let player = &mut self.player;
        match choice {
            Choice::Heal => {
                self.heal(heal);
            }
            Choice::Passive(Passive::Vitality) => {
                player.build.take(choice);
                player.hp += VITALITY_HP;
            }
            _ => player.build.take(choice),
        }
        self.pending_levels -= 1;
        self.offer = (self.pending_levels > 0).then(|| self.deal());
    }

    /// The cards on offer dealt again, as rare as they were, if the run has a reroll left.
    pub fn reroll(&mut self) {
        if self.offer.is_none() || self.rerolls == 0 {
            return;
        }
        self.rerolls -= 1;
        self.offer = Some(self.deal_as(self.offer_rare));
    }

    /// The cards of a level-up: rarer for a level a horde brought.
    pub(super) fn deal(&mut self) -> Vec<Choice> {
        let rare = if self.rare_levels > 0 {
            self.rare_levels -= 1;
            HORDE_RARE_PERCENT
        } else {
            RARE_PERCENT
        };
        self.offer_rare = rare;
        self.deal_as(rare)
    }

    /// Cards, each a rare one `rare` percent of the time. The longer without a legendary, the
    /// likelier one: a point of percent more for each level-up without, from the first chance
    /// on.
    pub(super) fn deal_as(&mut self, rare: u32) -> Vec<Choice> {
        let dusk = self.weather() == Weather::Dusk;
        let percent = LEGENDARY_PERCENT + self.without_legendary;
        let (progress, start) = (&self.dealt, self.start);
        // A hero's own action, once it has come, at level 3: its next levels to them alone; no
        // other action is dealt.
        let build = &self.player.build;
        let hero_action = build.hero().action();
        let own_level = build.action_level(hero_action);
        let offer = self
            .player
            .build
            .offers(&mut self.rng, dusk, rare, percent, |choice| match choice {
                // A hero's own weapon, to them alone.
                Choice::Weapon(weapon) if Skin::owning(weapon).is_some() => weapon == start,
                Choice::Action(action) => action == hero_action && own_level > 0,
                _ => progress.deals(choice),
            });
        if offer
            .iter()
            .any(|choice| matches!(choice, Choice::Legendary(_)))
        {
            self.without_legendary = 0;
        } else {
            self.without_legendary += 1;
        }
        offer
    }
}
