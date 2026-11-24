//! The enemies: how they appear, alone or in hordes, move, burn and fall; the abominations
//! bursting into parasites.

use std::collections::HashMap;

use rand::RngExt;

use super::super::announce::News;
use super::super::body::{fits, slide, touch};
use super::super::burst::{self, Blast, Swelling};
use super::super::enemy::{BOLT_PACE, BOLT_TICKS, Enemy, EnemyKind};
use super::super::gem::Gem;
use super::super::horde::{self, Tally, Warning};
use super::super::map::Feature;
use super::super::mode::Mode;
use super::super::progress::{Achievement, SWARM_HORDES};
use super::super::smoke::SLOW_PERCENT as SMOKE_SLOW_PERCENT;
use super::super::spawn::{self, MARGIN, MAX_ENEMIES};
use super::super::stain::{self, MAX_STAINS, Stain};
use super::super::weather;
use super::super::{Facing, Map, TICKS_PER_SECOND, Vec2, score, tick_seconds};
use super::{ARCADE_RATE, HORDE_LEVEL_DELAY, PLAYER_HALF_WIDTH, World};

/// How hard enemies that overlap push each other apart, as a share of the overlap per tick.
const SEPARATION: f32 = 0.25;
/// The longest a body moves in a tick, in tiles: less than a tile, so that it never jumps
/// over one.
const MAX_STEP: f32 = 0.9;
/// How many tries to find a free spot for an enemy to appear on.
const SPAWN_TRIES: u32 = 10;
/// The most enemies at once while a horde is about: a horde comes beyond the usual most.
const MAX_ENEMIES_WITH_HORDE: usize = 600;
/// The size of a cell of the grid enemies are sorted in to push each other apart, in tiles:
/// more than the widest two bodies, so that only neighbouring cells can overlap.
const SEPARATION_CELL: f32 = 2.0;
/// How near the player, in the dark, an enemy finds them, and how far it loses them again; how
/// far a campfire lures those that haven't, and how fast they wander, against their speed, in
/// tiles.
const NOTICE_REACH: f32 = 6.0;
const LOSE_REACH: f32 = 8.0;
/// The infiltrator, their torch out: how near an enemy has to be to notice them, in tiles, any
/// further losing them at once; and the ticks of dark before the Shadows hunt them: 4 s.
const GHOST_REACH: f32 = 2.0;
const GHOST_SHADOW_TICKS: u32 = 120;
const LURE_REACH: f32 = 20.0;
const WANDER_PACE: f32 = 0.5;
/// The parasites a felled abomination lets out.
const PARASITES: u16 = 4;
/// The steps a burst pushes in, each shorter than the longest a body moves in a tick.
const PUSH_STEPS: u16 = 3;
/// The ticks an enemy flashes white before it burns at daybreak.
const BURN_FLASH: u32 = 6;
/// The most gems lying around at once: past it, the oldest go.
const MAX_GEMS: usize = 500;

