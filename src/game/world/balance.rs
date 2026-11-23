//! Balance: runs played by a simple bot, many of them, to put numbers on the game's balance.
//! Not run with the other tests; on demand, in release for speed:
//!
//! ```sh
//! cargo test --release balance -- --ignored --nocapture
//! ```
//!
//! The bot flees the enemies near it, goes for the gems when none is close, keeps the torch lit,
//! and takes a card at random. With melee weapons alone, it goes at the nearest enemy instead,
//! backing off once surrounded or low on HP.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "counts and shares printed in a table"
)]

use std::thread;

use super::{
    Build, Choice, Input, Mode, Progress, Skin, TICKS_PER_SECOND, Vec2, Weapon, Weather, World,
    weather,
};

/// Half the map a terminal of 160 × 40 cells shows, in tiles: where the enemies come in from.
const VIEW: Vec2 = Vec2::new(40.0, 20.0);
/// The seeds played for each line of a table.
const SEEDS: u64 = 24;
/// How far off an enemy makes the bot flee, and a gem draws it, in tiles.
const FLEE: f32 = 6.0;
const GREED: f32 = 12.0;
/// The ticks the bot keeps a way out once stuck against an obstacle.
const UNSTUCK_TICKS: u32 = 20;
/// The ticks the bot takes to light its torch again once it is out, as a player would.
const REFLEX_TICKS: u32 = 15;
/// With melee weapons alone: how far off an enemy draws the bot, in tiles; how near it stops,
/// at striking distance; how many enemies close around it, and how low its HP, in percent, make
/// it back off; how near counts as close, in tiles.
const ENGAGE: f32 = 8.0;
const STRIKE: f32 = 1.3;
const SURROUNDED: usize = 4;
const LOW_HP_PERCENT: u32 = 40;
const CLOSE: f32 = 2.5;
/// The weapons that strike from afar: with any, the bot keeps away.
const FROM_AFAR: [Weapon; 10] = [
    Weapon::Pistol,
    Weapon::MachineGun,
    Weapon::Shotgun,
    Weapon::Shuriken,
    Weapon::Drones,
    Weapon::Molotov,
    Weapon::Napalm,
    Weapon::Grenade,
    Weapon::Sniper,
    Weapon::Flashball,
];

/// How a run is set up.
#[derive(Debug, Clone, Copy)]
enum Setup {
    /// A whole run: the crowbar to start with, a card at random on each level-up.
    Free,
    /// One weapon at one level, no card taken; the player can't die if `immortal`.
    Alone {
        weapon: Weapon,
        level: u8,
        immortal: bool,
    },
    /// A whole run as a hero: their own weapon, HP, speed and perk to start with, every card
    /// unlocked, a card at random on each level-up.
    Hero(Skin),
}

/// How a run ended.
#[derive(Debug, Clone, Copy)]
struct Outcome {
    seconds: u32,
    kills: u32,
    level: u32,
    dawn: bool,
    weather: Weather,
}

/// A run of `seed` as `setup` says, played by the bot until it is over or `limit` ticks in.
fn play(seed: u64, setup: Setup, limit: u32) -> Outcome {
    // A whole run as a first one: the cards dealt at first alone.
    let progress = match setup {
        Setup::Free => Progress::default(),
        Setup::Alone { .. } | Setup::Hero(_) => Progress::everything(),
    };
    let mut world = World::with(seed, Mode::Standard, false, progress);
    world.set_view(VIEW);
    if let Setup::Hero(hero) = setup {
        world.play_as(hero);
    }
    if let Setup::Alone { weapon, level, .. } = setup {
        let mut build = Build::unarmed();
        for _ in 0..level {
            build.take(Choice::Weapon(weapon));
        }
        world.player.build = build;
    }
    let mut bot = Bot::new(seed);
    while !world.is_over() && world.ticks < limit {
        if world.offer.is_some() {
            match setup {
                Setup::Free | Setup::Hero(_) => {
                    let count = world.offer.as_ref().map_or(1, Vec::len);
                    world.choose(bot.pick(count));
                }
                Setup::Alone { .. } => {
                    world.offer = None;
                    world.pending_levels = 0;
                }
            }
            continue;
        }
        bot.keep_torch_lit(&mut world);
        let input = bot.steer(&world);
        world.update(input);
        if let Setup::Alone { immortal: true, .. } = setup {
            world.player.hp = world.player.max_hp();
        }
    }
    Outcome {
        seconds: world.seconds(),
        kills: world.kills,
        level: world.player.level,
        dawn: world.survived(),
        weather: world.weather(),
    }
}

/// The bot: where it heads, and its luck at the cards.
struct Bot {
    random: u64,
    last: Vec2,
    checked_at: u32,
    detour: Option<(Vec2, u32)>,
    /// The tick it saw its torch out at, the night fallen.
    dark_since: Option<u32>,
}

