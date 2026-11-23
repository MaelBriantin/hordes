//! The player as a run goes: moving and dashing, hurt and healed, their XP.

use super::super::build::Choice;
use super::super::gatling;
use super::{
    ACTION_LEVEL, CAMPFIRE_LIGHT, Dash, Feature, HERB_HP, HERB_REACH, Input, PLAYER_HALF_WIDTH,
    PLAYER_SPEED, RngExt, Stain, TICKS_PER_SECOND, Vec2, World, heal, slide, tick_seconds, touch,
};

impl World {
    /// Regeneration gives HP back of itself, every second, tenths kept until they make a whole
    /// one.
    pub(super) fn regenerate(&mut self) {
        if !self.ticks.is_multiple_of(u32::from(TICKS_PER_SECOND)) {
            return;
        }
        let build = &self.player.build;
        self.hp_tenths += build.regeneration_tenths();
        let whole = self.hp_tenths / 10;
        self.hp_tenths %= 10;
        self.heal(whole);
    }

    /// XP for the player, in hundredths of a point: the whole points now, the rest kept for the
    /// next.
    pub(super) fn give_xp(&mut self, hundredths: u32) {
        self.xp_hundredths += hundredths;
        let whole = self.xp_hundredths / 100;
        self.xp_hundredths %= 100;
        self.pending_levels += self.player.gain(whole);
    }

    pub(super) fn move_player(&mut self, input: Input) {
        // Braced before a charge, the player stands still.
        if self.ram_windup.is_some() {
            return;
        }
        let player = &mut self.player;
        let (most, recharge) = (player.build.dashes(), player.build.dash_recharge());
        player.dashes.tick(most, recharge);
        // Dashing, the player goes on the way they dash, whatever the keys say.
        if let Some(dash) = &mut player.dash {
            let step = dash.step();
            dash.left -= 1;
            if dash.left == 0 {
                player.dash = None;
            }
            player.position = slide(&self.map, player.position, step, PLAYER_HALF_WIDTH, false);
            return;
        }
        player.facing = player.facing.after(input.direction);
        // Normalized, so a diagonal is no faster than a straight line.
        let player = &mut self.player;
        let mut speed = PLAYER_SPEED * f32::from(player.build.speed_percent()) / 100.0;
        // Firing the gatling, at half their pace.
        if self.gatling_left > 0 {
            speed *= f32::from(u16::try_from(gatling::PACE_PERCENT).unwrap_or(100)) / 100.0;
        }
        let step = input.direction.normalized() * (speed * tick_seconds());
        player.position = slide(&self.map, player.position, step, PLAYER_HALF_WIDTH, false);
    }

    /// The player dashes the way they face, if they have the Dash card and a dash is ready:
    /// nothing hurts them while they do.
    pub fn dash(&mut self) {
        let player = &mut self.player;
        if self.announcement.is_some()
            || player.dash.is_some()
            || player.build.dashes() == 0
            || !player.dashes.take()
        {
            return;
        }
        player.dash = Some(Dash::new(player.facing.way()));
    }