impl World {
    /// Enemies out of HP fall, each worth its points, some leaving a gem, some, with Drain,
    /// giving the player back a HP. The combo closes when
    /// no enemy has fallen for a while.
    pub(super) fn fell_enemies(&mut self) {
        if self.ticks - self.last_kill > score::combo_window() {
            self.combo = 0;
        }
        let mut wiped_out = 0;
        let mut drained = 0;
        let drain = self.player.build.drain_percent();
        let weather = self.weather();
        for enemy in self.enemies.iter().filter(|enemy| enemy.hp == 0) {
            // A kamikaze blown of itself is worth nothing: only its scorch mark stays.
            if enemy.blew {
                self.stains.push(Stain {
                    position: enemy.position,
                    tick: self.ticks,
                    size: enemy.kind.stats().stain,
                    ash: true,
                });
                continue;
            }
            // Worth the combo as it stood before this kill.
            let worth = score::worth(enemy.kind, weather, self.combo);
            self.progress.kills += 1;
            if enemy.kind == EnemyKind::Shadow {
                self.progress.shadows += 1;
                self.shadows_down += 1;
            }
            self.score += worth;
            self.combo += 1;
            self.last_kill = self.ticks;
            let stats = enemy.kind.stats();
            // A horde wiped out to the last brings a level; in arcade, it doubles what its kills
            // were worth too.
            if let Some(number) = enemy.horde
                && let Some(tally) = self.tallies.get_mut(&number)
            {
                tally.points += worth;
                tally.left -= 1;
                if tally.left == 0 {
                    self.progress.reach(Achievement::Hunter);
                    self.hordes_wiped += 1;
                    if self.hordes_wiped >= SWARM_HORDES {
                        self.progress.reach(Achievement::Swarm);
                    }
                    let bonus = if self.mode == Mode::Arcade {
                        tally.points
                    } else {
                        0
                    };
                    self.score += bonus;
                    self.tallies.remove(&number);
                    // Every horde of a standard run wiped out: none standing, none to come.
                    if self.mode == Mode::Standard
                        && self.tallies.is_empty()
                        && self.horde_warning.is_none()
                        && self.next_horde >= weather::dawn()
                    {
                        self.progress.reach(Achievement::Cleaner);
                    }
                    let others = !self.tallies.is_empty();
                    self.horde_cleared = Some((self.ticks, bonus, others.then_some(number)));
                    wiped_out += 1;
                }
            }
            let drop = (stats.drop_percent + self.player.build.drop_bonus()).min(100);
            if stats.xp > 0 && self.rng.random_range(0..100) < drop {
                self.gems.push(Gem::new(enemy.position, stats.xp));
            }
            self.stains.push(Stain {
                position: enemy.position,
                tick: self.ticks,
                size: stats.stain,
                ash: enemy.burning > 0,
            });
            // With Drain, some give the player back a HP.
            if drain > 0 && self.rng.random_range(0..100) < drain {
                drained += 1;
            }
            self.kills += 1;
        }
        if drained > 0 && self.heal(drained) > 0 {
            self.healed_at = Some(self.ticks);
        }
        let lasts = stain::lasts();
        let now = self.ticks;
        self.stains.retain(|stain| now - stain.tick < lasts);
        if self.stains.len() > MAX_STAINS {
            self.stains.drain(..self.stains.len() - MAX_STAINS);
        }
        // Its level comes once the horde wiped out has been told.
        for _ in 0..wiped_out {
            self.horde_levels_due
                .push(self.ticks + HORDE_LEVEL_DELAY * u32::from(TICKS_PER_SECOND));
        }
        let swelling = self
            .enemies
            .iter()
            .filter(|enemy| enemy.hp == 0 && enemy.kind == EnemyKind::Abomination)
            .map(|enemy| Swelling {
                position: enemy.position,
                facing: enemy.facing,
                left: burst::SWELL_TICKS,
            });
        self.swellings.extend(swelling.collect::<Vec<_>>());
        self.enemies.retain(|enemy| enemy.hp > 0);
        if self.gems.len() > MAX_GEMS {
            self.gems.drain(..self.gems.len() - MAX_GEMS);
        }
    }

    /// The felled abominations swell on, and burst once they have: a disc of bile that hurts
    /// and pushes back everything within its reach, the parasites thrown out of it.
    pub(super) fn burst_abominations(&mut self) {
        for blast in &mut self.blasts {
            blast.age += 1;
        }
        self.blasts.retain(Blast::shows);
        let mut bursting = Vec::new();
        self.swellings.retain_mut(|swelling| {
            swelling.left = swelling.left.saturating_sub(1);
            if swelling.left == 0 {
                bursting.push(swelling.position);
            }
            swelling.left > 0
        });
        for at in bursting {
            self.burst(at);
        }
    }

    /// An abomination bursts at `at`.
    pub(super) fn burst(&mut self, at: Vec2) {
        self.blasts.push(Blast { centre: at, age: 0 });
        self.blow_barricades(at, burst::REACH);
        for enemy in &mut self.enemies {
            let away = enemy.position - at;
            if away.length() > burst::REACH {
                continue;
            }
            enemy.hit(burst::DAMAGE);
            let stats = enemy.kind.stats();
            enemy.position = shove(
                &self.map,
                enemy.position,
                away.normalized() * burst::PUSH,
                stats.half_width,
                stats.flies,
            );
        }
        let away = self.player.position - at;
        let dashing = self.untouchable();
        if away.length() <= burst::REACH && !dashing {
            if !self.unhurt && !self.daybreak() && self.player.invincible == 0 {
                self.wound_player(burst::DAMAGE);
            }
            let thrown = self.player.build.hero().perk().thrown();
            let push = if thrown { burst::PUSH } else { 0.0 };
            self.player.position = shove(
                &self.map,
                self.player.position,
                away.normalized() * push,
                PLAYER_HALF_WIDTH,
                false,
            );
        }
        self.let_out_parasites(at);
    }

