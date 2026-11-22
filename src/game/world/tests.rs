use super::super::Obstacle;
use super::super::announce::{News, WeatherNews};
use super::super::barricade;
use super::super::body::fits;
use super::super::build::{Action, BASE_HP, Build, Choice, Legendary, Passive, Weapon};
use super::super::burst;
use super::super::enemy::EnemyKind;
use super::super::gem::Gem;
use super::super::guns::{self, Bullet};
use super::super::hammer;
use super::super::mode::Mode;
use super::super::progress::{Achievement, Progress, Unlock};
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

#[test]
fn standing_still_stays_put() {
    let mut world = running(1);
    let start = world.player.position;
    world.update(Input::default());
    assert_eq!(world.player.position, start);
}

#[test]
fn a_second_of_moving_covers_the_players_speed() {
    let mut world = running(1);
    let start = world.player.position;
    for _ in 0..30 {
        world.update(moving(1.0, 0.0));
    }
    assert_near(world.player.position.x - start.x, PLAYER_SPEED);
    assert_near(world.player.position.y, start.y);
}

#[test]
fn a_diagonal_is_no_faster_than_a_straight_line() {
    let mut world = running(1);
    let start = world.player.position;
    world.update(moving(1.0, -1.0));
    let moved = world.player.position - start;
    assert_near(moved.length(), PLAYER_SPEED * tick_seconds());
    assert!(moved.x > 0.0 && moved.y < 0.0);
}

#[test]
fn the_player_faces_where_they_go() {
    let mut world = running(1);
    world.update(moving(-1.0, 0.0));
    world.update(Input::default());
    assert_eq!(world.player.facing, Facing::Left);
}

#[test]
fn the_map_has_no_edge() {
    let mut world = on(Map::empty(), Vec2::new(0.5, 0.5));
    for _ in 0..30 * 60 {
        world.update(moving(-1.0, -1.0));
        world.player.hp = BASE_HP;
    }
    let position = world.player.position;
    assert!(position.x < -200.0 && position.y < -200.0, "{position:?}");
}

#[test]
fn an_obstacle_stops_the_player_against_it() {
    let mut world = by_a_rock(Vec2::new(17.5, 10.5));
    for _ in 0..30 {
        world.update(moving(1.0, 0.0));
    }
    assert_near(world.player.position.x, 20.0 - PLAYER_HALF_WIDTH);
    assert_near(world.player.position.y, 10.5);
}

#[test]
fn a_blocked_diagonal_slides_along_the_obstacle() {
    let mut world = by_a_rock(Vec2::new(19.0, 10.5));
    for _ in 0..10 {
        world.update(moving(1.0, 1.0));
    }
    let position = world.player.position;
    assert_near(position.x, 20.0 - PLAYER_HALF_WIDTH);
    assert!(position.y > 11.0, "still at {position:?}");
}

#[test]
fn the_player_slides_out_past_the_obstacles_corner() {
    let mut world = by_a_rock(Vec2::new(20.0 - PLAYER_HALF_WIDTH, 11.0));
    for _ in 0..30 {
        world.update(moving(1.0, 1.0));
    }
    assert!(
        world.player.position.x > 22.0,
        "{:?}",
        world.player.position
    );
}

#[test]
fn an_enemy_appears_after_a_second() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    for _ in 0..29 {
        world.update(Input::default());
    }
    assert_eq!(world.enemies.len(), 0);
    world.update(Input::default());
    assert_eq!(world.enemies.len(), 1);
}

#[test]
fn enemies_appear_beyond_the_view_and_clear_of_obstacles() {
    let mut world = running(9);
    world.set_view(Vec2::new(20.0, 8.0));
    for _ in 0..30 * 60 {
        world.update(Input::default());
        world.player.hp = BASE_HP;
        let centre = world.player.position;
        if let Some(new) = world.enemies.last()
            && world.ticks.is_multiple_of(30)
        {
            let apart = new.position - centre;
            assert!(apart.x.abs() > 20.0 || apart.y.abs() > 8.0);
        }
    }
    assert_ne!(world.enemies.len(), 0);
    for enemy in &world.enemies {
        let stats = enemy.kind.stats();
        assert!(fits(
            &world.map,
            enemy.position,
            stats.half_width,
            stats.flies
        ));
    }
}

#[test]
fn there_are_never_more_than_the_most_enemies() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    // No horde: it has a most of its own; no lightning, which fells some.
    world.next_horde = u32::MAX;
    world.next_strike_in = Some(u32::MAX);
    world.ticks = 30 * 60 * 9;
    for _ in 0..30 * 60 {
        world.update(Input::default());
        world.player.hp = BASE_HP;
    }
    assert_eq!(world.enemies.len(), MAX_ENEMIES);
}

#[test]
fn a_zombie_walks_to_the_player() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    wait(&mut world, 30);
    assert_near(world.enemies[0].position.x, 110.0 - 2.5);
    assert_eq!(world.enemies[0].facing, Facing::Left);
}

#[test]
fn a_rock_stops_a_zombie_but_not_a_shadow() {
    let mut world = by_a_rock(Vec2::new(17.0, 11.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(24.0, 11.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(24.0, 11.0)));
    wait(&mut world, 45);
    let (zombie, shadow) = (&world.enemies[0], &world.enemies[1]);
    assert!(zombie.position.x >= 22.0 + 0.45 - 1e-3, "{zombie:?}");
    assert!(shadow.position.x < 20.0, "{shadow:?}");
}

#[test]
fn a_gun_holds_its_fire_at_an_enemy_behind_a_rock_and_shoots_once_it_shows() {
    let mut world = by_a_rock(Vec2::new(17.0, 11.0));
    disarm(&mut world);
    world.player.build.take(Choice::Weapon(Weapon::Pistol));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(24.0, 11.0)));
    world.enemies[0].stunned = 1_000;
    wait(&mut world, 10);
    assert_eq!(world.bullets, [], "behind the rock");
    world.enemies[0].position = Vec2::new(24.0, 16.0);
    wait(&mut world, 10);
    assert!(
        !world.bullets.is_empty() || world.enemies[0].hp < 40,
        "in sight"
    );
}

#[test]
fn a_rock_stops_a_bullet_a_spark_where_it_hits() {
    let mut world = by_a_rock(Vec2::new(17.0, 11.0));
    disarm(&mut world);
    let gun = guns::gun(Weapon::Pistol, 1).expect("a gun");
    world.bullets.push(Bullet::toward(
        Vec2::new(18.0, 11.0),
        Vec2::new(30.0, 11.0),
        &gun,
        10,
    ));
    // Until it stops: its spark there just then.
    for _ in 0..10 {
        if world.bullets.is_empty() {
            break;
        }
        wait(&mut world, 1);
    }
    assert_eq!(world.bullets, []);
    assert_eq!(world.sparks.len(), 1);
    assert!((world.sparks[0].0.x - 20.0).abs() < 1.0);
}

#[test]
fn enemies_on_one_spot_move_apart() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    let spot = Vec2::new(100.0, 40.0);
    world.enemies.push(enemy(EnemyKind::Zombie, spot));
    world.enemies.push(enemy(EnemyKind::Zombie, spot));
    wait(&mut world, 10);
    let apart = world.enemies[0].position - world.enemies[1].position;
    assert!(apart.length() > 0.5, "{apart:?}");
}

#[test]
fn a_touch_hurts_then_the_player_is_safe_for_half_a_second() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(100.5, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.player.hp, BASE_HP - 5);
    wait(&mut world, 15);
    assert_eq!(world.player.hp, BASE_HP - 5);
    wait(&mut world, 1);
    assert_eq!(world.player.hp, BASE_HP - 10);
}

#[test]
fn the_hardest_hit_counts() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(100.5, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(99.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.player.hp, BASE_HP - 15);
}

#[test]
fn at_zero_hp_the_run_is_over_and_stops() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.player.hp = 3;
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(100.5, 50.0)));
    wait(&mut world, 1);
    assert!(world.is_over());
    let ticks = world.ticks();
    world.update(moving(1.0, 0.0));
    assert_eq!(world.ticks(), ticks);
}

#[test]
fn an_enemy_left_far_behind_appears_again_ahead() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.set_view(Vec2::new(20.0, 8.0));
    world.player.facing = Facing::Right;
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(40.0, 50.0)));
    wait(&mut world, 1);
    let x = world.enemies[0].position.x;
    assert!((x - (100.0 + 20.0 + MARGIN)).abs() < 0.2, "at {x}");
}