impl Bot {
    fn new(seed: u64) -> Self {
        Self {
            random: seed.wrapping_mul(6_364_136_223_846_793_005) | 1,
            last: Vec2::ZERO,
            checked_at: 0,
            detour: None,
            dark_since: None,
        }
    }

    /// A card among `count`, at random.
    fn pick(&mut self, count: usize) -> usize {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        (self.random % count.max(1) as u64) as usize
    }

    /// Its torch out, the night fallen, the bot lights it again a moment later: at nightfall, and
    /// after lightning has put it out.
    fn keep_torch_lit(&mut self, world: &mut World) {
        if world.torch_on() != Some(false) {
            self.dark_since = None;
            return;
        }
        let since = *self.dark_since.get_or_insert(world.ticks);
        if world.ticks >= since + REFLEX_TICKS {
            world.switch_torch();
            self.dark_since = None;
        }
    }

    /// With melee weapons alone, neither surrounded nor low on HP: at the nearest enemy within
    /// reach, still once at striking distance, facing it.
    fn engage(world: &World) -> Option<Input> {
        let build = &world.player.build;
        if FROM_AFAR
            .iter()
            .any(|weapon| build.weapon_level(*weapon) > 0)
        {
            return None;
        }
        let position = world.player.position;
        let close = world
            .enemies
            .iter()
            .filter(|enemy| (enemy.position - position).length() < CLOSE)
            .count();
        let low = world.player.hp * 100 < world.player.max_hp() * LOW_HP_PERCENT;
        if close >= SURROUNDED || low {
            return None;
        }
        let to = world
            .enemies
            .iter()
            .map(|enemy| enemy.position - position)
            .filter(|to| to.length() < ENGAGE)
            .min_by(|a, b| a.length().total_cmp(&b.length()))?;
        Some(if to.length() > STRIKE {
            input(to)
        } else {
            Input::default()
        })
    }

    /// Away from the enemies near, or to the nearest gem; a way round when stuck.
    fn steer(&mut self, world: &World) -> Input {
        let ticks = world.ticks;
        let position = world.player.position;
        if let Some((way, until)) = self.detour {
            if ticks < until {
                return input(way);
            }
            self.detour = None;
        }
        if let Some(input) = Self::engage(world) {
            return input;
        }
        let mut flee = Vec2::ZERO;
        for enemy in &world.enemies {
            let away = position - enemy.position;
            let distance = away.length().max(0.1);
            if distance < FLEE {
                flee += away * (1.0 / (distance * distance));
            }
        }
        let way = if flee.length() > 0.01 {
            flee
        } else {
            world
                .gems
                .iter()
                .map(|gem| gem.position - position)
                .filter(|to| to.length() < GREED)
                .min_by(|a, b| a.length().total_cmp(&b.length()))
                .unwrap_or_else(|| {
                    let turn = ticks as f32 / 200.0;
                    Vec2::new(turn.cos(), turn.sin())
                })
        };
        if ticks >= self.checked_at + UNSTUCK_TICKS {
            if (position - self.last).length() < 0.3 && way.length() > 0.01 {
                let aside = Vec2::new(-way.y, way.x);
                self.detour = Some((aside, ticks + UNSTUCK_TICKS));
            }
            self.last = position;
            self.checked_at = ticks;
        }
        input(way)
    }
}

/// A way as the keys give it: -1, 0 or 1 on each axis.
fn input(way: Vec2) -> Input {
    let axis = |value: f32, other: f32| {
        if value.abs() < other.abs() * 0.4 {
            0.0
        } else {
            value.signum()
        }
    };
    Input {
        direction: Vec2::new(axis(way.x, way.y), axis(way.y, way.x)),
    }
}

/// Every seed of a line, played at once on the machine's cores.
fn outcomes(setup: Setup, limit: u32) -> Vec<Outcome> {
    thread::scope(|scope| {
        let runs: Vec<_> = (0..SEEDS)
            .map(|seed| scope.spawn(move || play(seed, setup, limit)))
            .collect();
        runs.into_iter()
            .map(|run| run.join().expect("a run panicked"))
            .collect()
    })
}

fn mean(values: impl Iterator<Item = f32>) -> f32 {
    let (sum, count) = values.fold((0.0, 0.0), |(sum, count), value| (sum + value, count + 1.0));
    if count > 0.0 { sum / count } else { 0.0 }
}

