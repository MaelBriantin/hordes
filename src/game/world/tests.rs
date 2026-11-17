use super::super::Obstacle;
use super::super::build::{BASE_HP, Build, Choice, Passive, Weapon};
use super::super::gem::Gem;
use std::sync::atomic::{AtomicU32, Ordering};

use super::super::drones;
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
    let mut world = World::new(1);
    let start = world.player.position;
    world.update(Input::default());
    assert_eq!(world.player.position, start);
}

#[test]
fn a_second_of_moving_covers_the_players_speed() {
    let mut world = World::new(1);
    let start = world.player.position;
    for _ in 0..30 {
        world.update(moving(1.0, 0.0));
    }
    assert_near(world.player.position.x - start.x, PLAYER_SPEED);
    assert_near(world.player.position.y, start.y);
}

#[test]
fn a_diagonal_is_no_faster_than_a_straight_line() {
    let mut world = World::new(1);
    let start = world.player.position;
    world.update(moving(1.0, -1.0));
    let moved = world.player.position - start;
    assert_near(moved.length(), PLAYER_SPEED * tick_seconds());
    assert!(moved.x > 0.0 && moved.y < 0.0);
}

#[test]
fn the_player_faces_where_they_go() {
    let mut world = World::new(1);
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
    let mut world = World::new(9);
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
fn a_rock_stops_a_zombie_but_not_a_bat() {
    let mut world = by_a_rock(Vec2::new(17.0, 11.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(24.0, 11.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Bat, Vec2::new(24.0, 11.0)));
    wait(&mut world, 45);
    let (zombie, bat) = (&world.enemies[0], &world.enemies[1]);
    assert!(zombie.position.x >= 22.0 + 0.45 - 1e-3, "{zombie:?}");
    assert!(bat.position.x < 20.0, "{bat:?}");
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
fn a_run_starts_with_the_machete() {
    let world = World::new(1);
    assert_eq!(world.player.build.weapon_level(Weapon::Machete), 1);
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
    let mut shots = 0;
    for _ in 0..90 {
        let before = world.bullets.len();
        world.enemies[0].position = Vec2::new(109.0, 50.0);
        wait(&mut world, 1);
        if world.bullets.len() > before {
            shots += 1;
        }
    }
    // A shot every 3 ticks: the 20 rounds in 60 ticks, then a 3-second reload.
    assert_eq!(shots, 20);
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
fn a_drone_hits_the_same_enemy_only_every_half_second() {
    let mut world = armed_with(Weapon::Drones, 3);
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.5, 50.0)));
    world.enemies[0].hp = 1_000;
    for _ in 0..60 {
        // Kept on the circle, where the drones pass again and again.
        world.enemies[0].position = Vec2::new(102.5, 50.0);
        wait(&mut world, 1);
    }
    let hits = (1_000 - world.enemies[0].hp) / 12;
    assert!((2..=4).contains(&hits), "{hits} hits");
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
fn from_the_drizzle_lightning_strikes() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 120;
    wait(&mut world, 30 * 13);
    assert!(world.lightning.is_some());
}

#[test]
fn a_storms_lightning_hurts_the_enemies_it_falls_among() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 360;
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
fn enemies_appearing_later_are_tougher() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 480;
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
    world.ticks = 30 * 360;
    assert!((clear - world.light_radius() - 2.0).abs() < 1e-4);
}