#[test]
fn an_abomination_always_leaves_a_gem_a_zombie_three_times_in_four() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    let far = Vec2::new(150.0, 50.0);
    world.enemies.push(enemy(EnemyKind::Abomination, far));
    world.enemies[0].hp = 0;
    world.fell_enemies();
    assert_eq!(world.gems.len(), 1);
    assert_eq!(world.gems[0].value, 5);

    world.gems.clear();
    for _ in 0..200 {
        world.enemies.push(enemy(EnemyKind::Zombie, far));
        world.enemies[0].hp = 0;
        world.fell_enemies();
    }
    assert!(
        (125..=175).contains(&world.gems.len()),
        "{}",
        world.gems.len()
    );
    assert_eq!(world.kills, 201);
}

#[test]
fn a_near_gem_flies_to_the_player_and_a_far_one_stays() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(101.5, 50.0), 3));
    world.gems.push(Gem::new(Vec2::new(105.0, 50.0), 1));
    wait(&mut world, 10);
    assert_eq!(world.gems.len(), 1);
    assert_eq!(world.gems[0].position, Vec2::new(105.0, 50.0));
    assert_eq!(world.player.xp, 3);
}

#[test]
fn a_level_up_opens_an_offer_and_the_world_waits() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 5));
    wait(&mut world, 1);
    assert_eq!(world.player.level, 2);
    assert_eq!(world.offer.as_ref().map(Vec::len), Some(3));
    let ticks = world.ticks();
    wait(&mut world, 10);
    assert_eq!(world.ticks(), ticks);
}

#[test]
fn a_run_deals_the_cards_again_once() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 5));
    wait(&mut world, 1);
    assert_eq!(world.rerolls, 1);
    world.reroll();
    assert_eq!(world.rerolls, 0);
    assert_eq!(world.offer.as_ref().map(Vec::len), Some(3));
    let dealt = world.offer.clone();
    world.reroll();
    assert_eq!(world.offer, dealt, "none left");
}

#[test]
fn choosing_takes_the_choice_and_the_world_goes_on() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 5));
    wait(&mut world, 1);
    let choice = world.offer.as_ref().unwrap()[1];
    world.choose(1);
    assert_eq!(world.offer, None);
    assert_eq!(world.player.build.next_level(choice), 2);
    let ticks = world.ticks();
    wait(&mut world, 1);
    assert_eq!(world.ticks(), ticks + 1);
}

#[test]
fn two_levels_at_once_make_two_offers_in_a_row() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 13));
    wait(&mut world, 1);
    assert_eq!(world.player.level, 3);
    world.choose(0);
    assert!(world.offer.is_some());
    world.choose(0);
    assert_eq!(world.offer, None);
}

#[test]
fn vitality_raises_the_hp_and_heals_as_much() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.player.hp = 50;
    world.pending_levels = 1;
    world.offer = Some(vec![Choice::Passive(Passive::Vitality)]);
    world.choose(0);
    assert_eq!(world.player.max_hp(), BASE_HP + 20);
    assert_eq!(world.player.hp, 70);
}

#[test]
fn enough_xp_makes_a_level_and_the_rest_carries_over() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 7));
    wait(&mut world, 1);
    assert_eq!((world.player.level, world.player.xp), (2, 2));
    world.choose(0);
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 6 + 11));
    wait(&mut world, 1);
    assert_eq!((world.player.level, world.player.xp), (4, 0));
}

#[test]
fn a_run_starts_with_the_crowbar() {
    let world = World::new(1);
    assert_eq!(world.player.build.weapon_level(Weapon::Crowbar), 1);
    assert_eq!(world.ammo(), []);
}

#[test]
fn the_machete_slashes_enemies_in_front_not_behind() {
    let mut world = armed_with(Weapon::Machete, 1);
    world.player.facing = Facing::Right;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(98.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 40 - 12);
    assert_eq!(world.enemies[1].hp, 40);
    assert!(world.swing.is_some());
}

#[test]
fn the_machete_rests_while_no_enemy_is_near() {
    let mut world = armed_with(Weapon::Machete, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.swing, None);
}

#[test]
fn from_level_three_the_second_slash_strikes_behind() {
    let mut world = armed_with(Weapon::Machete, 3);
    world.player.facing = Facing::Right;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(98.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!((world.enemies[0].hp, world.enemies[1].hp), (40 - 12, 40));
    wait(&mut world, 5);
    assert_eq!((world.enemies[0].hp, world.enemies[1].hp), (40 - 12, 40));
    wait(&mut world, 1);
    assert_eq!(
        (world.enemies[0].hp, world.enemies[1].hp),
        (40 - 12, 40 - 12)
    );
    assert_eq!(world.swing.map(|swing| swing.facing), Some(Facing::Left));
}

#[test]
fn at_level_five_a_swing_strikes_ahead_behind_then_ahead() {
    let mut world = armed_with(Weapon::Machete, 5);
    world.player.facing = Facing::Right;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(98.0, 50.0)));
    wait(&mut world, 13);
    assert_eq!(
        (world.enemies[0].hp, world.enemies[1].hp),
        (40 - 24, 40 - 12)
    );
    // The swing is over until the machete is ready again.
    wait(&mut world, 20);
    assert_eq!(
        (world.enemies[0].hp, world.enemies[1].hp),
        (40 - 24, 40 - 12)
    );
}

#[test]
fn the_pistol_shoots_the_nearest_enemy_in_range() {
    let mut world = armed_with(Weapon::Pistol, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(100.0, 44.0)));
    wait(&mut world, 1);
    assert_eq!(world.bullets.len(), 1);
    let bullet = world.bullets[0].position;
    assert!(
        bullet.y < 50.0 && (bullet.x - 100.0).abs() < 1e-3,
        "{bullet:?}"
    );
}

#[test]
fn the_pistol_holds_fire_with_no_enemy_in_range() {
    let mut world = armed_with(Weapon::Pistol, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(120.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.bullets.len(), 0);
}

#[test]
fn the_pistol_empties_its_magazine_then_reloads() {
    let mut world = armed_with(Weapon::Pistol, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(111.0, 50.0)));
    world.enemies[0].hp = 100_000;
    let mut shots = Vec::new();
    for tick in 0..150 {
        let before = world.bullets.len();
        world.enemies[0].position = Vec2::new(111.0, 50.0);
        wait(&mut world, 1);
        if world.bullets.len() > before {
            shots.push(tick);
        }
    }
    // 6 rounds 0.5 s apart, a 2-second reload, then the next 6.
    assert_eq!(&shots[..7], [0, 16, 32, 48, 64, 80, 141]);
}

#[test]
fn two_bullets_fell_a_zombie() {
    let mut world = armed_with(Weapon::Pistol, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(105.0, 50.0)));
    wait(&mut world, 15);
    assert_eq!(world.enemies.len(), 1);
    wait(&mut world, 20);
    assert_eq!(world.enemies.len(), 0);
    assert_eq!(world.kills, 1);
    assert_eq!(world.bullets.len(), 0);
}

#[test]
fn the_pistol_at_level_four_shoots_the_two_nearest() {
    let mut world = armed_with(Weapon::Pistol, 4);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(105.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(95.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.bullets.len(), 2);
    assert!(world.bullets.iter().any(|bullet| bullet.position.x > 100.0));
    assert!(world.bullets.iter().any(|bullet| bullet.position.x < 100.0));
}

#[test]
fn the_machine_gun_fires_a_burst_of_its_magazine() {
    let mut world = armed_with(Weapon::MachineGun, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(109.0, 50.0)));
    world.enemies[0].hp = 100_000;
    let rounds = |world: &World| world.ammo()[0].1.rounds;
    // A shot every 3 ticks (its wait, then the shot): the 8 rounds in 24 ticks, then the
    // reload.
    wait(&mut world, 1);
    assert_eq!(rounds(&world), 7);
    wait(&mut world, 3 * 6);
    assert_eq!(rounds(&world), 1);
    assert_eq!(world.ammo()[0].1.reloading, 0);
    wait(&mut world, 3);
    assert!(world.ammo()[0].1.reloading > 0);
}

#[test]
fn a_piercing_bullet_goes_through_one_enemy() {
    let mut world = armed_with(Weapon::MachineGun, 5);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(104.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(106.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(108.0, 50.0)));
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
    }
    wait(&mut world, 1);
    let mut bullet = world.bullets.pop().unwrap();
    let start = world
        .enemies
        .iter()
        .map(|enemy| enemy.position)
        .collect::<Vec<_>>();
    world.bullets.clear();
    world.player.build = Build::unarmed();
    bullet.pierce = 1;
    world.bullets.push(bullet);
    for _ in 0..30 {
        for (enemy, position) in world.enemies.iter_mut().zip(&start) {
            enemy.position = *position;
        }
        wait(&mut world, 1);
    }
    let hit: Vec<bool> = world.enemies.iter().map(|enemy| enemy.hp < 1_000).collect();
    assert_eq!(hit, [true, true, false]);
}