    /// A burst abomination's parasites, thrown out around it, each its own way, as tough as the
    /// weather makes them; nearer where an obstacle stands in the way.
    pub(super) fn let_out_parasites(&mut self, at: Vec2) {
        let stats = EnemyKind::Parasite.stats();
        let first = spawn::phase(&mut self.rng);
        for index in 0..PARASITES {
            let angle = first + std::f32::consts::TAU * f32::from(index) / f32::from(PARASITES);
            let way = Vec2::new(angle.cos(), angle.sin());
            let spot = shove(
                &self.map,
                at,
                way * burst::THROW,
                stats.half_width,
                stats.flies,
            );
            let mut parasite = Enemy::new(self.next_enemy_id, EnemyKind::Parasite, spot, angle);
            parasite.hp = self.toughened(parasite.hp);
            self.enemies.push(parasite);
            self.next_enemy_id += 1;
        }
    }

    /// New enemies, as many as the rate owes, up to the most at once.
    pub(super) fn spawn(&mut self) {
        if self.daybreak() {
            self.spawn_debt = 0.0;
            return;
        }
        if self.enemies.len() >= MAX_ENEMIES {
            self.spawn_debt = 0.0;
            return;
        }
        let weather = self.weather();
        // By the weather in a standard run; by the pressure in arcade.
        let rate = match self.mode {
            Mode::Standard => spawn::rate(weather, self.weather_progress()),
            Mode::Arcade => ARCADE_RATE * self.pressure(),
        };
        self.spawn_debt += rate * tick_seconds();
        while self.spawn_debt >= 1.0 {
            self.spawn_debt -= 1.0;
            if self.enemies.len() >= MAX_ENEMIES {
                break;
            }
            let kind = spawn::pick_kind(&mut self.rng, weather);
            if let Some(position) = self.free_edge_point(kind, None) {
                let phase = spawn::phase(&mut self.rng);
                let mut enemy = Enemy::new(self.next_enemy_id, kind, position, phase);
                enemy.hp = self.toughened(enemy.hp);
                self.enemies.push(enemy);
                self.next_enemy_id += 1;
            }
        }
    }

    /// Announces the next horde a moment before it comes, then brings it: a pack out of one side,
    /// or a ring out of every side from the storm on.
    pub(super) fn hordes(&mut self) {
        if self.daybreak() {
            self.horde_warning = None;
            return;
        }
        if self.horde_warning.is_none() && self.ticks + horde::warning() >= self.next_horde {
            // Out of the other side from the enemies about: the player caught between.
            let side = if horde::rings(self.weather()) {
                None
            } else {
                Some(horde::pincer(
                    &mut self.rng,
                    self.player.position,
                    self.view,
                    self.enemies.iter().map(|enemy| enemy.position),
                ))
            };
            self.horde_warning = Some(Warning {
                arrives_at: self.next_horde,
                side,
                number: self.hordes_come,
            });
            self.tell(News::Horde {
                everywhere: side.is_none(),
            });
        }
        if let Some(warning) = self.horde_warning
            && self.ticks >= warning.arrives_at
        {
            self.bring_horde(warning);
            self.horde_warning = None;
            self.hordes_come += 1;
            // From when it came: never a horde owed from the past.
            self.next_horde = self.ticks + horde::every(self.mode);
        }
    }

