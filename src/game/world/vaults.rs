//! The vault at work, the special agent's own action.

use super::super::Vec2;
use super::super::body::{fits, touch};
use super::super::build::Action;
use super::super::vault::{
    self, AHEAD, BLOCK, HOP_TILES, KICK_REACH, KICK_TICKS, KICK_TILES, Stats, Vault,
};
use super::{PLAYER_HALF_WIDTH, World};

/// How far apart the points looked at ahead are, in tiles.
const LOOK_STEP: f32 = 0.25;

impl World {
    /// The agent vaults, if it is ready and there is somewhere to land: over the obstacle just
    /// ahead, or a short hop; then it waits its recharge.
    pub(super) fn vault(&mut self) {
        let level = self.player.build.action_level(Action::Vault);
        if self.announcement.is_some()
            || level == 0
            || self.vault_wait > 0
            || self.player.vault.is_some()
            || self.player.dash.is_some()
        {
            return;
        }
        let stats = vault::at(level);
        let Some(to) = self.landing(stats) else {
            return;
        };
        self.player.vault = Some(Vault {
            from: self.player.position,
            to,
            age: 0,
            kicks: stats.kicks,
        });
        self.vault_wait = self.player.build.cooldown(stats.recharge).max(1);
    }

    /// Where the vault lands, if anywhere: past the obstacles (and from its level 3 the enemies)
    /// just ahead, as many blocks as it clears; nothing ahead, a short hop, short of what stands
    /// in its way.
    fn landing(&self, stats: Stats) -> Option<Vec2> {
        let (from, way) = (self.player.position, self.player.facing.way());
        let at = |distance: f32| from + way * distance;
        let walled = |point: Vec2| !fits(&self.map, point, PLAYER_HALF_WIDTH, false);
        let crowded = |point: Vec2| {
            self.enemies.iter().any(|enemy| {
                enemy.hp > 0
                    && touch(
                        point,
                        PLAYER_HALF_WIDTH,
                        enemy.position,
                        enemy.kind.stats().half_width,
                    )
            })
        };
        let cleared = |point: Vec2| walled(point) || (stats.over_enemies && crowded(point));
        let steps = |tiles: f32| {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a few steps"
            )]
            let count = (tiles / LOOK_STEP).ceil() as u32;
            (1..=count).map(|step| f32::from(u16::try_from(step).unwrap_or(0)) * LOOK_STEP)
        };
        // The first thing in the way, if near.
        let first =
            steps(HOP_TILES).find(|distance| walled(at(*distance)) || crowded(at(*distance)));
        match first {
            Some(start) if start <= AHEAD && cleared(at(start)) => {
                // Over it: as far as it clears, then the first free ground.
                let most = f32::from(stats.blocks) * BLOCK + 2.0 * PLAYER_HALF_WIDTH + LOOK_STEP;
                steps(start + most + AHEAD)
                    .filter(|distance| *distance > start)
                    .take_while(|distance| *distance - start <= most || !cleared(at(*distance)))
                    .find(|distance| !walled(at(*distance)) && !crowded(at(*distance)))
                    .map(at)
            }
            // Something it can't clear right ahead: nowhere to go.
            Some(start) if start <= LOOK_STEP => None,
            // A hop, short of what stands in the way.
            Some(start) => Some(at(start - LOOK_STEP)),
            None => Some(at(HOP_TILES)),
        }
    }

    /// A vault under way carries the agent on, nothing hurting them; landed, a kick throws the
    /// enemies near back. Whether one is under way.
    pub(super) fn carry_vault(&mut self) -> bool {
        let Some(vault) = &mut self.player.vault else {
            return false;
        };
        vault.age += 1;
        let landed = vault.age >= vault::TICKS;
        let (position, kicks) = (vault.position(), vault.kicks);
        self.player.position = position;
        if landed {
            self.player.vault = None;
            if kicks {
                let ticks = f32::from(u16::try_from(KICK_TICKS).unwrap_or(1));
                for enemy in &mut self.enemies {
                    let away = enemy.position - position;
                    if away.length() <= KICK_REACH + enemy.kind.stats().half_width {
                        enemy.knock = away.normalized() * (KICK_TILES / ticks);
                        enemy.knocked = KICK_TICKS;
                    }
                }
            }
        }
        true
    }

    /// Whether the player is in the air or dashing: nothing hurts them.
    pub fn untouchable(&self) -> bool {
        self.player.dash.is_some() || self.player.vault.is_some()
    }
}