#[test]
fn drones_shoot_at_the_enemies_within_their_reach_again_and_again() {
    let mut world = armed_with(Weapon::Drones, 1);
    // Out of the drones' way, standing still: within their reach, never touching them.
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(107.0, 50.0)));
    world.enemies[0].hp = 1_000;
    world.enemies[0].stunned = 1_000;
    wait(&mut world, 30);
    let hp = world.enemies[0].hp;
    assert!(hp < 1_000, "hit");
    wait(&mut world, 30);
    assert!(world.enemies[0].hp < hp, "hit again");
}

#[test]
fn a_drone_touched_by_an_enemy_breaks_and_repairs_in_three_seconds() {
    let mut world = armed_with(Weapon::Drones, 1);
    let next = drones::positions(world.player.position, 1, drones::turn_per_tick(1))[0];
    world.enemies.push(enemy(EnemyKind::Abomination, next));
    world.enemies[0].hp = 1_000;
    world.enemies[0].stunned = 1_000;
    wait(&mut world, 1);
    assert!(world.drones()[0].1, "broken");
    assert_eq!(world.enemies[0].hp, 1_000 - 4, "struck a last time");
    // Out of the way, the drone repairs.
    world.enemies[0].position = Vec2::new(150.0, 50.0);
    wait(&mut world, 89);
    assert!(world.drones()[0].1, "still broken");
    wait(&mut world, 1);
    assert!(!world.drones()[0].1, "repaired");
}

#[test]
fn without_drones_there_are_no_drones() {
    let world = on(Map::empty(), Vec2::new(100.0, 50.0));
    assert_eq!(world.drones().len(), 0);
}

#[test]
fn a_fallen_enemy_and_a_hit_player_leave_stains_that_fade() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    // In the night, past the dusk: no news to stand still for while the stains fade.
    world.ticks = 30 * 40;
    world.next_horde = u32::MAX;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(150.0, 50.0)));
    world.enemies[0].hp = 0;
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(100.5, 50.0)));
    wait(&mut world, 1);
    let sizes: Vec<u8> = world.stains.iter().map(|stain| stain.size).collect();
    assert_eq!(sizes, [6, 1]);
    world.enemies.clear();
    world.swellings.clear();
    wait(&mut world, stain::lasts());
    assert_eq!(world.stains.len(), 0);
}

#[test]
fn a_clear_night_has_no_lightning() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    wait(&mut world, 30 * 110);
    assert_eq!(world.lightning, None);
}

#[test]
fn lightning_strikes_from_the_storm_on_and_not_in_the_rain() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.ticks = 30 * 300;
    wait(&mut world, 30 * 13);
    assert!(world.lightning.is_none());
    world.ticks = 30 * 420;
    wait(&mut world, 30 * 13);
    assert!(world.lightning.is_some());
}

#[test]
fn a_storms_lightning_hurts_the_enemies_it_falls_among_and_sets_them_on_fire() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 480;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(110.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(120.0, 50.0)));
    world.strike(Vec2::new(111.0, 50.0), world.weather().strikes_hurt());
    assert_eq!((world.enemies[0].hp, world.enemies[1].hp), (40 - 30, 40));
    assert!(world.enemies[0].burning > 0, "on fire");
    assert_eq!(world.enemies[1].burning, 0, "out of its ring");
}

#[test]
fn a_storms_strike_shows_its_ring_before_it_falls_on_the_player() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.next_strike_in = Some(u32::MAX);
    world.ticks = 30 * 480;
    world.strikes_coming.push(Lightning {
        tick: world.ticks,
        at: Vec2::new(101.0, 50.0),
    });
    // The ring shows; nothing falls yet.
    wait(&mut world, weather::STRIKE_WARNING_TICKS - 1);
    assert_eq!(world.player.hp, BASE_HP);
    assert!(world.lightning.is_none());
    // Half a second on, it falls: the player, still in the ring, unhurt, their torch out.
    wait(&mut world, 1);
    assert!(world.lightning.is_some());
    assert_eq!(world.strikes_coming, []);
    assert_eq!(world.player.hp, BASE_HP);
    assert_eq!(world.torch_on(), Some(false));
}

#[test]
fn a_strike_never_hurts_the_player() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 480;
    world.strike(world.player.position, true);
    assert_eq!(world.player.hp, BASE_HP);
}

#[test]
fn the_drizzle_unlocks_haste_and_300_kills_in_a_run_might() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.progress = Progress::default();
    world.next_horde = u32::MAX;
    world.ticks = (weather::DUSK_SECONDS + 2 * weather::stage_seconds(Mode::Standard)) * 30 - 1;
    wait(&mut world, 1);
    assert!(world.progress.deals(Choice::Passive(Passive::Haste)));
    assert!(!world.progress.deals(Choice::Passive(Passive::Might)));
    // The drizzle told, the world going on.
    world.announcement = None;
    world.kills = 300;
    wait(&mut world, 1);
    assert!(world.progress.deals(Choice::Passive(Passive::Might)));
}

#[test]
fn reaching_the_rain_unlocks_the_machine_gun_for_the_next_runs() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.progress = Progress::default();
    world.next_horde = u32::MAX;
    world.ticks = (weather::DUSK_SECONDS + 3 * weather::stage_seconds(Mode::Standard)) * 30 - 1;
    assert!(!world.progress.deals(Choice::Weapon(Weapon::MachineGun)));
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::Rain));
    assert!(world.progress.deals(Choice::Weapon(Weapon::MachineGun)));
    // Dealt from the next run on, not this one.
    world.dealt = Progress::default();
    let next = World::with(2, Mode::Standard, false, world.progress.clone());
    assert!(next.dealt.deals(Choice::Weapon(Weapon::MachineGun)));
}

#[test]
fn struck_by_lightning_and_standing_is_an_achievement() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.progress = Progress::default();
    world.ticks = 30 * 480;
    world.strike(Vec2::new(100.5, 50.0), true);
    assert!(world.player.hp > 0);
    assert!(world.progress.achieved.contains(&Achievement::Struck));
}

#[test]
fn a_run_counts_itself_and_its_kills_in_the_progress() {
    let world = World::with(1, Mode::Standard, false, Progress::default());
    assert_eq!(world.progress.runs, 1);
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    let count = wipe_out_a_horde(&mut world);
    assert_eq!(world.progress.kills, u64::try_from(count).unwrap());
}

#[test]
fn enemies_appearing_later_are_tougher() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 600;
    while world.enemies.is_empty() {
        world.update(Input::default());
    }
    let enemy = &world.enemies[0];
    assert_eq!(enemy.hp, enemy.kind.stats().hp * 200 / 100);
}

#[test]
fn the_weather_dims_the_light() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    let clear = world.light_radius();
    world.ticks = 30 * 480;
    assert!((clear - world.light_radius() - 2.0).abs() < 1e-4);
}

#[test]
fn kills_score_their_points_and_build_a_combo_that_closes() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    let far = Vec2::new(150.0, 50.0);
    for _ in 0..3 {
        world.enemies.push(enemy(EnemyKind::Zombie, far));
        world.enemies[0].hp = 0;
        wait(&mut world, 1);
    }
    // 10, then 11 and 12: the combo adds a tenth per kill.
    assert_eq!((world.score, world.combo), (33, 3));
    wait(&mut world, score::combo_window() + 1);
    assert_eq!(world.combo, 0);
}

#[test]
fn the_weather_multiplies_the_score() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 600;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(150.0, 50.0)));
    world.enemies[0].hp = 0;
    wait(&mut world, 1);
    assert_eq!(world.score, 500);
    assert_eq!(world.multiplier_tenths(), 55);
}