    /// A horde comes: its enemies, as many as fit, just beyond what the player sees.
    pub(super) fn bring_horde(&mut self, warning: Warning) {
        let weather = self.weather();
        let room = MAX_ENEMIES_WITH_HORDE.saturating_sub(self.enemies.len());
        let count = horde::size(warning.number).min(u32::try_from(room).unwrap_or(0));
        let spots = horde::spots(
            &mut self.rng,
            self.player.position,
            self.view,
            warning.side,
            count,
        );
        let mut tally = Tally::default();
        for spot in spots {
            let kind = horde::kind(&mut self.rng, weather);
            let stats = kind.stats();
            if !fits(&self.map, spot, stats.half_width, stats.flies) {
                continue;
            }
            let phase = spawn::phase(&mut self.rng);
            let mut enemy = Enemy::new(self.next_enemy_id, kind, spot, phase);
            enemy.hp = self.toughened(enemy.hp);
            enemy.horde = Some(warning.number);
            self.enemies.push(enemy);
            self.next_enemy_id += 1;
            tally.left += 1;
        }
        tally.size = tally.left;
        if tally.left > 0 {
            self.tallies.insert(warning.number, tally);
        }
    }

    /// A point just beyond what the player sees where an enemy of that kind fits: on `side`,
    /// or on any. None if the tries all fall on obstacles or off the map.
    pub(super) fn free_edge_point(
        &mut self,
        kind: EnemyKind,
        side: Option<Facing>,
    ) -> Option<Vec2> {
        let stats = kind.stats();
        (0..SPAWN_TRIES)
            .map(|_| spawn::edge_point(&mut self.rng, self.player.position, self.view, side))
            .find(|point| fits(&self.map, *point, stats.half_width, stats.flies))
    }

    /// What the enemies can tell of the player this tick.
    fn sight(&self) -> Sight {
        // At night, whether Shadows hunt: with the torch out; lit, they shun it. The infiltrator,
        // their torch out, slips away from the others, and the Shadows are slow to come.
        let ghost = self.player.build.hero().perk().ghost() && self.torch_on() == Some(false);
        // Meanwhile the Shadows are as the others: unaware of the infiltrator, neither hunting
        // nor fleeing.
        let shadows_wait = ghost && self.dark_streak < GHOST_SHADOW_TICKS;
        Sight {
            // In their smoke, the scout is lost to all but those right on them.
            smoked: self.smoked(),
            seen: self.seen_from_afar(),
            ghost,
            // The torch just out, the infiltrator vanishes from all but those right on them.
            vanishing: ghost && self.dark_streak == 0,
            shadows_wait,
            dark_hunt: self
                .torch_on()
                .filter(|_| !shadows_wait)
                .map(|_| !self.torch_lit()),
        }
    }

