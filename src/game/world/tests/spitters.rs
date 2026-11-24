//! The spitters: keeping off the player, their globs of acid, their puddles.

use super::super::super::spitter::{DAMAGE, FLY_TICKS, WIND_TICKS};
use super::*;

/// A spitter at `x` on the player's row, having found them, no weapon about.
fn facing_a_spitter(x: f32) -> World {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    let mut spitter = enemy(EnemyKind::Spitter, Vec2::new(x, 50.0));
    spitter.noticed = true;
    world.enemies.push(spitter);
    world
}

#[test]
fn a_spitter_comes_on_to_six_tiles_stands_there_and_backs_off_nearer() {
    for (x, way) in [(110.0, -1.0), (106.0, 0.0), (103.0, 1.0)] {
        let mut world = facing_a_spitter(x);
        wait(&mut world, 10);
        let moved = world.enemies[0].position.x - x;
        assert!(
            moved * way > 0.0 || (way == 0.0 && moved.abs() < 0.01),
            "{x}: {moved}"
        );
        assert_eq!(world.enemies[0].facing, Facing::Left, "it faces the player");
    }
}

#[test]
fn a_spitters_glob_lands_where_the_player_stood_a_second_on_and_leaves_acid() {
    let mut world = facing_a_spitter(106.0);
    world.enemies[0].spit_wait = 1;
    let hp = world.player.hp;
    wait(&mut world, 1);
    assert_eq!(world.globs.len(), 1);
    assert_eq!(world.globs[0].to, Vec2::new(100.0, 50.0));
    wait(&mut world, FLY_TICKS - 1);
    assert_eq!(world.player.hp, hp, "not yet");
    wait(&mut world, 1);
    assert_eq!(world.globs, Vec::new());
    assert_eq!(world.player.hp, hp - DAMAGE);
    let acid: Vec<_> = world.puddles.iter().filter(|puddle| puddle.acid).collect();
    assert_eq!(acid.len(), 1);
    assert!(acid[0].hostile, "it burns the player, not the enemies");
}

#[test]
fn the_player_stepping_aside_dodges_a_glob() {
    let mut world = facing_a_spitter(106.0);
    world.enemies[0].spit_wait = 1;
    let hp = world.player.hp;
    wait(&mut world, 1);
    world.player.position = Vec2::new(100.0, 47.0);
    wait(&mut world, FLY_TICKS);
    assert_eq!(world.player.hp, hp);
}

#[test]
fn a_spitter_out_of_reach_holds_its_spit() {
    let mut world = facing_a_spitter(120.0);
    world.enemies[0].spit_wait = 1;
    wait(&mut world, 1);
    assert_eq!(world.globs, Vec::new());
    assert!(world.enemies[0].spit_wait > WIND_TICKS);
}