#[test]
fn a_hurt_player_picks_up_a_herb_and_heals() {
    let mut map = Map::empty();
    map.put_feature((100, 50), Feature::Herb);
    let mut world = on(map, Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.player.hp = 50;
    wait(&mut world, 1);
    assert_eq!(world.player.hp, 50 + HERB_HP);
    assert_eq!(world.features_in((99, 49), (101, 51)), []);
}

#[test]
fn a_herb_is_picked_up_within_two_tiles_magnet_or_not() {
    let mut map = Map::empty();
    map.put_feature((100, 50), Feature::Herb);
    // 2.5 tiles from it: out of reach, even with Magnet.
    let mut world = on(map, Vec2::new(103.0, 50.5));
    disarm(&mut world);
    world.player.hp = 50;
    for _ in 0..3 {
        world.player.build.take(Choice::Passive(Passive::Magnet));
    }
    wait(&mut world, 1);
    assert_eq!(world.player.hp, 50);
    // 1.8 tiles from it: picked up.
    world.player.position = Vec2::new(102.3, 50.5);
    wait(&mut world, 1);
    assert_eq!(world.player.hp, 50 + HERB_HP);
}

#[test]
fn a_player_in_full_health_leaves_the_herb_for_later() {
    let mut map = Map::empty();
    map.put_feature((100, 50), Feature::Herb);
    let mut world = on(map, Vec2::new(100.5, 50.5));
    disarm(&mut world);
    wait(&mut world, 1);
    assert_eq!(world.features_in((99, 49), (101, 51)).len(), 1);
}

#[test]
fn a_campfire_heals_three_hp_a_second_until_it_dies_out() {
    let mut map = Map::empty();
    map.put_feature((101, 50), Feature::Campfire);
    let mut world = on(map, Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.player.hp = 10;
    wait(&mut world, 30);
    assert_eq!(world.player.hp, 13);
    wait(&mut world, 30 * 20);
    assert_eq!(world.player.hp, 10 + CAMPFIRE_HP);
    assert_eq!(world.campfire_left((101, 50)), 0);
}

#[test]
fn a_horde_is_announced_then_comes_out_of_its_side() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.set_view(Vec2::new(30.0, 12.0));
    world.ticks = 30 * 87 - 1;
    wait(&mut world, 1);
    let warning = world.horde_warning.expect("announced 3 seconds before");
    assert_eq!(warning.arrives_at, 30 * 90);
    assert!(warning.side.is_some());
    assert_eq!(
        world.announcement.map(|told| told.news),
        Some(News::Horde { everywhere: false })
    );
    world.enemies.clear();
    // The announcement, 2 seconds, then the 3 of the warning.
    wait(&mut world, 30 * 2 + 30 * 3);
    assert_eq!(world.horde_warning, None);
    let horde: Vec<_> = world
        .enemies
        .iter()
        .filter(|enemy| enemy.horde == Some(0))
        .collect();
    assert_eq!(horde.len(), 30);
}

#[test]
fn from_the_storm_on_a_horde_comes_from_every_side() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.ticks = 30 * 480;
    world.next_horde = 30 * 482;
    wait(&mut world, 1);
    assert_eq!(world.horde_warning.map(|warning| warning.side), Some(None));
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

#[test]
fn a_horde_wiped_out_brings_a_level() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    let count = wipe_out_a_horde(&mut world);
    assert!(count > 0);
    assert_eq!(world.horde_cleared.map(|(_, bonus, _)| bonus), Some(0));
    // Once told, two seconds on.
    assert_eq!(world.player.level, 1);
    wait(&mut world, 30 * 2);
    assert_eq!(world.player.level, 2);
}

#[test]
fn a_horde_wiped_out_is_the_hunters_achievement() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    wipe_out_a_horde(&mut world);
    assert!(world.progress.achieved.contains(&Achievement::Hunter));
    assert!(world.progress.deals(Choice::Passive(Passive::HordeHunter)));
}

#[test]
fn two_hordes_wiped_out_in_a_run_unlock_the_drones() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    wipe_out_a_horde(&mut world);
    assert!(!world.progress.deals(Choice::Weapon(Weapon::Drones)));
    wipe_out_a_horde(&mut world);
    assert!(world.progress.deals(Choice::Weapon(Weapon::Drones)));
}

#[test]
fn horde_hunter_veils_more_of_a_horde_with_its_levels() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.set_view(Vec2::new(30.0, 12.0));
    world.bring_horde(Warning {
        arrives_at: 0,
        side: Some(Facing::Right),
        number: 0,
    });
    let first = world.enemies[0].clone();
    assert_eq!(world.horde_veil(&first), None, "not without the card");
    world
        .player
        .build
        .take(Choice::Passive(Passive::HordeHunter));
    assert_eq!(
        world.horde_veil(&first),
        None,
        "the whole horde still stands"
    );
    // Down to its last one: it pulses.
    let size = world.enemies.len();
    for enemy in &mut world.enemies[1..] {
        enemy.hp = 0;
    }
    world.fell_enemies();
    assert_eq!(world.enemies.len(), 1);
    assert_eq!(world.horde_veil(&world.enemies[0]), Some(Veil::Pulsing));
    assert!(size > 1);
}

#[test]
fn at_its_last_level_horde_hunter_veils_the_whole_horde_and_shows_it_in_the_dark() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.set_view(Vec2::new(30.0, 12.0));
    for _ in 0..5 {
        world
            .player
            .build
            .take(Choice::Passive(Passive::HordeHunter));
    }
    world.bring_horde(Warning {
        arrives_at: 0,
        side: Some(Facing::Right),
        number: 0,
    });
    assert!(world.player.build.horde_veil_in_dark());
    assert_eq!(world.horde_veil(&world.enemies[0]), Some(Veil::Steady));
}

#[test]
fn horde_hunter_hurts_a_hordes_enemies_more() {
    let mut world = armed_with(Weapon::Crowbar, 1);
    world
        .player
        .build
        .take(Choice::Passive(Passive::HordeHunter));
    world
        .player
        .build
        .take(Choice::Passive(Passive::HordeHunter));
    world.player.facing = Facing::Right;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.5)));
    world.enemies[0].hp = 1_000;
    world.enemies[1].hp = 1_000;
    world.enemies[1].horde = Some(0);
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 1_000 - 18);
    assert_eq!(world.enemies[1].hp, 1_000 - 18 * 120 / 100);
}

#[test]
fn hordes_standing_together_are_counted_apart_and_told_apart_once_wiped_out() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.set_view(Vec2::new(30.0, 12.0));
    for (number, side) in [(0, Facing::Right), (1, Facing::Left)] {
        world.bring_horde(Warning {
            arrives_at: 0,
            side: Some(side),
            number,
        });
    }
    let hordes = world.hordes_left();
    assert_eq!(
        hordes.iter().map(|&(number, _)| number).collect::<Vec<_>>(),
        [0, 1]
    );
    for enemy in world
        .enemies
        .iter_mut()
        .filter(|enemy| enemy.horde == Some(0))
    {
        enemy.hp = 0;
    }
    world.fell_enemies();
    assert_eq!(world.hordes_left(), [hordes[1]]);
    assert_eq!(
        world.horde_cleared.map(|(_, _, number)| number),
        Some(Some(0))
    );
}

#[test]
fn in_arcade_a_horde_wiped_out_doubles_its_kills_too() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.mode = Mode::Arcade;
    let count = wipe_out_a_horde(&mut world);
    let (_, bonus, _) = world.horde_cleared.expect("wiped out");
    assert_eq!(world.score, 2 * bonus);
    assert!(bonus >= 10 * u64::try_from(count).unwrap());
    wait(&mut world, 30 * 2);
    assert_eq!(world.player.level, 2);
}

#[test]
fn enemies_far_apart_never_push_each_other() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(120.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(120.5, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(140.0, 50.0)));
    let pushes = world.separations();
    assert!(pushes[0].x < 0.0 && pushes[1].x > 0.0);
    assert_eq!(pushes[2], Vec2::ZERO);
}

#[test]
fn a_run_opens_on_the_dusk_and_stands_still_while_it_is_told() {
    let mut world = World::new(1);
    assert_eq!(
        world.announcement.map(|told| told.news),
        Some(News::Weather(Weather::Dusk))
    );
    wait(&mut world, 30 * 2);
    assert_eq!((world.ticks(), world.announcement), (0, None));
    wait(&mut world, 1);
    assert_eq!(world.ticks(), 1);
}

#[test]
fn a_new_weather_is_announced() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 30 - 1;
    wait(&mut world, 1);
    assert_eq!(
        world.announcement.map(|told| told.news),
        Some(News::Weather(Weather::Clear))
    );
}

#[test]
fn the_run_is_won_at_dawn_with_a_bonus() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.score = 1_000;
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    world.ticks = 30 * 630 - 1;
    wait(&mut world, 1);
    assert!(world.daybreak() && !world.is_over());
    // The daybreak: the enemies flee, then burn; then the run is won.
    wait(&mut world, 30 * 4);
    assert_eq!(world.enemies, []);
    assert!(world.survived() && world.is_over());
    assert_eq!((world.score, world.dawn_bonus), (1_500, Some(500)));
    let won_at = world.ticks();
    wait(&mut world, 1);
    assert_eq!(world.ticks(), won_at);
}

#[test]
fn the_debug_console_skips_to_the_next_weather() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.debug_next_weather();
    wait(&mut world, 1);
    assert_eq!(
        world.announcement.map(|told| told.news),
        Some(News::Weather(Weather::Clear))
    );
}

#[test]
fn illumination_gives_a_point_of_xp_a_second() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .player
        .build
        .take(Choice::Legendary(Legendary::Illumination));
    wait(&mut world, 30 * 2);
    assert_eq!(world.player.xp, 2);
}

#[test]
fn the_night_fallen_the_torch_waits_for_the_player_to_light_it_f_shown_till_then() {
    let mut world = World::with(1, Mode::Standard, false, Progress::everything());
    world.announcement = None;
    world.next_horde = u32::MAX;
    disarm(&mut world);
    assert!(!world.torch_hint(), "not at dusk");
    world.switch_torch();
    assert_eq!(world.torch_on(), None, "nothing to light at dusk");
    world.ticks = 30 * 26;
    assert_eq!(world.torch_on(), Some(false), "out as the night falls");
    assert!(world.torch_hint());
    world.switch_torch();
    assert!(!world.torch_hint());
    // Holding once it has flickered, a second and a half at most.
    wait(&mut world, 46);
    assert!(world.torch_lit());
    world.switch_torch();
    assert!(!world.torch_lit() && !world.torch_shines());
    assert!(!world.torch_hint(), "lit once this run: no more F");
}

