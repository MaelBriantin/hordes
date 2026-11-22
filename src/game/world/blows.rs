//! The demolition hammer at work: its blows on the ground ahead of the player, their shock, and
//! the shocks following on.

use super::super::Vec2;
use super::super::build::Weapon;
use super::super::enemy::EnemyKind;
use super::super::hammer::{
    self, AHEAD, BLOW_TICKS, Blow, KNOCKBACK, RAISE_TICKS, SHOCK_RADIUS, WAVE_TICKS,
};
use super::World;
use super::weapons::against;

impl World {
    /// The hammer: its last blow fades; once it can, and an enemy stands where it would land, it
    /// is raised over the player's head a moment, then falls just ahead of them, the way they face
    /// then, its shock hurting every enemy around, throwing them back, stunning them at the last
    /// level. In the dark too: it strikes what is near.
    pub(super) fn swing_hammer(&mut self) {
        if let Some(blow) = &mut self.blow {
            blow.age += 1;
            if blow.age >= BLOW_TICKS {
                self.blow = None;
            }
        }
        for wave in &mut self.quake_waves {
            wave.age += 1;
        }
        self.quake_waves.retain(|wave| wave.age < BLOW_TICKS);
        let level = self.player.build.weapon_level(Weapon::Hammer);
        if level == 0 {
            self.quakes.clear();
            return;
        }
        let stats = hammer::at(level);
        self.shake(stats);
        let at = self.player.position + self.player.facing.way() * AHEAD;
        let reaches = |position: Vec2, half: f32| (position - at).length() <= stats.radius + half;
        // Raised, it falls once the moment is over.
        if self.hammer_raised > 0 {
            self.hammer_raised -= 1;
            if self.hammer_raised > 0 {
                return;
            }
        } else {
            self.hammer_wait = self.hammer_wait.saturating_sub(1);
            let near = self
                .enemies
                .iter()
                .any(|enemy| reaches(enemy.position, enemy.kind.stats().half_width));
            if self.hammer_wait == 0 && near {
                self.hammer_raised = RAISE_TICKS;
            }
            return;
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        self.blow_hit.clear();
        for enemy in &mut self.enemies {
            let enemy_stats = enemy.kind.stats();
            if !reaches(enemy.position, enemy_stats.half_width) {
                continue;
            }
            self.blow_hit.push(enemy.id);
            let standing = enemy.hp > 0;
            enemy.hit(against(damage, enemy, hunter));
            enemy.stunned = enemy.stunned.max(stats.stun);
            // Thrown out over the wave's ticks, as it goes.
            let ticks = f32::from(u16::try_from(WAVE_TICKS).unwrap_or(1));
            enemy.knock = (enemy.position - at).normalized() * (KNOCKBACK / ticks);
            enemy.knocked = WAVE_TICKS;
            if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                self.abominations_smashed += 1;
            }
        }
        self.blow = Some(Blow {
            at,
            radius: stats.radius,
            age: 0,
        });
        let way = self.player.facing.way();
        self.quakes = hammer::quakes(at, way, stats.shocks, stats.sideways);
        self.hammer_wait = self.player.build.cooldown(stats.every);
    }

    /// The shocks following the last blow, each once its time comes: every enemy around it not
    /// yet hit by the blow is, thrown back, stunned at the last level.
    fn shake(&mut self, stats: hammer::Hammer) {
        for quake in &mut self.quakes {
            quake.left = quake.left.saturating_sub(1);
        }
        let build = &self.player.build;
        let (damage, hunter) = (build.damage(stats.damage), build.horde_bonus());
        let ticks = f32::from(u16::try_from(WAVE_TICKS).unwrap_or(1));
        for quake in self.quakes.iter().filter(|quake| quake.left == 0) {
            for enemy in &mut self.enemies {
                let apart = enemy.position - quake.at;
                if apart.length() > SHOCK_RADIUS + enemy.kind.stats().half_width
                    || self.blow_hit.contains(&enemy.id)
                {
                    continue;
                }
                self.blow_hit.push(enemy.id);
                let standing = enemy.hp > 0;
                enemy.hit(against(damage, enemy, hunter));
                enemy.stunned = enemy.stunned.max(stats.stun);
                enemy.knock = apart.normalized() * (KNOCKBACK / ticks);
                enemy.knocked = WAVE_TICKS;
                if standing && enemy.hp == 0 && enemy.kind == EnemyKind::Abomination {
                    self.abominations_smashed += 1;
                }
            }
            self.quake_waves.push(Blow {
                at: quake.at,
                radius: SHOCK_RADIUS,
                age: 0,
            });
        }
        self.quakes.retain(|quake| quake.left > 0);
    }
}
