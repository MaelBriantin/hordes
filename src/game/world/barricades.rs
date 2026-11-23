//! The barricades the player sets down, from the Barricade card, and the enemies battering them.

use super::super::barricade::{self, Barricade};
use super::super::body::fits;
use super::super::build::Action;
use super::super::{TICKS_PER_SECOND, Vec2};
use super::{PLAYER_HALF_WIDTH, World};

impl World {
    /// The player plays their action card: a dash, a barricade set down, or a ram.
    pub fn act(&mut self) {
        let build = &self.player.build;
        if build.action_level(Action::Barricade) > 0 {
            self.put_barricade();
        } else if build.action_level(Action::Ram) > 0 {
            self.ram();
        } else if build.action_level(Action::C4) > 0 {
            self.c4();
        } else {
            self.dash();
        }
    }

    /// A barricade set down just ahead of the player, if one is ready and the ground there is
    /// free, or shown refused there a moment: the walking enemies in its way are thrown beyond
    /// it. Past the most that stand at
    /// once, the oldest is taken away.
    fn put_barricade(&mut self) {
        let level = self.player.build.action_level(Action::Barricade);
        if self.announcement.is_some() || level == 0 || self.barricade_wait > 0 {
            return;
        }
        let (position, facing) = (self.player.position, self.player.facing);
        let corner = barricade::corner_ahead(position, facing, PLAYER_HALF_WIDTH);
        let new = Barricade::new(corner, facing, level);
        if new.tiles().any(|tile| self.map.blocks(tile)) {
            self.barricade_refused = Some((new, self.ticks));
            return;
        }
        self.barricade_refused = None;
        for tile in new.tiles() {
            self.map.barricade(tile, true);
        }
        for enemy in &mut self.enemies {
            let stats = enemy.kind.stats();
            if stats.flies || !new.overlaps(enemy.position, stats.half_width) {
                continue;
            }
            let thrown = new.beyond(enemy.position, stats.half_width);
            if fits(&self.map, thrown, stats.half_width, false) {
                enemy.position = thrown;
            }
        }
        self.barricades.push(new);
        if self.barricades.len() > barricade::most(level) {
            let oldest = self.barricades[0].corner;
            self.raze(|barricade| barricade.corner != oldest);
        }
        self.barricade_wait = self
            .player
            .build
            .cooldown(barricade::recharge(level))
            .max(1);
    }

    /// Every second, each walking enemy touching a wooden barricade hits it as it would the
    /// player; one of metal, from the card's level 5, takes nothing. A barricade with no HP left
    /// breaks.
    pub(super) fn batter_barricades(&mut self) {
        self.barricade_wait = self.barricade_wait.saturating_sub(1);
        if self
            .barricade_refused
            .is_some_and(|(_, at)| self.ticks.saturating_sub(at) >= barricade::REFUSED_TICKS)
        {
            self.barricade_refused = None;
        }
        for barricade in &mut self.barricades {
            barricade.flash = barricade.flash.saturating_sub(1);
        }
        if self.barricades.is_empty() || !self.ticks.is_multiple_of(u32::from(TICKS_PER_SECOND)) {
            return;
        }
        for barricade in self
            .barricades
            .iter_mut()
            .filter(|barricade| !barricade.metal)
        {
            for enemy in &mut self.enemies {
                let stats = enemy.kind.stats();
                if stats.flies || !barricade.touches(enemy.position, stats.half_width) {
                    continue;
                }
                barricade.hp = barricade.hp.saturating_sub(stats.damage);
                barricade.flash = barricade::FLASH_TICKS;
            }
        }
        self.raze(|barricade| barricade.hp > 0);
    }

    /// The wooden barricades within `reach` of `at` are blown away: an abomination's burst.
    pub(super) fn blow_barricades(&mut self, at: Vec2, reach: f32) {
        self.raze(|barricade| barricade.metal || (barricade.centre() - at).length() > reach);
    }

    /// Keeps the barricades `stays` says, the others taken away, their ground free again.
    fn raze(&mut self, stays: impl Fn(&Barricade) -> bool) {
        let map = &mut self.map;
        self.barricades.retain(|barricade| {
            let kept = stays(barricade);
            if !kept {
                for tile in barricade.tiles() {
                    map.barricade(tile, false);
                }
            }
            kept
        });
    }
}