#[test]
fn a_flickering_torch_lights_nothing_for_the_run() {
    let mut world = in_the_dark();
    // Lit again, flickering as long as it may.
    world.switch_torch();
    world.torch_steady_at = world.ticks() + 45;
    assert!(!world.torch_lit());
    world.player.build.take(Choice::Weapon(Weapon::Pistol));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(110.0, 50.0)));
    wait(&mut world, 30);
    assert_eq!(world.bullets, []);
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

#[test]
fn in_the_dark_a_far_enemy_does_not_find_the_player() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(120.0, 50.0)));
    wait(&mut world, 30);
    let apart = (world.enemies[0].position - world.player.position).length();
    assert!(apart > 18.5, "{apart}");
    assert!(!world.enemies[0].noticed);
}

#[test]
fn in_the_dark_a_near_enemy_finds_the_player() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(105.0, 50.0)));
    wait(&mut world, 30);
    assert!(world.enemies[0].noticed);
    assert!(world.enemies[0].position.x < 104.0);
}

#[test]
fn in_the_dark_the_guns_hold_their_fire() {
    let mut world = in_the_dark();
    world.player.build.take(Choice::Weapon(Weapon::Pistol));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(110.0, 50.0)));
    wait(&mut world, 30);
    assert_eq!(world.bullets, []);
    // Lit again: once it holds, after its flicker, the guns fire.
    world.switch_torch();
    wait(&mut world, 45 + 30);
    assert!(world.enemies[0].hp < EnemyKind::Abomination.stats().hp);
}

#[test]
fn shadows_hunt_in_the_dark_and_shun_the_torch() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(125.0, 50.0)));
    wait(&mut world, 1);
    assert!(world.enemies[0].noticed, "the torch out, it hunts");
    world.switch_torch();
    // Once it holds, after its flicker.
    wait(&mut world, 45);
    let at = world.enemies[0].position.x;
    wait(&mut world, 30);
    assert!(!world.enemies[0].noticed);
    assert!(world.enemies[0].position.x > at, "away from the light");
}

#[test]
fn a_shadow_hunting_in_the_dark_goes_faster_and_faster() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(160.0, 50.0)));
    wait(&mut world, 1);
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let first = at - world.enemies[0].position.x;
    wait(&mut world, 150);
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let later = at - world.enemies[0].position.x;
    assert!(
        (later / first - 1.5).abs() < 0.05,
        "{later} against {first}"
    );
}

#[test]
fn a_shadow_caught_in_the_light_bolts_out_of_it() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(110.0, 50.0)));
    wait(&mut world, 1);
    world.switch_torch();
    // The light holds after its flicker: the Shadow, lit, bolts.
    while !world.torch_lit() {
        wait(&mut world, 1);
    }
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let bolted = world.enemies[0].position.x - at;
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let fled = world.enemies[0].position.x - at;
    assert!(bolted > fled * 2.0, "{bolted} against {fled}");
}

#[test]
fn a_felled_abomination_swells_then_bursts_into_parasites_worth_no_gem() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.unhurt = true;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(110.0, 50.0)));
    world.enemies[0].hp = 0;
    wait(&mut world, 1);
    world.gems.clear();
    assert_eq!(world.enemies, []);
    assert_eq!(world.swellings.len(), 1, "it swells first");
    wait(&mut world, burst::SWELL_TICKS);
    assert_eq!(world.swellings, []);
    assert_eq!(world.blasts.len(), 1);
    assert_eq!(world.enemies.len(), 4);
    for parasite in &world.enemies {
        assert_eq!(parasite.kind, EnemyKind::Parasite);
        let out = (parasite.position - Vec2::new(110.0, 50.0)).length();
        assert!(out > 2.0, "thrown out of it: {out}");
    }
    for parasite in &mut world.enemies {
        parasite.hp = 0;
    }
    wait(&mut world, 1);
    assert_eq!(world.enemies, []);
    assert_eq!(world.gems, []);
}

#[test]
fn a_burst_hurts_and_pushes_back_all_around() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(103.5, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    world.enemies[0].hp = 0;
    // The zombies stand still, for the player to be hurt by the burst alone.
    for zombie in &mut world.enemies[1..] {
        zombie.stunned = 100;
    }
    let (near, far) = (world.enemies[1].id, world.enemies[2].id);
    let hp = world.player.hp;
    // Its gem taken away, for no level to stop the world.
    wait(&mut world, 1);
    world.gems.clear();
    wait(&mut world, burst::SWELL_TICKS);
    assert_eq!(world.player.hp, hp - burst::DAMAGE);
    assert!(world.player.position.x < 99.0, "pushed back");
    let zombie = |id| world.enemies.iter().find(|enemy| enemy.id == id).unwrap();
    let full = EnemyKind::Zombie.stats().hp;
    assert_eq!(zombie(near).hp, full - burst::DAMAGE);
    assert!(zombie(near).position.x > 104.5, "pushed back");
    assert_eq!(zombie(far).hp, full, "out of its reach");
}

#[test]
fn in_the_dark_a_parasite_finds_the_player_from_afar() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Parasite, Vec2::new(125.0, 50.0)));
    wait(&mut world, 1);
    assert!(world.enemies[0].noticed);
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

#[test]
fn glare_slows_the_enemies_in_the_torch_light() {
    let free = zombie_step(0);
    let glared = zombie_step(2);
    // Two levels: 20% slower.
    assert!(
        (glared / free - 0.8).abs() < 0.02,
        "{glared} against {free}"
    );
}

#[test]
fn glare_reaches_as_far_as_the_light_shows_while_it_is_lit() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.ticks = 30 * 40;
    world.torch_steady_at = 0;
    assert_eq!(world.glare(), None, "not without Glare");
    world.player.build.take(Choice::Passive(Passive::Glare));
    // Its 9 tiles of light, and the 6 it fades over.
    assert_eq!(world.glare(), Some((15.0, 10)));
    world.switch_torch();
    assert_eq!(world.glare(), None, "the torch out");
}

#[test]
fn the_torch_stays_lit_and_lights_again_at_will() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.next_strike_in = Some(u32::MAX);
    world.ticks = 30 * 40;
    world.unhurt = true;
    // Five minutes lit, and still on.
    wait(&mut world, 30 * 300);
    assert_eq!(world.torch_on(), Some(true));
    // Out, then on again at once.
    world.switch_torch();
    assert_eq!(world.torch_on(), Some(false));
    world.switch_torch();
    assert_eq!(world.torch_on(), Some(true));
}

#[test]
fn with_the_card_a_dash_goes_the_way_the_player_faces_nothing_hurting_them() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.player.build.take(Choice::Action(Action::Dash));
    world.update(moving(1.0, 0.0));
    let start = world.player.position.x;
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, world.player.position));
    world.enemies[0].stunned = u32::MAX;
    world.dash();
    for _ in 0..8 {
        world.update(Input::default());
    }
    assert_eq!(world.player.hp, BASE_HP);
    assert!((world.player.position.x - start - 3.0).abs() < 1e-3);
    assert!(world.player.dash.is_none());
}

#[test]
fn loot_makes_gems_likelier() {
    let mut build = Build::new();
    assert_eq!(build.drop_bonus(), 0);
    build.take(Choice::Passive(Passive::Loot));
    build.take(Choice::Passive(Passive::Loot));
    assert_eq!(build.drop_bonus(), 20);
}

#[test]
fn skipping_the_dusk_opens_on_the_night_the_torch_lit() {
    let world = World::with(1, Mode::Standard, true, Progress::everything());
    assert_eq!(world.weather(), Weather::Clear);
    assert_eq!(
        world.announcement.map(|told| told.news),
        Some(News::Weather(Weather::Clear))
    );
    assert!(world.torch_lit());
}

#[test]
fn in_arcade_there_is_no_dawn_and_the_pressure_grows() {
    let mut world = World::with(1, Mode::Arcade, false, Progress::everything());
    world.announcement = None;
    world.next_horde = u32::MAX;
    world.player.build = Build::unarmed();
    world.unhurt = true;
    world.ticks = 30 * 630;
    wait(&mut world, 30 * 5);
    assert!(!world.daybreak() && !world.is_over());
    world.ticks = 30 * 600;
    assert!((world.pressure() - 1.07_f32.powi(10)).abs() < 0.01);
    // The multiplier follows the pressure: ×2 at ten minutes.
    assert_eq!(world.multiplier_tenths(), 20);
}