fn clock(seconds: f32) -> String {
    let seconds = seconds as u32;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

const FIRST_MINUTES: u32 = 4;

/// Each weapon alone, at levels 1, 3 and 5: how fast it kills (the player unable to die, the
/// dusk and the first minutes of the night), and how long it keeps the player alive.
#[test]
#[ignore = "balance, on demand"]
fn balance_weapons_alone() {
    let night = weather::DUSK_SECONDS * u32::from(TICKS_PER_SECOND);
    let kill_limit = night + FIRST_MINUTES * 60 * u32::from(TICKS_PER_SECOND);
    let run_limit = weather::dawn() + weather::daybreak();
    println!(
        "\nEach weapon alone, {SEEDS} seeds a line: kills a minute over the dusk and the first {FIRST_MINUTES} minutes (the player unable to die); the time held out, on average and at worst; the share reaching the dawn.\n"
    );
    println!(
        "{:<12} {:>5} {:>10} {:>9} {:>9} {:>6}",
        "weapon", "level", "kills/min", "held", "worst", "dawn"
    );
    for weapon in Weapon::ALL {
        for level in 1..=5 {
            let killing = outcomes(
                Setup::Alone {
                    weapon,
                    level,
                    immortal: true,
                },
                kill_limit,
            );
            let surviving = outcomes(
                Setup::Alone {
                    weapon,
                    level,
                    immortal: false,
                },
                run_limit,
            );
            let kill_rate = mean(
                killing
                    .iter()
                    .map(|run| run.kills as f32 * 60.0 / run.seconds.max(1) as f32),
            );
            let held = mean(surviving.iter().map(|run| run.seconds as f32));
            let worst = surviving.iter().map(|run| run.seconds).min().unwrap_or(0);
            let dawn = surviving.iter().filter(|run| run.dawn).count() * 100 / surviving.len();
            println!(
                "{:<12} {:>5} {:>10.1} {:>9} {:>9} {:>5}%",
                format!("{weapon:?}"),
                level,
                kill_rate,
                clock(held),
                clock(worst as f32),
                dawn
            );
        }
    }
}

/// Whole runs as each hero, every card unlocked, a card at random on each level-up: the kills a
/// minute, the time held out, on average and at worst, the level reached, the share reaching the
/// dawn.
#[test]
#[ignore = "balance, on demand"]
fn balance_heroes() {
    let limit = weather::dawn() + weather::daybreak();
    println!(
        "\nWhole runs as each hero, {SEEDS} seeds a line, every card unlocked, a card at random on each level-up.\n"
    );
    println!(
        "{:<13} {:>10} {:>9} {:>9} {:>6} {:>6}",
        "hero", "kills/min", "held", "worst", "level", "dawn"
    );
    for hero in Skin::ALL {
        let runs = outcomes(Setup::Hero(hero), limit);
        let kill_rate = mean(
            runs.iter()
                .map(|run| run.kills as f32 * 60.0 / run.seconds.max(1) as f32),
        );
        let held = mean(runs.iter().map(|run| run.seconds as f32));
        let worst = runs.iter().map(|run| run.seconds).min().unwrap_or(0);
        let level = mean(runs.iter().map(|run| run.level as f32));
        let dawn = runs.iter().filter(|run| run.dawn).count() * 100 / runs.len();
        println!(
            "{:<13} {:>10.1} {:>9} {:>9} {:>6.1} {:>5}%",
            format!("{hero:?}"),
            kill_rate,
            clock(held),
            clock(worst as f32),
            level,
            dawn
        );
    }
}

/// Whole runs, a card at random on each level-up: how many reach the dawn, and where the others
/// fall.
#[test]
#[ignore = "balance, on demand"]
fn balance_whole_runs() {
    let runs = outcomes(Setup::Free, weather::dawn() + weather::daybreak());
    let dawn = runs.iter().filter(|run| run.dawn).count();
    println!("\nWhole runs, {SEEDS} seeds, as a first run: the cards dealt at first, at random\n");
    println!("dawn reached: {dawn} of {}", runs.len());
    println!(
        "time held out: {} on average",
        clock(mean(runs.iter().map(|run| run.seconds as f32)))
    );
    println!(
        "level: {:.1} on average",
        mean(runs.iter().map(|run| run.level as f32))
    );
    println!(
        "kills: {:.0} on average",
        mean(runs.iter().map(|run| run.kills as f32))
    );
    for weather in [
        Weather::Dusk,
        Weather::Clear,
        Weather::Drizzle,
        Weather::Rain,
        Weather::Storm,
        Weather::Tempest,
    ] {
        let fallen = runs
            .iter()
            .filter(|run| !run.dawn && run.weather == weather)
            .count();
        println!("fallen in {weather:?}: {fallen}");
    }
}

#[test]
fn the_bot_lights_its_torch_once_the_night_falls_and_again_once_it_is_put_out() {
    let mut world = World::with(1, Mode::Standard, false, Progress::default());
    world.unhurt = true;
    let mut bot = Bot::new(1);
    // Until the tick asked for: announcements hold the ticks back a while.
    let mut run = |world: &mut World, until: u32| {
        while world.ticks < until {
            world.offer = None;
            world.pending_levels = 0;
            bot.keep_torch_lit(world);
            let input = bot.steer(world);
            world.update(input);
        }
    };
    let relit = REFLEX_TICKS + super::TORCH_FLICKER_MOST + 2;
    run(&mut world, super::torch_lights_up() + relit);
    assert!(world.torch_lit(), "lit at nightfall");
    world.cut_torch();
    let now = world.ticks;
    run(&mut world, now + relit);
    assert!(world.torch_lit(), "lit again after lightning");
}