    /// Each enemy goes for the player, pushed apart from the others it overlaps, and stops on
    /// touching the player.
    pub(super) fn move_enemies(&mut self) {
        let pushes = self.separations();
        let target = self.player.position;
        let weather = self.weather();
        // At daybreak, they flee the player, the risen sun at their back.
        let flee = self.daybreak();
        // In the dark, those that haven't seen the player go for the campfires, or wander.
        let sight = self.sight();
        let fires = if sight.seen {
            Vec::new()
        } else {
            self.fires_near()
        };
        let dark_hunt = sight.dark_hunt;
        // At its last level, the smoke slows those in it.
        let smoke = self.smoke.filter(|cloud| cloud.slows);
        // With Glare, those in the torch's light go slower.
        let glare = self.glare();
        // Coming into it, a kamikaze lights its fuse.
        let kamikaze_reach = self.kamikaze_reach();
        let ticks = self.ticks;
        for (enemy, push) in self.enemies.iter_mut().zip(pushes) {
            enemy.flash = enemy.flash.saturating_sub(1);
            enemy.shocked = enemy.shocked.saturating_sub(1);
            enemy.dazed = enemy.dazed.saturating_sub(1);
            let stats = enemy.kind.stats();
            let apart = (target - enemy.position).length();
            notice(enemy, apart, sight);
            let fire = nearest_fire(&fires, enemy.position);
            let hunts = enemy.noticed && !flee;
            let touching = touch(enemy.position, stats.half_width, target, PLAYER_HALF_WIDTH);
            let (heading, pace) = if enemy.stunned > 0 {
                enemy.stunned -= 1;
                (Vec2::ZERO, 0.0)
            } else if let Some(pace) =
                super::kamikazes::fuse(enemy, (apart, kamikaze_reach), hunts, touching)
            {
                (enemy.heading(target), pace)
            } else if let Some(going) = super::chargers::charge(enemy, apart, target, hunts) {
                going
            } else if flee {
                ((enemy.position - target).normalized(), 1.0)
            } else if !enemy.noticed && enemy.kind == EnemyKind::Shadow && dark_hunt == Some(false)
            {
                // Away from the light, at their speed: bolting, at first.
                let pace = if enemy.bolting > 0 {
                    enemy.bolting -= 1;
                    BOLT_PACE
                } else {
                    1.0
                };
                ((enemy.position - target).normalized(), pace)
            } else if !enemy.noticed {
                match fire {
                    Some(fire) if (fire - enemy.position).length() > 1.0 => {
                        (enemy.heading(fire), WANDER_PACE)
                    }
                    Some(_) => (Vec2::ZERO, WANDER_PACE),
                    None => {
                        // A slow turn of its own, never the same as its neighbours'.
                        let id = f32::from(u16::try_from(enemy.id % 1_000).unwrap_or(0));
                        let now = f32::from(u16::try_from(ticks % 10_000).unwrap_or(0));
                        enemy.wander += (now * 0.02 + id).sin() * 0.04;
                        (
                            Vec2::new(enemy.wander.cos(), enemy.wander.sin()),
                            WANDER_PACE,
                        )
                    }
                }
            } else if let Some(keeping) =
                super::spitters::keep_off(enemy, apart, enemy.heading(target))
            {
                keeping
            } else if touching {
                (Vec2::ZERO, 1.0)
            } else {
                (enemy.heading(target), enemy.hunt_pace())
            };
            // A spitter faces the player, backing off or standing.
            let facing = super::spitters::facing(enemy, heading, target, flee);
            enemy.facing = enemy.facing.after(facing);
            let slowed = match glare {
                Some((reach, percent)) if apart <= reach => percent,
                _ => 0,
            };
            let smoked = match smoke {
                Some(cloud) if cloud.holds(enemy.position) => SMOKE_SLOW_PERCENT,
                _ => 0,
            };
            let slowed = (slowed + enemy.sawn() + smoked).min(100);
            let glared = f32::from(u16::try_from(100 - slowed).unwrap_or(100)) / 100.0;
            let speed = stats.speed * f32::from(weather.speed_percent()) / 100.0 * pace * glared;
            // Thrown by a shock, it goes on its way out, whatever else it does.
            let knock = if enemy.knocked > 0 {
                enemy.knocked -= 1;
                enemy.knock
            } else {
                Vec2::ZERO
            };
            let step = (heading * (speed * tick_seconds()) + push + knock).clamp_length(MAX_STEP);
            enemy.position = slide(
                &self.map,
                enemy.position,
                step,
                stats.half_width,
                stats.flies,
            );
        }
    }

    /// The campfires still burning around what the player sees, as far as they lure enemies: in
    /// the dark, those that haven't found the player go for them.
    pub(super) fn fires_near(&self) -> Vec<Vec2> {
        let (x, y) = self.player.position.tile();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the view and the lure, a few dozen tiles"
        )]
        let reach = (self.view.x.max(self.view.y) + LURE_REACH).ceil() as i32;
        self.features_in((x - reach, y - reach), (x + reach, y + reach))
            .into_iter()
            .filter(|(tile, feature)| {
                *feature == Feature::Campfire && self.campfire_left(*tile) > 0
            })
            .map(|(tile, _)| Vec2::middle_of(tile))
            .collect()
    }

    /// At daybreak, the enemies burn in the sun one by one, each at a moment of its own over the
    /// daybreak, flashing white just before: they leave nothing, and are worth nothing.
    pub(super) fn burn_enemies(&mut self) {
        if !self.daybreak() {
            return;
        }
        let since = self.ticks - weather::dawn();
        let last = weather::daybreak().saturating_sub(BURN_FLASH).max(1);
        self.enemies.retain_mut(|enemy| {
            let burns_at = enemy.id.wrapping_mul(2_654_435_761) % last;
            if since + BURN_FLASH >= burns_at {
                enemy.flash = enemy.flash.max(1);
            }
            since < burns_at + BURN_FLASH
        });
    }

    /// How far each enemy is pushed by those it overlaps, in the order of `enemies`. Enemies are
    /// sorted into a grid first, so that each meets only those in its own cell and the cells
    /// around: a horde of hundreds stays cheap.
    pub(super) fn separations(&self) -> Vec<Vec2> {
        let cell_of = |position: Vec2| (position * (1.0 / SEPARATION_CELL)).tile();
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (index, enemy) in self.enemies.iter().enumerate() {
            grid.entry(cell_of(enemy.position)).or_default().push(index);
        }
        let mut pushes = vec![Vec2::ZERO; self.enemies.len()];
        for (i, a) in self.enemies.iter().enumerate() {
            let (cx, cy) = cell_of(a.position);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(others) = grid.get(&(cx + dx, cy + dy)) else {
                        continue;
                    };
                    // Each pair once: the one with the larger index pushes both.
                    for &j in others.iter().filter(|j| **j > i) {
                        let b = &self.enemies[j];
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
            }
        }
        pushes
    }

    /// Enemies left far behind appear again just beyond the side the player faces.
    pub(super) fn relocate_stragglers(&mut self) {
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
}

