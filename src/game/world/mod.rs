use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::body::{fits, slide, touch};
use super::bolt::{self, Spark};
use super::build::{BASE_HP, Build, Choice, HEAL, Passive, VITALITY_HP, Weapon};
use super::enemy::{Enemy, EnemyKind};
use super::gem::Gem;
use super::level::xp_to_next;
use super::orb;
use super::spawn::{self, MARGIN, MAX_ENEMIES};
use super::{Facing, Map, TICKS_PER_SECOND, Vec2, tick_seconds};

#[cfg(test)]
mod tests;

/// The player's speed, in tiles per second.
const PLAYER_SPEED: f32 = 6.0;
/// Half the width of the player's body, in tiles: what obstacles stop, smaller than the sprite
/// so that brushing past one is forgiven.
const PLAYER_HALF_WIDTH: f32 = 0.45;
/// How hard enemies that overlap push each other apart, as a share of the overlap per tick.
const SEPARATION: f32 = 0.25;
/// The longest a body moves in a tick, in tiles: less than a tile, so that it never jumps
/// over one.
const MAX_STEP: f32 = 0.9;
/// How many tries to find a free spot for an enemy to appear on.
const SPAWN_TRIES: u32 = 10;
/// The most gems lying around at once: past it, the oldest go.
const MAX_GEMS: usize = 500;
/// What the player sees until the app tells, half of it each way, in tiles.
const DEFAULT_VIEW: Vec2 = Vec2::new(40.0, 12.0);

/// What the player asks for during a tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Input {
    /// Where to go: -1, 0 or 1 on each axis, `y` growing downwards.
    pub direction: Vec2,
}

#[derive(Debug)]
pub struct Player {
    /// The centre of the player's body, in tiles: the tile `(0, 0)` spans from 0.0 to 1.0.
    pub position: Vec2,
    pub facing: Facing,
    pub hp: u32,
    /// Ticks left during which nothing hurts the player, after a hit.
    pub invincible: u32,
    pub level: u32,
    /// The XP gathered towards the next level.
    pub xp: u32,
    pub build: Build,
}

impl Player {
    pub fn max_hp(&self) -> u32 {
        self.build.max_hp()
    }

    /// Gathers XP, going up as many levels as it makes. How many.
    fn gain(&mut self, xp: u32) -> u32 {
        self.xp += xp;
        let mut levels = 0;
        while self.xp >= xp_to_next(self.level) {
            self.xp -= xp_to_next(self.level);
            self.level += 1;
            levels += 1;
        }
        levels
    }
}

#[derive(Debug)]
pub struct World {
    pub map: Map,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub sparks: Vec<Spark>,
    pub gems: Vec<Gem>,
    /// The enemies the player has felled.
    pub kills: u32,
    /// Ticks left before Bolt can shoot again.
    bolt_cooldown: u32,
    /// Where the first orb is on its circle, in radians.
    orb_angle: f32,
    /// What the player is offered to pick, a level gained: the world waits meanwhile.
    pub offer: Option<Vec<Choice>>,
    /// Levels gained whose choice is still to come, the one on offer included.
    pending_levels: u32,
    ticks: u32,
    rng: StdRng,
    /// Enemies owed: grows with the rate they appear at, one appears per whole unit.
    spawn_debt: f32,
    /// Half the size of what the player sees, in tiles: enemies appear just beyond it.
    view: Vec2,
}

impl World {
    /// A new run, all its luck from `seed`: the map, and the enemies that appear.
    pub fn new(seed: u64) -> Self {
        let start = Vec2::new(0.5, 0.5);
        let map = Map::new(seed, start.tile());
        Self::on(map, start, StdRng::seed_from_u64(seed))
    }

    /// A run on a given map, the player at `start`.
    fn on(map: Map, start: Vec2, rng: StdRng) -> Self {
        Self {
            map,
            player: Player {
                position: start,
                facing: Facing::default(),
                hp: BASE_HP,
                invincible: 0,
                level: 1,
                xp: 0,
                build: Build::new(),
            },
            enemies: Vec::new(),
            sparks: Vec::new(),
            gems: Vec::new(),
            kills: 0,
            bolt_cooldown: 0,
            orb_angle: 0.0,
            offer: None,
            pending_levels: 0,
            ticks: 0,
            rng,
            spawn_debt: 0.0,
            view: DEFAULT_VIEW,
        }
    }

    /// Tells the world how much of it the player sees: half of it each way, in tiles.
    pub fn set_view(&mut self, half: Vec2) {
        self.view = half;
    }

    /// The ticks the run has lasted.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    /// The whole seconds the run has lasted.
    pub fn seconds(&self) -> u32 {
        self.ticks / u32::from(TICKS_PER_SECOND)
    }

    /// Whether the player has fallen: the world no longer moves on.
    pub fn is_over(&self) -> bool {
        self.player.hp == 0
    }

