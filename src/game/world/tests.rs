use super::super::Obstacle;
use super::super::announce::{News, WeatherNews};
use super::super::build::{BASE_HP, Build, Choice, Legendary, Passive, Weapon};
use super::super::gem::Gem;
use super::super::mode::Mode;
use super::super::progress::{Achievement, Progress};
use std::sync::atomic::{AtomicU32, Ordering};

use super::super::drones;
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
fn a_bullet_fells_a_zombie() {
    let mut world = armed_with(Weapon::Pistol, 1);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(105.0, 50.0)));
    wait(&mut world, 15);
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
fn a_drone_hits_an_enemy_on_its_circle_and_pushes_it_back() {
    let mut world = armed_with(Weapon::Drones, 1);
    let next = drones::positions(world.player.position, 1, drones::turn_per_tick(1))[0];
    world.enemies.push(enemy(EnemyKind::Abomination, next));
    let before = (world.enemies[0].position - world.player.position).length();
    wait(&mut world, 1);
    let enemy = &world.enemies[0];
    assert_eq!(enemy.hp, 40 - 8);
    let after = (enemy.position - world.player.position).length();
    assert!(after > before, "{before} then {after}");
}

#[test]
fn a_drone_breaks_on_its_blow_and_repairs_in_two_seconds() {
    let mut world = armed_with(Weapon::Drones, 1);
    let next = drones::positions(world.player.position, 1, drones::turn_per_tick(1))[0];
    world.enemies.push(enemy(EnemyKind::Abomination, next));
    world.enemies[0].hp = 1_000;
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 1_000 - 8);
    assert!(world.drones()[0].1, "broken");
    // Broken, it passes through the enemy, harmless.
    for _ in 0..60 {
        world.enemies[0].position = world.drones()[0].0;
        wait(&mut world, 1);
    }
    assert_eq!(world.enemies[0].hp, 1_000 - 8);
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
fn a_storms_lightning_hurts_the_enemies_it_falls_among() {
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
    // Half a second on, it falls: the player, still in the ring, is hurt.
    wait(&mut world, 1);
    assert!(world.lightning.is_some());
    assert_eq!(world.strikes_coming, []);
    assert_eq!(world.player.hp, BASE_HP - weather::STRIKE_PLAYER_DAMAGE);
}

#[test]
fn a_strike_spares_the_player_out_of_its_ring() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 480;
    world.strike(Vec2::new(103.0, 50.0), true);
    assert_eq!(world.player.hp, BASE_HP);
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
    assert_eq!(enemy.hp, enemy.kind.stats().hp * 150 / 100);
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
fn the_torch_lights_up_of_itself_as_the_night_falls_and_goes_out_at_will() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    assert!(!world.torch_lit());
    world.switch_torch();
    assert_eq!(world.torch_on(), None);
    // It lights up at 0:26, holding once it has flickered, a second and a half at most.
    world.ticks = 30 * 26 - 1;
    wait(&mut world, 1);
    wait(&mut world, 45);
    assert!(world.torch_lit());
    world.switch_torch();
    assert!(!world.torch_lit() && !world.torch_shines());
    world.switch_torch();
    wait(&mut world, 46);
    assert!(world.torch_lit());
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
    world.player.build.take(Choice::Passive(Passive::Dash));
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