    /// A hurt player picks up the herbs within their reach, and is healed by a burning campfire
    /// they stay by, which gives of itself as it does.
    pub(super) fn heal_player(&mut self) {
        let player = self.player.position;
        let (x, y) = player.tile();
        // As far as a campfire's light shows at its brightest, fading included: it heals within.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a campfire's light, a few tiles"
        )]
        let around = (2.0 * CAMPFIRE_LIGHT).ceil() as i32 + 1;
        let mut fire = None;
        for (tile, feature) in self.features_in((x - around, y - around), (x + around, y + around))
        {
            let middle = Vec2::middle_of(tile);
            let apart = (middle - player).length();
            let hurt = self.player.hp < self.player.max_hp();
            match feature {
                Feature::Herb if hurt && apart <= HERB_REACH => {
                    let herb = HERB_HP * self.player.build.hero().perk().heal_percent() / 100;
                    self.heal(herb);
                    self.picked.insert(tile);
                    self.healed_at = Some(self.ticks);
                }
                Feature::Campfire
                    if hurt
                        && self.campfire_left(tile) > 0
                        && apart <= heal::reach(self.campfire_left(tile)) =>
                {
                    fire = Some(tile);
                }
                _ => {}
            }
        }
        let Some(tile) = fire else {
            self.by_the_fire = 0;
            return;
        };
        self.by_the_fire += 1;
        let percent = self.player.build.hero().perk().heal_percent();
        if self.by_the_fire >= heal::ticks_per_hp() * 100 / percent {
            self.by_the_fire = 0;
            self.heal(1);
            *self.burnt.entry(tile).or_insert(0) += 1;
            self.healed_at = Some(self.ticks);
        }
    }

    /// The player loses the HP of the hardest-hitting enemy touching them, then nothing hurts
    /// them for half a second.
    pub(super) fn hurt_player(&mut self) {
        if self.unhurt || self.daybreak() || self.player.dash.is_some() {
            return;
        }
        let player = &mut self.player;
        if player.invincible > 0 {
            player.invincible -= 1;
            return;
        }
        let hit = self
            .enemies
            .iter()
            .filter(|enemy| {
                touch(
                    enemy.position,
                    enemy.kind.stats().half_width,
                    player.position,
                    PLAYER_HALF_WIDTH,
                )
            })
            .map(|enemy| (enemy.kind.stats().damage, enemy.position))
            .max_by_key(|(damage, _)| *damage);
        if let Some((damage, at)) = hit {
            // A shield takes a share of a blow from ahead.
            let ahead = {
                let (way, to) = (player.facing.way(), at - player.position);
                way.x * to.x + way.y * to.y > 0.0
            };
            let percent = if ahead {
                player.build.hero().perk().front_hurt_percent()
            } else {
                100
            };
            self.wound_player(super::super::percent_of(damage, percent));
        }
    }

    /// The player gets back `amount` HP, never over their most: how many they did, counted for
    /// First aid.
    pub(super) fn heal(&mut self, amount: u32) -> u32 {
        let player = &mut self.player;
        let healed = amount.min(player.max_hp().saturating_sub(player.hp));
        player.hp += healed;
        self.healed += healed;
        healed
    }

    /// The player loses `damage` HP, bleeds, and can't be hurt again for half a second; nothing
    /// while their chainsaw grinds: they charge into the crowd.
    pub(super) fn wound_player(&mut self, damage: u32) {
        if self.sawing {
            return;
        }
        // The ninja dodges a blow now and then: nothing taken, but as long unhurt after.
        let dodge = self.player.build.hero().perk().dodge_percent();
        if dodge > 0 && self.rng.random_range(0..100) < dodge {
            self.player.invincible = u32::from(TICKS_PER_SECOND) / 2;
            self.dodged_at = Some(self.ticks);
            return;
        }
        let player = &mut self.player;
        let damage = super::super::percent_of(damage, player.build.hero().perk().hurt_percent());
        player.hp = player.hp.saturating_sub(damage);
        player.invincible = u32::from(TICKS_PER_SECOND) / 2;
        self.hurt_at = Some(self.ticks);
        self.stains.push(Stain {
            position: player.position,
            tick: self.ticks,
            size: 1,
            ash: false,
        });
    }

    /// At the run's level 3, the hero's own action, if they have one, at its first level, its
    /// card shown to the player.
    pub(super) fn unlock_action(&mut self) {
        let build = &mut self.player.build;
        if self.player.level < ACTION_LEVEL {
            return;
        }
        if let Some(action) = build
            .hero()
            .action()
            .filter(|action| build.action_level(*action) == 0)
        {
            build.take(Choice::Action(action));
            self.unlocked = Some(action);
        }
    }

    /// The player has seen their action's card: the run goes on.
    pub fn seen_action(&mut self) {
        self.unlocked = None;
    }
}