#[test]
fn a_new_weather_told_quietly_lets_the_run_go_on() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.weather_news = WeatherNews::Banner;
    world.ticks = 30 * 30 - 1;
    wait(&mut world, 1);
    assert_eq!(world.announcement, None);
    assert_eq!(world.told, Some((News::Weather(Weather::Clear), 30 * 30)));
}

#[test]
fn a_horde_told_without_a_pause_lets_the_run_go_on() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.weather_news = WeatherNews::Big;
    world.set_view(Vec2::new(30.0, 12.0));
    world.ticks = 30 * 87 - 1;
    wait(&mut world, 1);
    assert_eq!(world.announcement, None);
    assert_eq!(
        world.told,
        Some((News::Horde { everywhere: false }, 30 * 87))
    );
}

#[test]
fn a_campfire_heals_within_its_light() {
    let mut map = Map::empty();
    map.put_feature((110, 50), Feature::Campfire);
    let mut world = on(map, Vec2::new(105.5, 50.5));
    disarm(&mut world);
    world.player.hp = 50;
    wait(&mut world, 30);
    // Five tiles off, within its six of light: healed.
    assert!(world.player.hp > 50);
    assert!(world.healed_at.is_some());
}

#[test]
fn regeneration_gives_hp_back_of_itself() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.player.hp = 50;
    for _ in 0..5 {
        world
            .player
            .build
            .take(Choice::Passive(Passive::Regeneration));
    }
    // 2 HP a second at level 5.
    wait(&mut world, 30 * 3);
    assert_eq!(world.player.hp, 56);
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

#[test]
fn with_life_drain_some_enemies_felled_give_a_hp_back() {
    assert_eq!(drained(0, 200), 0, "none without it");
    let first = drained(1, 200);
    assert!((5..=40).contains(&first), "about 10%: {first}");
    let last = drained(5, 200);
    assert!((35..=90).contains(&last), "about 30%: {last}");
}

#[test]
fn an_enemy_burnt_to_death_leaves_ash() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    world.enemies[0].hp = 2;
    world.enemies[0].ignite();
    wait(&mut world, 30);
    assert_eq!(world.enemies, []);
    assert!(world.stains.iter().any(|stain| stain.ash));
}

#[test]
fn a_molotov_bursts_into_a_puddle_that_sets_enemies_on_fire() {
    let mut world = armed_with(Weapon::Molotov, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(104.0, 50.0)));
    world.enemies[0].hp = 1_000;
    wait(&mut world, 1);
    assert_eq!(world.bottles.len(), 1);
    wait(&mut world, 12);
    assert_eq!(world.puddles.len(), 1);
    assert!(world.enemies[0].burning > 0);
}

#[test]
fn in_the_dark_no_molotov_is_thrown() {
    let mut world = in_the_dark();
    world.player.build.take(Choice::Weapon(Weapon::Molotov));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(104.0, 50.0)));
    wait(&mut world, 30);
    assert_eq!(world.bottles, []);
}

#[test]
fn a_melee_weapon_waits_between_two_swings_and_tells_how_long() {
    let mut world = armed_with(Weapon::Crowbar, 1);
    assert_eq!(world.waits()[0].1, 0, "ready");
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world.enemies[0].hp = 1_000;
    wait(&mut world, 1);
    let (weapon, left, whole) = world.waits()[0];
    assert_eq!(weapon, Weapon::Crowbar);
    assert_eq!(left, whole, "just swung: all of it to wait");
    wait(&mut world, 1);
    assert_eq!(world.waits()[0].1, whole - 1);
}

#[test]
fn the_crowbar_pushes_back_what_it_hits() {
    let mut world = armed_with(Weapon::Crowbar, 1);
    world.player.facing = Facing::Right;
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world.enemies[0].hp = 1_000;
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 1_000 - 18);
    assert!(world.enemies[0].position.x > 102.5);
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

#[test]
fn a_barricade_set_down_ahead_stops_a_zombie_which_batters_it_until_it_breaks() {
    let mut world = barricading(1);
    world.act();
    assert_eq!(world.barricades.len(), 1);
    let left = world.barricades[0].corner.0;
    assert_eq!(world.barricades[0].corner, (102, 49));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 51.0)));
    wait(&mut world, 30 * 3);
    let zombie = world.enemies[0].position.x;
    assert!(
        zombie > f32::from(u8::try_from(left).unwrap()) + 2.0,
        "stopped at {zombie}"
    );
    assert_eq!(world.player.hp, BASE_HP);
    assert!(world.barricades[0].hp < 80, "battered");
    // 80 HP, 5 a second: broken in 16 seconds, the ground free again.
    wait(&mut world, 30 * 16);
    assert_eq!(world.barricades, Vec::new());
    assert!(!world.map.blocks((102, 50)));
}

#[test]
fn a_shadow_flies_over_a_barricade_and_the_shots_go_over_it() {
    let mut world = barricading(1);
    world.act();
    let corner = world.barricades[0].corner;
    assert!(super::super::body::in_sight(
        &world.map,
        world.player.position,
        Vec2::new(110.0, 51.0)
    ));
    assert!(world.map.blocks(corner));
    assert!(!world.map.stops_shots(corner));
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(104.0, 51.0)));
    wait(&mut world, 30);
    assert!(world.enemies[0].position.x < 104.0, "went on");
    assert_eq!(world.barricades[0].hp, 80, "flew over, never touching it");
}

#[test]
fn barricades_come_back_in_their_time_the_oldest_taken_away_past_the_most() {
    let mut world = barricading(1);
    world.act();
    world.act();
    assert_eq!(world.barricades.len(), 1, "the next not ready yet");
    let first = world.barricades[0].corner;
    for _ in 0..2 {
        world.player.position.y -= 5.0;
        wait(&mut world, 30 * 8);
        world.act();
    }
    assert_eq!(world.barricades.len(), 2);
    assert!(
        world
            .barricades
            .iter()
            .all(|barricade| barricade.corner != first)
    );
    assert!(!world.map.blocks(first));
}

#[test]
fn a_barricade_is_not_set_down_on_a_rock() {
    let mut world = by_a_rock(Vec2::new(19.0, 10.5));
    disarm(&mut world);
    world.player.facing = Facing::Right;
    world.player.build.take(Choice::Action(Action::Barricade));
    world.act();
    assert_eq!(world.barricades, Vec::new());
    assert_eq!(world.barricade_wait, 0, "still ready");
    assert!(world.barricade_refused.is_some(), "shown refused");
    wait(&mut world, barricade::REFUSED_TICKS);
    assert_eq!(world.barricade_refused, None);
}

#[test]
fn an_abomination_s_burst_blows_a_barricade_away() {
    let mut world = barricading(1);
    world.act();
    let centre = world.barricades[0].centre();
    world.burst(centre + Vec2::new(1.5, 0.0));
    assert_eq!(world.barricades, Vec::new());
}

#[test]
fn at_level_5_the_barricade_is_of_metal_neither_battered_nor_blown_away() {
    let mut world = barricading(5);
    world.act();
    assert!(world.barricades[0].metal);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(107.0, 51.0)));
    world.enemies[0].hp = 1_000;
    wait(&mut world, 30 * 5);
    assert_eq!(world.barricades[0].hp, 180, "battered for nothing");
    let centre = world.barricades[0].centre();
    world.burst(centre + Vec2::new(1.5, 0.0));
    assert_eq!(world.barricades.len(), 1, "still standing");
}

#[test]
fn two_minutes_untouched_after_nightfall_unlock_the_barricade() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.weather_news = WeatherNews::Banner;
    // Untouched all through the dusk: nothing yet.
    wait(&mut world, 30 * 120);
    assert!(!world.progress.achieved.contains(&Achievement::Untouchable));
    world.hurt_at = Some(world.ticks);
    wait(&mut world, 30 * 120 - 1);
    assert!(!world.progress.achieved.contains(&Achievement::Untouchable));
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::Untouchable));
    assert!(world.progress.deals(Choice::Action(Action::Barricade)));
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

#[test]
fn the_flamethrower_burns_the_enemies_ahead_and_sets_them_on_fire_not_those_behind() {
    let mut world = flaming(1);
    wait(&mut world, 1);
    assert!(world.flaming);
    assert_eq!(world.enemies[0].hp, 1_000 - 2);
    assert!(world.enemies[0].burning > 0);
    assert_eq!((world.enemies[3].hp, world.enemies[3].burning), (1_000, 0));
    // Again a few ticks on.
    wait(&mut world, flamethrower::BURN_EVERY);
    assert_eq!(world.enemies[0].hp, 1_000 - 4);
}

