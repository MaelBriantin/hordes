use super::super::Obstacle;
use super::super::build::{BASE_HP, Choice, Passive, Weapon};
use super::super::gem::Gem;
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

/// Bolt never shooting: to watch enemies without them falling.
fn disarm(world: &mut World) {
    world.bolt_cooldown = u32::MAX;
}

/// One enemy at `position`, its phase 0.
fn enemy(kind: EnemyKind, position: Vec2) -> Enemy {
    Enemy::new(kind, position, 0.0)
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
fn bolt_shoots_at_the_nearest_enemy_in_range() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(110.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(100.0, 44.0)));
    wait(&mut world, 1);
    assert_eq!(world.sparks.len(), 1);
    let spark = world.sparks[0].position;
    assert!(
        spark.y < 50.0 && (spark.x - 100.0).abs() < 1e-3,
        "{spark:?}"
    );
}

#[test]
fn bolt_holds_fire_with_no_enemy_in_range() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(120.0, 50.0)));
    wait(&mut world, 1);
    assert_eq!(world.sparks.len(), 0);
}

#[test]
fn bolt_shoots_every_two_seconds() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(110.0, 50.0)));
    world.enemies[0].hp = 1_000;
    let mut shots = 0;
    for _ in 0..150 {
        let before = world.sparks.len();
        wait(&mut world, 1);
        if world.sparks.len() > before {
            shots += 1;
        }
    }
    assert_eq!(shots, 3);
}

#[test]
fn a_spark_fells_a_zombie_which_leaves_a_gem() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(105.0, 50.0)));
    wait(&mut world, 15);
    assert_eq!(world.enemies.len(), 0);
    assert_eq!(world.kills, 1);
    assert_eq!(world.sparks.len(), 0);
    assert_eq!(world.gems.len(), 1);
    assert_eq!(world.gems[0].value, 1);
}

#[test]
fn an_abomination_takes_four_sparks() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(108.0, 50.0)));
    wait(&mut world, 15);
    assert_eq!(world.enemies[0].hp, 30);
    assert!(world.enemies[0].flash > 0);
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
fn bolt_at_level_two_shoots_the_two_nearest() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.player.build.take(Choice::Weapon(Weapon::Bolt));
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
    assert_eq!(world.sparks.len(), 2);
    assert!(world.sparks.iter().any(|spark| spark.position.x > 100.0));
    assert!(world.sparks.iter().any(|spark| spark.position.x < 100.0));
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
