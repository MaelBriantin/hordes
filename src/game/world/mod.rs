use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::body::{fits, slide, touch};
use super::build::{BASE_HP, Build, Choice, HEAL, Passive, VITALITY_HP, Weapon};
use super::drones;
use super::enemy::{Enemy, EnemyKind};
use super::gem::Gem;
use super::guns::{self, Bullet, Magazine};
use super::heal::{self, CAMPFIRE_HP, CAMPFIRE_REACH, HERB_HP, HERB_REACH};
use super::level::xp_to_next;
use super::machete::{self, Swing};
use super::map::Feature;
use super::score;
use super::spawn::{self, MARGIN, MAX_ENEMIES};
use super::stain::{self, MAX_STAINS, Stain};
use super::weather::{self, Weather};
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

/// The last strike of lightning: where it fell, and when.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lightning {
    pub tick: u32,
    pub at: Vec2,
}

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
    /// The number the next enemy to appear gets.
    next_enemy_id: u32,
    pub bullets: Vec<Bullet>,
    pub gems: Vec<Gem>,
    /// Where blood was spilt lately, oldest first.
    pub stains: Vec<Stain>,
    /// The enemies the player has felled.
    pub kills: u32,
    pub score: u64,
    /// The kills in a row, each within the combo's window of the one before; 0 once it closes.
    pub combo: u32,
    /// The tick of the last kill.
    last_kill: u32,
    /// Ticks left before the machete can swing again.
    machete_wait: u32,
    /// The slashes of the current swing done, and still to come, and the ticks before the next.
    slashes_done: u32,
    slashes_left: u32,
    slash_wait: u32,
    /// The machete's last swing, while it shows.
    pub swing: Option<Swing>,
    /// Each gun's magazine, once the player has the gun.
    pub magazines: HashMap<Weapon, Magazine>,
    /// Where the first drone is on its circle, in radians.
    drone_angle: f32,
    /// The last strike of lightning, if any yet.
    pub lightning: Option<Lightning>,
    /// The herbs picked, by tile.
    picked: HashSet<(i32, i32)>,
    /// The HP each campfire has given, by tile.
    burnt: HashMap<(i32, i32), u32>,
    /// Ticks spent by a campfire towards its next HP.
    by_the_fire: u32,
    /// Ticks before the next strike, once the weather brings lightning.
    next_strike_in: Option<u32>,
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
            next_enemy_id: 0,
            bullets: Vec::new(),
            gems: Vec::new(),
            stains: Vec::new(),
            kills: 0,
            score: 0,
            combo: 0,
            last_kill: 0,
            machete_wait: 0,
            slashes_done: 0,
            slashes_left: 0,
            slash_wait: 0,
            swing: None,
            magazines: HashMap::new(),
            drone_angle: 0.0,
            lightning: None,
            picked: HashSet::new(),
            burnt: HashMap::new(),
            by_the_fire: 0,
            next_strike_in: None,
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

    /// The weather now: it worsens as the run goes on.
    pub fn weather(&self) -> Weather {
        Weather::at(self.seconds())
    }

    /// How far around the player the night is lit, in tiles: their light, less what the
    /// weather takes, never under 4.
    pub fn light_radius(&self) -> f32 {
        (self.player.build.light_radius() - self.weather().light_loss()).max(4.0)
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
        self.storm();
        self.swing_machete();
        self.fire_guns();
        self.move_bullets();
        self.turn_drones();
        self.fell_enemies();
        self.gather_gems();
        self.heal_player();
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

    /// The machete swings once it can and an enemy is in reach; the slashes of a swing follow
    /// each other.
    fn swing_machete(&mut self) {
        if let Some(swing) = &mut self.swing {
            swing.left -= 1;
            if swing.left == 0 {
                self.swing = None;
            }
        }
        let level = self.player.build.weapon_level(Weapon::Machete);
        if level == 0 {
            return;
        }
        self.machete_wait = self.machete_wait.saturating_sub(1);
        if self.slashes_left > 0 {
            if self.slash_wait > 0 {
                self.slash_wait -= 1;
            } else {
                self.slash(level);
                self.slashes_left -= 1;
                self.slash_wait = machete::SLASH_GAP - 1;
            }
            return;
        }
        if self.machete_wait > 0 {
            return;
        }
        let player = self.player.position;
        let reach = machete::reach(level);
        let near = self.enemies.iter().any(|enemy| {
            machete::in_reach(
                enemy.position - player,
                enemy.kind.stats().half_width,
                reach,
            )
        });
        if !near {
            return;
        }
        self.slashes_done = 0;
        self.slash(level);
        self.slashes_left = machete::slashes(level) - 1;
        self.slash_wait = machete::SLASH_GAP - 1;
        self.machete_wait = self.player.build.cooldown(machete::cooldown());
    }

    /// A slash of the machete: it hits every enemy in its arc, the way the player faces for the
    /// swing's first and third slashes, behind them for its second.
    fn slash(&mut self, level: u8) {
        let player = self.player.position;
        let facing = if self.slashes_done % 2 == 1 {
            self.player.facing.opposite()
        } else {
            self.player.facing
        };
        self.slashes_done += 1;
        let swing = Swing {
            facing,
            left: machete::SWING_TICKS,
            reach: machete::reach(level),
            half_arc: machete::half_arc(level),
        };
        let ahead = swing.facing.way();
        let damage = self.player.build.damage(machete::DAMAGE);
        for enemy in &mut self.enemies {
            let offset = enemy.position - player;
            let half_width = enemy.kind.stats().half_width;
            let hit = machete::in_arc(offset, half_width, swing.reach, ahead, swing.half_arc);
            if hit {
                enemy.hit(damage);
            }
        }
        self.swing = Some(swing);
    }

    /// Lightning strikes now and then, as often as the weather has it, somewhere the player
    /// sees; from the storm on, it hurts the enemies it falls among.
    fn storm(&mut self) {
        let weather = self.weather();
        let Some(every) = weather.lightning_every() else {
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
        let at = centre
            + Vec2::new(
                self.rng.random_range(-view.x..=view.x),
                self.rng.random_range(-view.y..=view.y),
            );
        self.strike(at, weather.strikes_hurt());
    }

    /// A strike of lightning falls at `at`, hurting the enemies around it if `hurts`.
    fn strike(&mut self, at: Vec2, hurts: bool) {
        self.lightning = Some(Lightning {
            tick: self.ticks,
            at,
        });
        if !hurts {
            return;
        }
        for enemy in &mut self.enemies {
            if (enemy.position - at).length() <= weather::STRIKE_RADIUS {
                enemy.hit(weather::STRIKE_DAMAGE);
            }
        }
    }

    /// Each gun the player has shoots at the nearest enemies in its range, while its magazine
    /// holds rounds; empty, it reloads.
    fn fire_guns(&mut self) {
        let Self {
            player,
            enemies,
            bullets,
            magazines,
            ..
        } = self;
        let build = &player.build;
        for weapon in [Weapon::Pistol, Weapon::MachineGun] {
            let level = build.weapon_level(weapon);
            let Some(gun) = guns::gun(weapon, level).filter(|_| level > 0) else {
                continue;
            };
            let magazine = magazines.entry(weapon).or_insert(Magazine {
                rounds: gun.magazine,
                ..Magazine::default()
            });
            if magazine.reloading > 0 {
                magazine.reloading -= 1;
                if magazine.reloading == 0 {
                    magazine.rounds = gun.magazine;
                }
                continue;
            }
            if magazine.wait > 0 {
                magazine.wait -= 1;
                continue;
            }
            let targets = guns::targets(enemies, player.position, gun.bullets, gun.range);
            if targets.is_empty() {
                continue;
            }
            let damage = build.damage(gun.damage);
            for target in targets {
                bullets.push(Bullet::toward(player.position, target, &gun, damage));
            }
            magazine.rounds = magazine.rounds.saturating_sub(1);
            magazine.wait = gun.interval;
            if magazine.rounds == 0 {
                // Reloaded, the gun shoots at once: the reload is its wait.
                magazine.wait = 0;
                magazine.reloading = build.cooldown(gun.reload).max(1);
            }
        }
    }

    /// Bullets fly on, hitting each enemy they touch once; gone once they have gone through as
    /// many as they can, or flown their range.
    fn move_bullets(&mut self) {
        let enemies = &mut self.enemies;
        self.bullets.retain_mut(|bullet| {
            if !bullet.advance() {
                return false;
            }
            for enemy in enemies.iter_mut() {
                let touched = touch(
                    bullet.position,
                    guns::BULLET_HALF_WIDTH,
                    enemy.position,
                    enemy.kind.stats().half_width,
                );
                if enemy.hp == 0 || !touched || bullet.hit.contains(&enemy.id) {
                    continue;
                }
                enemy.hit(bullet.damage);
                bullet.hit.push(enemy.id);
                if bullet.pierce == 0 {
                    return false;
                }
                bullet.pierce -= 1;
            }
            true
        });
    }

    /// The guns the player has, each with its magazine and how many rounds it holds full.
    pub fn ammo(&self) -> Vec<(Weapon, Magazine, u32)> {
        [Weapon::Pistol, Weapon::MachineGun]
            .into_iter()
            .filter_map(|weapon| {
                let level = self.player.build.weapon_level(weapon);
                let gun = guns::gun(weapon, level).filter(|_| level > 0)?;
                let magazine = self.magazines.get(&weapon).copied().unwrap_or(Magazine {
                    rounds: gun.magazine,
                    ..Magazine::default()
                });
                Some((weapon, magazine, gun.magazine))
            })
            .collect()
    }

    /// Where the drones are, if the player has them.
    pub fn drones(&self) -> Vec<Vec2> {
        let level = self.player.build.weapon_level(Weapon::Drones);
        drones::positions(self.player.position, level, self.drone_angle)
    }

    /// The drones turn on, each hitting the enemies it touches that can be hit, and pushing
    /// them back from the player.
    fn turn_drones(&mut self) {
        for enemy in &mut self.enemies {
            enemy.drone_ready_in = enemy.drone_ready_in.saturating_sub(1);
        }
        let build = &self.player.build;
        let level = build.weapon_level(Weapon::Drones);
        if level == 0 {
            return;
        }
        self.drone_angle =
            (self.drone_angle + drones::turn_per_tick(level)) % std::f32::consts::TAU;
        let damage = build.damage(drones::damage(level));
        let rehit = build.cooldown(drones::rehit());
        let player = self.player.position;
        for position in self.drones() {
            for enemy in &mut self.enemies {
                let stats = enemy.kind.stats();
                if enemy.hp == 0
                    || enemy.drone_ready_in > 0
                    || !touch(
                        position,
                        drones::HALF_WIDTH,
                        enemy.position,
                        stats.half_width,
                    )
                {
                    continue;
                }
                enemy.hit(damage);
                enemy.drone_ready_in = rehit;
                let away = (enemy.position - player).normalized() * drones::KNOCKBACK;
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

    /// The score's multiplier now, in tenths: the weather's times the combo's.
    pub fn multiplier_tenths(&self) -> u64 {
        score::weather_tenths(self.weather()) * score::combo_tenths(self.combo) / 10
    }

    /// Enemies out of HP fall, each worth its points, some leaving a gem. The combo closes when
    /// no enemy has fallen for a while.
    fn fell_enemies(&mut self) {
        if self.ticks - self.last_kill > score::combo_window() {
            self.combo = 0;
        }
        let weather = self.weather();
        for enemy in self.enemies.iter().filter(|enemy| enemy.hp == 0) {
            self.score += score::worth(enemy.kind, weather, self.combo);
            self.combo += 1;
            self.last_kill = self.ticks;
            let stats = enemy.kind.stats();
            if self.rng.random_range(0..100) < stats.drop_percent {
                self.gems.push(Gem::new(enemy.position, stats.xp));
            }
            self.stains.push(Stain {
                position: enemy.position,
                tick: self.ticks,
                size: stats.stain,
            });
            self.kills += 1;
        }
        let lasts = stain::lasts();
        let now = self.ticks;
        self.stains.retain(|stain| now - stain.tick < lasts);
        if self.stains.len() > MAX_STAINS {
            self.stains.drain(..self.stains.len() - MAX_STAINS);
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
                let mut enemy = Enemy::new(self.next_enemy_id, kind, position, phase);
                enemy.hp = enemy.hp * self.weather().hp_percent() / 100;
                self.enemies.push(enemy);
                self.next_enemy_id += 1;
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
        let weather = self.weather();
        for (enemy, push) in self.enemies.iter_mut().zip(pushes) {
            enemy.flash = enemy.flash.saturating_sub(1);
            let stats = enemy.kind.stats();
            let heading = if touch(enemy.position, stats.half_width, target, PLAYER_HALF_WIDTH) {
                Vec2::ZERO
            } else {
                enemy.heading(target)
            };
            enemy.facing = enemy.facing.after(heading);
            let speed = stats.speed * f32::from(weather.speed_percent()) / 100.0;
            let step = (heading * (speed * tick_seconds()) + push).clamp_length(MAX_STEP);
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

    /// What lies on the ground between two tiles, by tile, as it is now: the herbs not yet
    /// picked, and every campfire, burning or out.
    pub fn features_in(&self, from: (i32, i32), to: (i32, i32)) -> Vec<((i32, i32), Feature)> {
        let mut features = self.map.features_in(from, to);
        features.retain(|(tile, feature)| *feature != Feature::Herb || !self.picked.contains(tile));
        features
    }

    /// Where the healing nearest to the player is, if they carry a compass and any lies within
    /// its range: a herb not yet picked, or a campfire still burning.
    pub fn compass(&self) -> Option<Vec2> {
        let range = self.player.build.compass_range()?;
        let player = self.player.position;
        let (x, y) = player.tile();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a range of a few hundred tiles"
        )]
        let reach = range.ceil() as i32;
        self.features_in((x - reach, y - reach), (x + reach, y + reach))
            .into_iter()
            .filter(|(tile, feature)| *feature == Feature::Herb || self.campfire_left(*tile) > 0)
            .map(|(tile, _)| Vec2::middle_of(tile))
            .filter(|healing| (*healing - player).length() <= range)
            .min_by(|a, b| (*a - player).length().total_cmp(&(*b - player).length()))
    }

    /// The HP a campfire still has to give: none once it is out.
    pub fn campfire_left(&self, tile: (i32, i32)) -> u32 {
        CAMPFIRE_HP - self.burnt.get(&tile).copied().unwrap_or(0).min(CAMPFIRE_HP)
    }

    /// A hurt player picks up the herbs they walk over, and is healed by a burning campfire they
    /// stay by, which gives of itself as it does.
    fn heal_player(&mut self) {
        let player = self.player.position;
        let (x, y) = player.tile();
        let around = 3;
        let mut fire = None;
        for (tile, feature) in self.features_in((x - around, y - around), (x + around, y + around))
        {
            let middle = Vec2::middle_of(tile);
            let apart = (middle - player).length();
            let hurt = self.player.hp < self.player.max_hp();
            match feature {
                Feature::Herb if hurt && apart <= HERB_REACH => {
                    self.player.hp = (self.player.hp + HERB_HP).min(self.player.max_hp());
                    self.picked.insert(tile);
                }
                Feature::Campfire
                    if hurt && apart <= CAMPFIRE_REACH && self.campfire_left(tile) > 0 =>
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
        if self.by_the_fire >= heal::ticks_per_hp() {
            self.by_the_fire = 0;
            self.player.hp += 1;
            *self.burnt.entry(tile).or_insert(0) += 1;
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
            self.stains.push(Stain {
                position: player.position,
                tick: self.ticks,
                size: 1,
            });
        }
    }
}