    /// Moves the world on by one tick; not while the run is over or a choice is on offer.
    pub fn update(&mut self, input: Input) {
        if self.is_over() || self.offer.is_some() {
            return;
        }
        self.ticks += 1;
        self.move_player(input);
        self.spawn();
        self.move_enemies();
        self.relocate_stragglers();
        self.shoot();
        self.move_sparks();
        self.turn_orbs();
        self.fell_enemies();
        self.gather_gems();
        self.hurt_player();
        if self.pending_levels > 0 && !self.is_over() {
            self.offer = Some(self.player.build.offers(&mut self.rng));
        }
    }

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
        let player = &mut self.player;
        match choice {
            Choice::Heal => player.hp = (player.hp + HEAL).min(player.max_hp()),
            Choice::Passive(Passive::Vitality) => {
                player.build.take(choice);
                player.hp += VITALITY_HP;
            }
            _ => player.build.take(choice),
        }
        self.pending_levels -= 1;
        self.offer = (self.pending_levels > 0).then(|| self.player.build.offers(&mut self.rng));
    }

    /// Bolt shoots its sparks at the nearest enemies in range, once its cooldown is over.
    fn shoot(&mut self) {
        let build = &self.player.build;
        let level = build.weapon_level(Weapon::Bolt);
        if level == 0 {
            return;
        }
        if self.bolt_cooldown > 0 {
            self.bolt_cooldown -= 1;
            return;
        }
        let from = self.player.position;
        let targets = bolt::targets(&self.enemies, from, bolt::sparks(level));
        if targets.is_empty() {
            return;
        }
        let damage = build.damage(bolt::damage(level));
        for target in targets {
            self.sparks.push(Spark::toward(from, target, damage));
        }
        self.bolt_cooldown = build.cooldown(bolt::cooldown(level));
    }

    /// Sparks fly on, each hitting the first enemy it touches, gone once it hits or has flown
    /// its range.
    fn move_sparks(&mut self) {
        let enemies = &mut self.enemies;
        self.sparks.retain_mut(|spark| {
            if !spark.advance() {
                return false;
            }
            let hit = enemies.iter_mut().find(|enemy| {
                enemy.hp > 0
                    && touch(
                        spark.position,
                        bolt::HALF_WIDTH,
                        enemy.position,
                        enemy.kind.stats().half_width,
                    )
            });
            match hit {
                Some(enemy) => {
                    enemy.hit(spark.damage);
                    false
                }
                None => true,
            }
        });
    }

    /// Where the orbs are, if the player has them.
    pub fn orbs(&self) -> Vec<Vec2> {
        let level = self.player.build.weapon_level(Weapon::Orb);
        orb::positions(self.player.position, level, self.orb_angle)
    }

    /// The orbs turn on, each hitting the enemies it touches that can be hit, and pushing them
    /// back from the player.
    fn turn_orbs(&mut self) {
        for enemy in &mut self.enemies {
            enemy.orb_ready_in = enemy.orb_ready_in.saturating_sub(1);
        }
        let build = &self.player.build;
        let level = build.weapon_level(Weapon::Orb);
        if level == 0 {
            return;
        }
        self.orb_angle = (self.orb_angle + orb::turn_per_tick(level)) % std::f32::consts::TAU;
        let damage = build.damage(orb::damage(level));
        let rehit = build.cooldown(orb::rehit());
        let player = self.player.position;
        for position in self.orbs() {
            for enemy in &mut self.enemies {
                let stats = enemy.kind.stats();
                if enemy.hp == 0
                    || enemy.orb_ready_in > 0
                    || !touch(position, orb::HALF_WIDTH, enemy.position, stats.half_width)
                {
                    continue;
                }
                enemy.hit(damage);
                enemy.orb_ready_in = rehit;
                let away = (enemy.position - player).normalized() * orb::KNOCKBACK;
                enemy.position = slide(
                    &self.map,
                    enemy.position,
                    away,
                    stats.half_width,
                    stats.flies,
                );
            }
        }
    }

    /// Enemies out of HP fall, some leaving a gem.
    fn fell_enemies(&mut self) {
        for enemy in self.enemies.iter().filter(|enemy| enemy.hp == 0) {
            let stats = enemy.kind.stats();
            if self.rng.random_range(0..100) < stats.drop_percent {
                self.gems.push(Gem::new(enemy.position, stats.xp));
            }
            self.kills += 1;
        }
        self.enemies.retain(|enemy| enemy.hp > 0);
        if self.gems.len() > MAX_GEMS {
            self.gems.drain(..self.gems.len() - MAX_GEMS);
        }
    }

    /// Gems near the player fly to them; those they reach give their XP.
    fn gather_gems(&mut self) {
        let player = &mut self.player;
        let magnet = player.build.magnet_radius();
        let mut levels = 0;
        self.gems.retain_mut(|gem| {
            let picked = gem.update(player.position, magnet);
            if picked {
                levels += player.gain(gem.value);
            }
            !picked
        });
        self.pending_levels += levels;
    }

    fn move_player(&mut self, input: Input) {
        let player = &mut self.player;
        player.facing = player.facing.after(input.direction);
        // Normalized, so a diagonal is no faster than a straight line.
        let speed = PLAYER_SPEED * f32::from(player.build.speed_percent()) / 100.0;
        let step = input.direction.normalized() * (speed * tick_seconds());
        player.position = slide(&self.map, player.position, step, PLAYER_HALF_WIDTH, false);
    }

    /// New enemies, as many as the rate owes, up to the most at once.
    fn spawn(&mut self) {
        if self.enemies.len() >= MAX_ENEMIES {
            self.spawn_debt = 0.0;
            return;
        }
        let seconds = self.seconds();
        self.spawn_debt += spawn::rate(seconds) * tick_seconds();
        while self.spawn_debt >= 1.0 {
            self.spawn_debt -= 1.0;
            if self.enemies.len() >= MAX_ENEMIES {
                break;
            }
            let kind = spawn::pick_kind(&mut self.rng, seconds);
            if let Some(position) = self.free_edge_point(kind, None) {
                let phase = spawn::phase(&mut self.rng);
                self.enemies.push(Enemy::new(kind, position, phase));
            }
        }
    }

    /// A point just beyond what the player sees where an enemy of that kind fits: on `side`,
    /// or on any. None if the tries all fall on obstacles or off the map.
    fn free_edge_point(&mut self, kind: EnemyKind, side: Option<Facing>) -> Option<Vec2> {
        let stats = kind.stats();
        (0..SPAWN_TRIES)
            .map(|_| spawn::edge_point(&mut self.rng, self.player.position, self.view, side))
            .find(|point| fits(&self.map, *point, stats.half_width, stats.flies))
    }

    /// Each enemy goes for the player, pushed apart from the others it overlaps, and stops on
    /// touching the player.
    fn move_enemies(&mut self) {
        let pushes = self.separations();
        let target = self.player.position;
        for (enemy, push) in self.enemies.iter_mut().zip(pushes) {
            enemy.flash = enemy.flash.saturating_sub(1);
            let stats = enemy.kind.stats();
            let heading = if touch(enemy.position, stats.half_width, target, PLAYER_HALF_WIDTH) {
                Vec2::ZERO
            } else {
                enemy.heading(target)
            };
            enemy.facing = enemy.facing.after(heading);
            let step = (heading * (stats.speed * tick_seconds()) + push).clamp_length(MAX_STEP);
            enemy.position = slide(
                &self.map,
                enemy.position,
                step,
                stats.half_width,
                stats.flies,
            );
        }
    }

    /// How far each enemy is pushed by those it overlaps, in the order of `enemies`.
    fn separations(&self) -> Vec<Vec2> {
        let mut pushes = vec![Vec2::ZERO; self.enemies.len()];
        for (i, a) in self.enemies.iter().enumerate() {
            for (j, b) in self.enemies.iter().enumerate().skip(i + 1) {
                let reach = a.kind.stats().half_width + b.kind.stats().half_width;
                let apart = a.position - b.position;
                let distance = apart.length();
                if distance >= reach {
                    continue;
                }
                // Two on the very same spot part sideways.
                let away = if distance > f32::EPSILON {
                    apart * (1.0 / distance)
                } else {
                    Vec2::new(1.0, 0.0)
                };
                let push = away * ((reach - distance) * SEPARATION);
                pushes[i] += push;
                pushes[j] -= push;
            }
        }
        pushes
    }

    /// Enemies left far behind appear again just beyond the side the player faces.
    fn relocate_stragglers(&mut self) {
        let (reach_x, reach_y) = (2.0 * (self.view.x + MARGIN), 2.0 * (self.view.y + MARGIN));
        let centre = self.player.position;
        let facing = self.player.facing;
        for index in 0..self.enemies.len() {
            let enemy = &self.enemies[index];
            let apart = enemy.position - centre;
            if apart.x.abs() <= reach_x && apart.y.abs() <= reach_y {
                continue;
            }
            if let Some(point) = self.free_edge_point(enemy.kind, Some(facing)) {
                self.enemies[index].position = point;
            }
        }
    }

    /// The player loses the HP of the hardest-hitting enemy touching them, then nothing hurts
    /// them for half a second.
    fn hurt_player(&mut self) {
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
            .map(|enemy| enemy.kind.stats().damage)
            .max();
        if let Some(damage) = hit {
            player.hp = player.hp.saturating_sub(damage);
            player.invincible = u32::from(TICKS_PER_SECOND) / 2;
        }
    }
}