#[test]
fn the_flamethrower_s_tank_empties_in_three_seconds_and_fills_again_in_two() {
    let mut world = flaming(1);
    wait(&mut world, 30 * 3);
    let tank = world.magazines[&Weapon::Flamethrower];
    assert_eq!(tank.rounds, 0);
    assert!(tank.reloading > 0);
    wait(&mut world, 1);
    assert!(!world.flaming, "empty");
    wait(&mut world, 30 * 2);
    assert!(world.flaming, "full again");
}

#[test]
fn the_flamethrower_burns_nothing_behind_a_rock() {
    let mut world = by_a_rock(Vec2::new(18.5, 11.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    for _ in 0..2 {
        world
            .player
            .build
            .take(Choice::Weapon(Weapon::Flamethrower));
    }
    world.player.facing = Facing::Right;
    for y in [10.6, 11.0, 11.4] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(22.4, y)));
    }
    for enemy in &mut world.enemies {
        enemy.stunned = u32::MAX;
    }
    let hp = world.enemies[0].hp;
    wait(&mut world, 1);
    assert!(!world.flaming);
    assert_eq!(world.enemies[0].hp, hp);
}

#[test]
fn lit_by_one_enemy_the_flamethrower_burns_its_whole_tank() {
    let mut world = flaming(1);
    world.enemies.truncate(1);
    wait(&mut world, 1);
    assert!(world.flaming, "lit by one");
    world.enemies.clear();
    wait(&mut world, 30 * 3 - 2);
    assert!(world.flaming, "on, the tank not yet empty");
    wait(&mut world, 1);
    assert_eq!(world.magazines[&Weapon::Flamethrower].rounds, 0);
    wait(&mut world, 1);
    assert!(!world.flaming, "out, filling again");
}

#[test]
fn the_flamethrower_lights_at_the_nearest_enemy_whichever_way_the_player_faces() {
    let mut world = flaming(1);
    world.enemies.drain(..3);
    wait(&mut world, 1);
    assert!(world.flaming, "lit at the one behind");
    assert_eq!(world.enemies[0].hp, 1_000 - 2);
    let (_, way, _) = world.flame_cone().unwrap();
    assert!(way.x < -0.99, "{way:?}");
}

#[test]
fn burning_the_flamethrower_turns_after_the_nearest_enemy_no_faster_than_it_may() {
    let mut world = flaming(1);
    world.enemies.truncate(1);
    wait(&mut world, 1);
    // Now the only one stands behind the player.
    world.enemies[0].position = Vec2::new(98.5, 50.5);
    wait(&mut world, 1);
    let (_, way, _) = world.flame_cone().unwrap();
    let turned = way.y.atan2(way.x).abs().to_degrees();
    assert!(
        (turned - flamethrower::TURN_DEGREES).abs() < 0.01,
        "{turned}"
    );
    wait(&mut world, 30);
    let (_, way, _) = world.flame_cone().unwrap();
    assert!(way.x < -0.99, "round at last: {way:?}");
}

#[test]
fn the_flamethrower_burns_in_the_dark_too() {
    let mut world = flaming(1);
    world.ticks = 30 * 40;
    world.torch_switch = false;
    assert_eq!(world.torch_on(), Some(false));
    wait(&mut world, 1);
    assert!(world.flaming);
}

#[test]
fn at_level_5_the_flamethrower_s_tank_leaks_a_trail_of_fires() {
    let mut world = flaming(5);
    world.enemies.clear();
    for _ in 0..15 {
        world.spawn_debt = 0.0;
        world.update(moving(1.0, 0.0));
    }
    assert!(world.puddles.len() >= 3, "{} fires", world.puddles.len());
    assert!(world.puddles.iter().all(|puddle| !puddle.burst));
    assert!(world.puddles.iter().all(|puddle| puddle.centre.x > 100.0));
}

#[test]
fn every_horde_of_a_run_wiped_out_unlocks_the_flamethrower() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    world.next_horde = weather::dawn() - 1;
    wipe_out_a_horde(&mut world);
    assert!(
        !world.progress.deals(Choice::Weapon(Weapon::Flamethrower)),
        "one still to come"
    );
    // The last of the run, none after it before the dawn.
    world.next_horde = weather::dawn() + 1;
    wipe_out_a_horde(&mut world);
    assert!(world.progress.deals(Choice::Weapon(Weapon::Flamethrower)));
}

#[test]
fn level_25_in_a_run_unlocks_illumination() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    world.player.level = 24;
    wait(&mut world, 1);
    assert!(!world.progress.achieved.contains(&Achievement::Veteran));
    world.player.level = 25;
    wait(&mut world, 1);
    assert!(
        world
            .progress
            .deals(Choice::Legendary(Legendary::Illumination))
    );
}

#[test]
fn a_strike_with_the_player_in_its_ring_puts_their_torch_out_up_to_them_to_light_it_again() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.ticks = 30 * 40;
    world.torch_steady_at = 0;
    assert!(world.torch_lit());
    world.strike(world.player.position + Vec2::new(3.0, 0.0), false);
    assert!(world.torch_lit(), "out of its ring");
    world.strike(world.player.position + Vec2::new(2.0, 0.0), false);
    assert_eq!(world.torch_on(), Some(false), "out");
    wait(&mut world, 30 * 5);
    assert_eq!(world.torch_on(), Some(false), "not back of itself");
    world.switch_torch();
    assert_eq!(world.torch_on(), Some(true));
}

#[test]
fn the_hammer_lands_ahead_hurting_and_throwing_back_every_enemy_around_not_those_away() {
    let mut world = armed_with(Weapon::Hammer, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // Where it lands, 2.5 tiles ahead; within its 2 tiles; well away.
    for x in [102.5, 104.0, 110.0] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    // Raised a moment over the player's head, then down.
    wait(&mut world, 1);
    assert!(world.hammer_raised > 0 && world.blow.is_none());
    assert_eq!(world.enemies[0].hp, 1_000);
    wait(&mut world, hammer::RAISE_TICKS);
    assert!(world.blow.is_some());
    assert_eq!(world.enemies[0].hp, 1_000 - 28);
    assert_eq!(world.enemies[1].hp, 1_000 - 28);
    // Thrown out as the wave goes, a tile and a half.
    wait(&mut world, hammer::WAVE_TICKS);
    assert!(world.enemies[1].position.x > 105.0, "thrown back");
    assert_eq!(world.enemies[2].hp, 1_000);
    // Two seconds after the blow before it is raised again.
    let hp = world.enemies[0].hp;
    wait(&mut world, 30 * 2 - 1 - hammer::WAVE_TICKS);
    assert_eq!(world.enemies[0].hp, hp);
    assert_eq!(world.hammer_raised, 0);
}

#[test]
fn ten_abominations_felled_in_melee_in_a_run_unlock_the_demolisher() {
    let mut world = armed_with(Weapon::Crowbar, 1);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.player.facing = Facing::Right;
    for felled in 0..10 {
        assert!(
            !world.progress.has(Unlock::Skin(Skin::Demolisher)),
            "{felled}"
        );
        world.melee.clear();
        world
            .enemies
            .push(enemy(EnemyKind::Abomination, Vec2::new(101.5, 50.0)));
        let last = world.enemies.len() - 1;
        world.enemies[last].hp = 1;
        world.enemies[last].stunned = u32::MAX;
        wait(&mut world, 1);
        world.enemies.clear();
        world.swellings.clear();
    }
    assert!(world.progress.has(Unlock::Skin(Skin::Demolisher)));
}

#[test]
fn a_hero_starts_with_their_own_weapon_dealt_to_them_alone() {
    let mut world = World::with(1, Mode::Standard, false, Progress::everything());
    world.play_as(Skin::Classic);
    assert_eq!(world.player.build.weapon_level(Weapon::Machete), 1);
    assert_eq!(world.player.build.weapon_level(Weapon::Crowbar), 0);
    let dealt: Vec<Choice> = (0..60).flat_map(|_| world.deal_as(25)).collect();
    assert!(dealt.contains(&Choice::Weapon(Weapon::Machete)));
    for others in [Weapon::Crowbar, Weapon::Hammer, Weapon::Baton] {
        assert!(!dealt.contains(&Choice::Weapon(others)), "{others:?}");
    }
}

#[test]
fn the_baton_thrusts_straight_ahead_hurting_and_paralysing_the_enemies_on_its_line() {
    let mut world = armed_with(Weapon::Baton, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // On its line, near and at its reach; aside; behind.
    for (x, y) in [(101.5, 50.0), (102.8, 50.2), (101.5, 52.0), (98.5, 50.0)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
    }
    wait(&mut world, 1);
    assert!(world.thrust.is_some());
    let hit: Vec<bool> = world.enemies.iter().map(|e| e.hp < 1_000).collect();
    assert_eq!(hit, [true, true, false, false]);
    assert!(world.enemies[0].stunned > 0, "paralysed");
    assert!(world.enemies[0].shocked > 0, "shown shocked");
    assert_eq!(world.enemies[2].stunned, 0);
}

#[test]
fn the_knife_stabs_the_nearest_enemy_whichever_way_quick_and_often() {
    let mut world = armed_with(Weapon::Knife, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // Behind the player, the nearest; a little further; out of reach.
    for (x, y) in [(99.2, 50.0), (101.4, 50.0), (104.0, 50.0)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1);
    let hp: Vec<u32> = world.enemies.iter().map(|e| e.hp).collect();
    assert_eq!(hp, [1_000 - 8, 1_000, 1_000]);
    assert_eq!(world.stabs.len(), 1);
    // Again 0.4 s on.
    wait(&mut world, 12);
    assert_eq!(world.enemies[0].hp, 1_000 - 16);
}

#[test]
fn a_minute_under_30_percent_of_hp_in_a_run_unlocks_the_special_agent_and_the_chainsaw() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.weather_news = WeatherNews::Banner;
    world.player.hp = 29;
    wait(&mut world, 30 * 60 - 1);
    assert!(!world.progress.achieved.contains(&Achievement::ColdBlood));
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::ColdBlood));
    assert_eq!(Skin::Agent.achievement(), Some(Achievement::ColdBlood));
    assert!(world.progress.deals(Choice::Weapon(Weapon::Chainsaw)));
}