/// Where a body ends up pushed `by` from `from`, step by step, sliding along what stands in
/// its way.
pub(super) fn shove(map: &Map, from: Vec2, by: Vec2, half_width: f32, flies: bool) -> Vec2 {
    let step = by * (1.0 / f32::from(PUSH_STEPS));
    (0..PUSH_STEPS).fold(from, |at, _| slide(map, at, step, half_width, flies))
}

/// The nearest of the campfires `fires` to `at`, within its lure.
fn nearest_fire(fires: &[Vec2], at: Vec2) -> Option<Vec2> {
    fires
        .iter()
        .copied()
        .filter(|fire| (*fire - at).length() <= LURE_REACH)
        .min_by(|a, b| (*a - at).length().total_cmp(&(*b - at).length()))
}

/// What the enemies can tell of the player this tick: whether the scout is in their smoke;
/// whether they see them from afar; whether
/// the infiltrator is a ghost in the dark, just vanished, the Shadows still waiting; whether the
/// Shadows hunt (with the torch out), shun the light (lit), or neither (by day, or waiting).
#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "what the enemies can tell of the player, each a yes or a no"
)]
struct Sight {
    smoked: bool,
    seen: bool,
    ghost: bool,
    vanishing: bool,
    shadows_wait: bool,
    dark_hunt: Option<bool>,
}

/// Whether `enemy`, `apart` tiles from the player, has noticed them, as `sight` has it.
fn notice(enemy: &mut Enemy, apart: f32, sight: Sight) {
    // Waiting on the infiltrator, a Shadow loses its pace of the hunt too.
    if sight.shadows_wait && enemy.kind == EnemyKind::Shadow {
        enemy.hunting = 0;
    }
    if sight.smoked {
        if enemy.kind == EnemyKind::Shadow {
            enemy.hunting = 0;
        }
        enemy.noticed = apart <= GHOST_REACH;
    } else if enemy.horde.is_some() || enemy.kind == EnemyKind::Parasite {
        enemy.noticed = true;
    } else if enemy.kind == EnemyKind::Shadow && sight.dark_hunt.is_some() {
        // Shadows hunt in the dark, faster and faster; caught in the torch's light, they
        // bolt out of it.
        let hunts = sight.dark_hunt == Some(true);
        if enemy.noticed && !hunts {
            enemy.bolting = BOLT_TICKS;
        }
        enemy.hunting = if hunts { enemy.hunting + 1 } else { 0 };
        enemy.noticed = hunts;
    } else if sight.vanishing {
        enemy.noticed = apart <= GHOST_REACH;
    } else if sight.ghost {
        // Lightning may have shown them: those that saw them follow as ever.
        if apart <= GHOST_REACH {
            enemy.noticed = true;
        } else if apart > LOSE_REACH {
            enemy.noticed = false;
        }
    } else if sight.seen || apart <= NOTICE_REACH {
        enemy.noticed = true;
    } else if apart > LOSE_REACH {
        enemy.noticed = false;
    }
}
