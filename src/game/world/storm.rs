//! Lightning: strikes foretold, then falling, hurting and setting on fire what they fall among.

use rand::RngExt;

use super::super::Vec2;
use super::super::mode::Mode;
use super::super::progress::Achievement;
use super::super::weather;
use super::{Lightning, World};

impl World {
    /// Lightning strikes now and then, as often as the weather has it, somewhere the player
    /// sees; from the storm on, it hurts the enemies it falls among, and the player: its ring
    /// shows where a second before it falls, often where the player stands.
    pub(super) fn storm(&mut self) {
        let weather = self.weather();
        let due = self.ticks.saturating_sub(weather::STRIKE_WARNING_TICKS);
        let (falling, coming): (Vec<_>, Vec<_>) = self
            .strikes_coming
            .iter()
            .partition(|strike| strike.tick <= due);
        self.strikes_coming = coming;
        for strike in falling {
            self.strike(strike.at, true);
        }
        let rising = self.mode == Mode::Standard && self.ticks >= weather::sunrise();
        let Some(every) = weather.lightning_every().filter(|_| !rising) else {
            self.next_strike_in = None;
            return;
        };
        let wait = self.next_strike_in.get_or_insert(every);
        if *wait > 0 {
            *wait -= 1;
            return;
        }
        // Not like clockwork: between half and one and a half times the weather's wait.
        self.next_strike_in = Some(self.rng.random_range(every / 2..=every * 3 / 2));
        let (view, centre) = (self.view, self.player.position);
        let aimed = self.rng.random_range(0..100) < weather.aimed_strike_percent();
        let at = if aimed {
            centre
        } else {
            centre
                + Vec2::new(
                    self.rng.random_range(-view.x..=view.x),
                    self.rng.random_range(-view.y..=view.y),
                )
        };
        if weather.strikes_hurt() {
            self.strikes_coming.push(Lightning {
                tick: self.ticks,
                at,
            });
        } else {
            self.strike(at, false);
        }
    }

    /// A strike of lightning falls at `at`, hurting the enemies around it and setting them on
    /// fire, if `hurts`; never the player, whose torch it puts out if they are in its ring.
    pub(super) fn strike(&mut self, at: Vec2, hurts: bool) {
        self.lightning = Some(Lightning {
            tick: self.ticks,
            at,
        });
        // The whole map lit: every enemy on screen sees the player.
        self.give_away(self.view.x.max(self.view.y));
        // In its ring, the player's torch goes out.
        if (self.player.position - at).length() <= weather::STRIKE_RADIUS {
            self.cut_torch();
        }
        if !hurts {
            return;
        }
        let burns = self.player.build.burn_ticks();
        for enemy in &mut self.enemies {
            if (enemy.position - at).length() <= weather::STRIKE_RADIUS {
                enemy.hit(weather::STRIKE_DAMAGE);
                enemy.ignite_for(burns);
            }
        }
        // It never hurts the player: struck, they have the achievement.
        if (self.player.position - at).length() <= weather::STRIKE_RADIUS
            && !self.daybreak()
            && self.player.dash.is_none()
        {
            self.progress.reach(Achievement::Struck);
        }
    }

    /// The enemies within `reach` of the player find them: a strike of lightning gives their
    /// place away.
    pub(super) fn give_away(&mut self, reach: f32) {
        let player = self.player.position;
        for enemy in &mut self.enemies {
            if (enemy.position - player).length() <= reach {
                enemy.noticed = true;
            }
        }
    }
}