#[test]
fn each_hero_plays_with_their_own_hp_speed_and_perk() {
    let build = |hero| Build::playing(hero);
    assert_eq!(build(Skin::Survivor).max_hp(), BASE_HP);
    assert_eq!(build(Skin::Demolisher).max_hp(), 140);
    assert_eq!(build(Skin::Agent).max_hp(), 80);
    assert_eq!(build(Skin::Survivor).speed_percent(), 100);
    assert_eq!(build(Skin::Demolisher).speed_percent(), 85);
    assert_eq!(build(Skin::Agent).speed_percent(), 110);
    assert_eq!(build(Skin::Classic).damage(20), 22, "commando");
    assert_eq!(build(Skin::Survivor).damage(20), 20);
    assert_eq!(build(Skin::Agent).reload(50), 40, "reflexes");
    let burns = build(Skin::Survivor).burn_ticks();
    assert_eq!(
        build(Skin::Exterminator).burn_ticks(),
        burns * 2,
        "pyromaniac"
    );
}

#[test]
fn the_demolisher_takes_less_damage_and_no_burst_throws_them() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    world.play_as(Skin::Demolisher);
    assert_eq!(world.player.hp, 140);
    let start = world.player.position;
    world.burst(start + Vec2::new(1.0, 0.0));
    assert_eq!(world.player.hp, 140 - burst::DAMAGE * 90 / 100);
    assert_eq!(world.player.position, start, "not thrown");
}

#[test]
fn the_katana_cuts_the_nearest_then_chains_through_the_next_ones_near() {
    let mut world = armed_with(Weapon::Katana, 1);
    world.next_horde = u32::MAX;
    // The nearest, then each within a leap of the last; one too far from the chain.
    for (x, y) in [
        (101.2, 50.0),
        (103.0, 50.0),
        (105.0, 50.5),
        (106.8, 51.0),
        (115.0, 50.0),
    ] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1);
    let taken: Vec<u32> = world.enemies.iter().map(|e| 1_000 - e.hp).collect();
    assert_eq!(
        taken,
        [14, 10, 7, 0, 0],
        "three at the first level, each less"
    );
    assert_eq!(world.cuts.as_ref().map(|cuts| cuts.path.len()), Some(4));
}

#[test]
fn shadow_hunter_unlocks_the_shurikens_and_the_ninja() {
    let mut progress = Progress::default();
    assert!(!progress.deals(Choice::Weapon(Weapon::Shuriken)));
    progress.reach(Achievement::ShadowHunter);
    assert!(progress.deals(Choice::Weapon(Weapon::Shuriken)));
    assert_eq!(Skin::Ninja.achievement(), Some(Achievement::ShadowHunter));
}

#[test]
fn the_ninja_dodges_a_blow_now_and_then_taking_nothing() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    world.play_as(Skin::Ninja);
    let (mut dodged, mut taken) = (0, 0);
    for _ in 0..200 {
        let hp = world.player.hp;
        world.player.invincible = 0;
        world.wound_player(1);
        if world.player.hp == hp {
            dodged += 1;
        } else {
            taken += 1;
            world.player.hp = hp;
        }
    }
    // About one in seven dodged.
    assert!((10..60).contains(&dodged), "{dodged} dodged, {taken} taken");
}

#[test]
fn seven_hundred_enemies_felled_in_a_run_unlock_the_classic_hero() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.kills = 699;
    wait(&mut world, 1);
    assert!(!world.progress.has(Unlock::Skin(Skin::Classic)));
    world.kills = 700;
    wait(&mut world, 1);
    assert!(world.progress.has(Unlock::Skin(Skin::Classic)));
}

#[test]
fn the_defibrillator_leaps_through_the_enemies_near_and_heals_for_each_it_fells() {
    let mut world = armed_with(Weapon::Defibrillator, 1);
    world.next_horde = u32::MAX;
    world.player.hp = 50;
    for (x, hp) in [(101.5, 1), (103.2, 1000), (104.9, 1), (110.0, 1)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
        let last = world.enemies.len() - 1;
        world.enemies[last].hp = hp;
        world.enemies[last].stunned = u32::MAX;
    }
    wait(&mut world, 1);
    assert_eq!(world.shock.as_ref().map(|shock| shock.path.len()), Some(4));
    // The two felled are gone; the one between stands, shocked.
    let between = world
        .enemies
        .iter()
        .find(|e| (e.position.x - 103.2).abs() < 0.1);
    assert_eq!(between.map(|e| e.hp), Some(1_000 - 12));
    assert_eq!(world.player.hp, 52, "two felled, a point each");
}

#[test]
fn the_medic_heals_half_again_as_much_and_gets_hp_back_of_themselves() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    world.play_as(Skin::Medic);
    world.next_horde = u32::MAX;
    world.player.hp = 40;
    world.map.put_feature((100, 51), Feature::Herb);
    wait(&mut world, 1);
    assert_eq!(world.player.hp, 40 + HERB_HP * 3 / 2, "a herb, half again");
    let hp = world.player.hp;
    wait(&mut world, 30 * 5);
    assert_eq!(world.player.hp, hp + 1, "a point in 5 s");
}

#[test]
fn three_hundred_hp_got_back_in_a_run_unlock_the_medic_and_life_drain() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    for _ in 0..3 {
        world.player.hp = 1;
        world.heal(99);
    }
    wait(&mut world, 1);
    assert!(
        !world.progress.achieved.contains(&Achievement::FirstAid),
        "297"
    );
    assert!(!world.progress.deals(Choice::Passive(Passive::Drain)));
    world.player.hp = 1;
    world.heal(3);
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::FirstAid));
    assert!(world.progress.deals(Choice::Passive(Passive::Drain)));
}

#[test]
fn the_dawn_reached_with_a_hero_unlocks_their_second_colours() {
    let mut world = World::with(1, Mode::Standard, false, Progress::default());
    world.play_as(Skin::Medic);
    world.announcement = None;
    world.ticks = weather::dawn() + weather::daybreak() - 1;
    world.update(Input::default());
    assert!(world.survived());
    assert!(world.progress.wears(Skin::Medic, 1));
    assert!(!world.progress.wears(Skin::Survivor, 1));
}

#[test]
fn every_hero_plays_a_minute_of_a_run_taking_every_card_without_failing() {
    for (seed, hero) in (1_u64..).zip(Skin::ALL) {
        let mut world = World::with(seed, Mode::Standard, false, Progress::everything());
        world.play_as(hero);
        world.weather_news = WeatherNews::Banner;
        world.announcement = None;
        world.unhurt = true;
        let ways = [
            moving(1.0, 0.0),
            moving(0.0, 1.0),
            moving(-1.0, 0.0),
            moving(0.0, -1.0),
        ];
        for tick in 0..30 * 60 {
            if world.offer.is_some() {
                world.choose(tick % 3);
                continue;
            }
            world.update(ways[(tick / 45) % 4]);
            if tick % 200 == 0 {
                world.act();
            }
        }
        assert!(world.seconds() >= 50, "{hero:?} played on");
    }
}

#[test]
fn the_chainsaw_grinds_what_touches_it_ahead_a_bite_every_few_ticks_slowing_it() {
    let mut world = armed_with(Weapon::Chainsaw, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    for x in [101.2, 98.8] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1);
    assert!(world.sawing);
    assert_eq!(world.enemies[0].hp, 1_000 - 4, "a bite at once");
    assert!(world.enemies[0].slowed > 0, "slowed");
    assert_eq!(world.enemies[1].hp, 1_000, "behind, untouched");
    wait(&mut world, 6);
    assert_eq!(world.enemies[0].hp, 1_000 - 8, "the next bite, 0.2 s on");
}
