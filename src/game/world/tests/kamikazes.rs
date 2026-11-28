//! The kamikazes: their fuse, their blast, their fire.

use super::super::super::kamikaze::{DAMAGE, FIRE_DAMAGE, FIRES, LAST_TICKS, PUSH};
use super::super::super::molotov::Puddle;
use super::*;

/// A kamikaze just short of the player, a zombie standing still behind it, no weapon about.
fn lit() -> World {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world
        .enemies
        .push(enemy(EnemyKind::Kamikaze, Vec2::new(100.95, 50.0)));
    let mut zombie = enemy(EnemyKind::Zombie, Vec2::new(103.0, 50.0));
    zombie.stunned = u32::MAX;
    world.enemies.push(zombie);
    world
}

#[test]
fn a_kamikaze_on_the_player_stands_still_a_last_moment_then_blows_on_everything_around() {
    let mut world = lit();
    let hp = world.player.hp;
    // A tick walking in, then on them: its last moment, without striking them.
    wait(&mut world, 1 + LAST_TICKS - 1);
    let kamikaze = &world.enemies[0];
    assert_eq!((kamikaze.kind, kamikaze.fuse), (EnemyKind::Kamikaze, 1));
    let standing = kamikaze.position;
    assert_eq!(world.player.hp, hp);
    wait(&mut world, 1);
    assert_eq!(world.enemies, Vec::new(), "the zombie felled by the blast");
    assert_eq!(world.player.hp, hp - DAMAGE);
    assert_eq!(world.kamikaze_blasts.len(), 1);
    assert_near(world.kamikaze_blasts[0].at.x, standing.x);
    assert_eq!(
        world.puddles.iter().filter(|puddle| puddle.hostile).count(),
        FIRES.len()
    );
    // Blown of itself, it is worth nothing; the zombie it took with it counts.
    assert_eq!(world.kills, 1);
}

#[test]
fn the_player_fleeing_at_once_gets_away_from_a_kamikaze_standing_its_last_moment() {
    let mut world = lit();
    let hp = world.player.hp;
    wait(&mut world, 2);
    assert!(world.enemies[0].fuse > 0 && world.enemies[0].fuse < LAST_TICKS);
    for _ in 0..LAST_TICKS {
        world.spawn_debt = 0.0;
        world.update(moving(-1.0, 0.0));
    }
    assert_eq!(world.kamikaze_blasts.len(), 1);
    assert_eq!(world.player.hp, hp);
}

#[test]
fn a_kamikaze_comes_at_its_own_speed_unlit_until_it_touches_the_player() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world
        .enemies
        .push(enemy(EnemyKind::Kamikaze, Vec2::new(103.0, 50.0)));
    wait(&mut world, 1);
    let x = world.enemies[0].position.x;
    wait(&mut world, 1);
    let speed = EnemyKind::Kamikaze.stats().speed;
    assert_near(x - world.enemies[0].position.x, speed / 30.0);
    assert_eq!(world.enemies[0].fuse, 0, "near, not on them");
    wait(&mut world, 30);
    assert!(world.enemies[0].fuse > 0, "on them, lit");
}

#[test]
fn a_kamikaze_thrown_off_the_player_its_fuse_lit_stands_still_then_blows() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    let mut kamikaze = enemy(EnemyKind::Kamikaze, Vec2::new(110.0, 50.0));
    kamikaze.noticed = true;
    kamikaze.fuse = LAST_TICKS + 1;
    world.enemies.push(kamikaze);
    let hp = world.player.hp;
    wait(&mut world, 1);
    let x = world.enemies[0].position.x;
    wait(&mut world, LAST_TICKS - 1);
    assert_near(world.enemies[0].position.x, x);
    wait(&mut world, 1);
    assert_eq!(world.enemies, Vec::new());
    assert_eq!(world.kamikaze_blasts.len(), 1);
    assert_eq!(world.player.hp, hp, "too far to be hurt");
}

#[test]
fn a_kamikaze_far_from_the_player_comes_on_its_fuse_unlit() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Kamikaze, Vec2::new(106.0, 50.0)));
    wait(&mut world, 10);
    assert_eq!(world.enemies[0].fuse, 0);
    assert!(world.enemies[0].position.x < 106.0);
}

#[test]
fn a_kamikaze_felled_blows_at_once_felling_another_that_blows_too() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world
        .enemies
        .push(enemy(EnemyKind::Kamikaze, Vec2::new(110.0, 50.0)));
    world
        .enemies
        .push(enemy(EnemyKind::Kamikaze, Vec2::new(112.0, 50.0)));
    world.enemies[0].hp = 0;
    wait(&mut world, 1);
    assert_eq!(world.enemies, Vec::new());
    assert_eq!(world.kamikaze_blasts.len(), 2);
    // Felled, not blown of themselves: both count.
    assert_eq!(world.kills, 2);
}

#[test]
fn a_kamikaze_throws_the_player_half_a_tile_and_the_demolisher_not_at_all() {
    for (skin, thrown) in [(Skin::Survivor, PUSH), (Skin::Demolisher, 0.0)] {
        let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
        world.player.build = Build::playing(skin);
        let mut kamikaze = enemy(EnemyKind::Kamikaze, Vec2::new(101.2, 50.0));
        kamikaze.hp = 0;
        world.enemies.push(kamikaze);
        wait(&mut world, 1);
        assert_near(world.player.position.x, 100.0 - thrown);
    }
}

#[test]
fn a_kamikazes_fire_burns_the_player_standing_in_it_not_the_enemies() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    let mut zombie = enemy(EnemyKind::Zombie, Vec2::new(105.0, 50.0));
    zombie.stunned = u32::MAX;
    world.enemies.push(zombie);
    for centre in [Vec2::new(100.0, 50.0), Vec2::new(105.0, 50.0)] {
        world.puddles.push(Puddle {
            centre,
            radius: 0.6,
            left: 90,
            lasts: 90,
            burst: false,
            wall: false,
            hostile: true,
            acid: false,
        });
    }
    let hp = world.player.hp;
    wait(&mut world, 1);
    assert_eq!(world.player.hp, hp - FIRE_DAMAGE);
    assert_eq!(world.enemies[0].burning, 0);
    // Not again until they can be hurt again: half a second on.
    wait(&mut world, 14);
    assert_eq!(world.player.hp, hp - FIRE_DAMAGE);
    wait(&mut world, 2);
    assert_eq!(world.player.hp, hp - 2 * FIRE_DAMAGE);
}
