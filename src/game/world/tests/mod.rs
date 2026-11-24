use super::super::Obstacle;
use super::super::announce::{News, WeatherNews};
use super::super::barricade;
use super::super::baton;
use super::super::body::fits;
use super::super::build::{Action, BASE_HP, Build, Choice, Legendary, Passive, Weapon};
use super::super::burst;
use super::super::c4;
use super::super::enemy::EnemyKind;
use super::super::fists;
use super::super::gem::Gem;
use super::super::grenade;
use super::super::guns::{self, Bullet};
use super::super::hammer;
use super::super::machete;
use super::super::mode::Mode;
use super::super::napalm;
use super::super::progress::{Achievement, COLLECTION, Progress, Unlock};
use super::super::ram;
use super::super::sniper;
use super::super::spawn::{MARGIN, MAX_ENEMIES};
use super::Veil;
use std::sync::atomic::{AtomicU32, Ordering};

use super::super::drones;
use super::super::flamethrower;
use super::super::heal::{CAMPFIRE_HP, HERB_HP};
use super::super::horde::Warning;
use super::super::map::Feature;
use super::super::score;
use super::super::stain;
use super::*;

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "{actual} is not {expected}"
    );
}

fn moving(x: f32, y: f32) -> Input {
    Input {
        direction: Vec2::new(x, y),
    }
}

/// A run as the game starts it, past the dusk's announcement.
fn running(seed: u64) -> World {
    let mut world = World::new(seed);
    world.announcement = None;
    world
}

fn on(map: Map, start: Vec2) -> World {
    World::on(map, start, StdRng::seed_from_u64(1))
}

/// A run on an empty map but for a rock covering the tiles (20, 10) to (21, 11).
fn by_a_rock(start: Vec2) -> World {
    let mut map = Map::empty();
    map.put((20, 10), Obstacle::Rock);
    on(map, start)
}

/// No weapon at all: to watch enemies without them falling.
fn disarm(world: &mut World) {
    world.player.build = Build::unarmed();
}

/// A run with only `weapon`, at `level`, the player at (100, 50).
fn armed_with(weapon: Weapon, level: u8) -> World {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    for _ in 0..level {
        world.player.build.take(Choice::Weapon(weapon));
    }
    world
}

/// One enemy at `position`, its phase 0, with a number of its own.
fn enemy(kind: EnemyKind, position: Vec2) -> Enemy {
    static NEXT_ID: AtomicU32 = AtomicU32::new(1_000);
    Enemy::new(NEXT_ID.fetch_add(1, Ordering::Relaxed), kind, position, 0.0)
}

/// The world moved on by `ticks` ticks, the player still, no enemy appearing.
fn wait(world: &mut World, ticks: u32) {
    for _ in 0..ticks {
        world.spawn_debt = 0.0;
        world.update(Input::default());
    }
}

/// A horde come on the right of the player, every one of its enemies falling at once.
fn wipe_out_a_horde(world: &mut World) -> usize {
    world.set_view(Vec2::new(30.0, 12.0));
    world.bring_horde(Warning {
        arrives_at: 0,
        side: Some(Facing::Right),
        number: 0,
    });
    let count = world.enemies.len();
    for enemy in &mut world.enemies {
        enemy.hp = 0;
    }
    world.fell_enemies();
    count
}

/// A run in the night, the torch out, no horde nor lightning to come.
fn in_the_dark() -> World {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.next_strike_in = Some(u32::MAX);
    world.ticks = 30 * 40;
    world.switch_torch();
    world
}

/// How far a zombie 8 tiles off comes towards the player in a second, the torch lit, with
/// `glare` levels of Glare.
fn zombie_step(glare: u8) -> f32 {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.unhurt = true;
    world.next_horde = u32::MAX;
    world.next_strike_in = Some(u32::MAX);
    world.ticks = 30 * 40;
    world.torch_steady_at = 0;
    for _ in 0..glare {
        world.player.build.take(Choice::Passive(Passive::Glare));
    }
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(108.0, 50.0)));
    wait(&mut world, 30);
    108.0 - world.enemies[0].position.x
}

/// The HP got back as `count` enemies fall far from the player, at `level` of Drain.
fn drained(level: u8, count: usize) -> u32 {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.player.hp = 1;
    for _ in 0..level {
        world.player.build.take(Choice::Passive(Passive::Drain));
    }
    for _ in 0..count {
        let mut felled = enemy(EnemyKind::Zombie, Vec2::new(130.0, 50.0));
        felled.hp = 0;
        world.enemies.push(felled);
    }
    wait(&mut world, 1);
    world.player.hp - 1
}

/// A run with the Barricade card at `level`, the player at (100.5, 50.5) facing right, unarmed.
fn barricading(level: u8) -> World {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    for _ in 0..level {
        world.player.build.take(Choice::Action(Action::Barricade));
    }
    world
}

/// The flamethrower at `level`, the player at (100.5, 50.5) facing right, enough enemies ahead
/// to light it, stunned and tough, and one behind, the last.
fn flaming(level: u8) -> World {
    let mut world = armed_with(Weapon::Flamethrower, level);
    world.next_horde = u32::MAX;
    world.player.position = Vec2::new(100.5, 50.5);
    world.player.facing = Facing::Right;
    for (x, y) in [(102.5, 50.5), (102.5, 50.0), (102.5, 51.0), (98.5, 50.5)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    world
}

/// The C4 at `level`, the player's own action, an enemy stunned and tough at the player's feet,
/// one within its blast, one well away.
fn charged(level: u8) -> World {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.announcement = None;
    for _ in 0..level {
        world.player.build.take(Choice::Action(Action::C4));
    }
    for x in [100.0, 102.0, 110.0] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    world
}

/// The napalm at `level`, a crowd stunned and tough 5 tiles right of the player, one apart
/// off its strip, and one well out of reach.
fn napalm_on_a_crowd(level: u8) -> World {
    let mut world = armed_with(Weapon::Napalm, level);
    world.next_horde = u32::MAX;
    for (x, y) in [
        (105.0, 50.0),
        (105.5, 50.0),
        (105.0, 50.5),
        (104.0, 53.0),
        (130.0, 50.0),
    ] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    world
}

/// The infiltrator, their torch out, the night fallen, their fists alone.
fn infiltrator_in_the_dark() -> World {
    let mut world = in_the_dark();
    world.play_as(Skin::Infiltrator);
    world.announcement = None;
    assert_eq!(world.torch_on(), Some(false));
    world
}

fn ram_throw_ticks() -> u32 {
    ram::THROW_TICKS
}

mod blasts;
mod chargers;
mod heroes;
mod kamikazes;
mod melee;
mod night;
mod progress;
mod ranged;
mod run;
mod spitters;
