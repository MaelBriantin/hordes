//! The melee weapons: the crowbar, the machete, the knife, the katana, the baton, the truncheon, the fists, the chainsaw, the defibrillator.

use super::*;

#[test]
fn a_run_starts_with_the_crowbar() {
    let world = World::new(1);
    assert_eq!(world.player.build.weapon_level(Weapon::Crowbar), 1);
    assert_eq!(world.ammo(), []);
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
    wait(&mut world, machete::cooldown() - 14);
    assert_eq!(
        (world.enemies[0].hp, world.enemies[1].hp),
        (40 - 24, 40 - 12)
    );
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
fn from_its_third_level_every_third_strike_of_the_baton_is_a_whirlwind_hitting_all_around() {
    let mut world = armed_with(Weapon::Baton, 3);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // On its line; behind, within the whirlwind; aside, out of it.
    for (x, y) in [(101.5, 50.0), (98.5, 50.0), (100.0, 53.5)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    let every = world.player.build.cooldown(baton::at(3).every);
    wait(&mut world, 1 + every);
    assert_eq!(world.enemies[1].hp, 1_000, "two thrusts ahead");
    assert!(!world.thrust.is_some_and(|thrust| thrust.whirl));
    wait(&mut world, every);
    assert!(world.thrust.is_some_and(|thrust| thrust.whirl), "the third");
    assert_eq!(
        world.enemies[1].hp,
        1_000 - 22,
        "behind, hit by the whirlwind"
    );
    assert_eq!(world.enemies[2].hp, 1_000, "out of it");
    // The player shows spinning round, from the way they face.
    assert_eq!(world.hero_facing(), Facing::Right);
    wait(&mut world, 3);
    assert_eq!(world.hero_facing(), Facing::Up);
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

#[test]
fn the_chainsaw_s_tank_empties_only_as_it_grinds_and_fills_again_in_three_seconds() {
    let mut world = armed_with(Weapon::Chainsaw, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    wait(&mut world, 30);
    let full = world.ammo()[0].2;
    assert_eq!(
        world.ammo()[0].1.rounds,
        full,
        "nothing to grind: still full"
    );
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(101.2, 50.0)));
    world.enemies[0].hp = 100_000;
    world.enemies[0].stunned = u32::MAX;
    wait(&mut world, 30 * 5);
    let (_, tank, _, _) = world.ammo()[0];
    assert_eq!(tank.rounds, 0);
    assert!(tank.reloading > 0, "filling again");
    // Empty, it bites no more, nor slows, until full again.
    let hp = world.enemies[0].hp;
    wait(&mut world, 30 * 3 - 1);
    assert!(!world.sawing);
    assert_eq!(world.enemies[0].hp, hp);
    wait(&mut world, 1);
    assert_eq!(world.ammo()[0].1.rounds, full, "full again");
    wait(&mut world, 1);
    assert!(world.sawing);
    assert!(world.enemies[0].hp < hp);
}

#[test]
fn nothing_hurts_the_player_while_their_chainsaw_grinds_its_tank_empty_it_does_again() {
    let mut world = armed_with(Weapon::Chainsaw, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // One ahead, ground; one touching the player from behind.
    for x in [101.2, 99.4] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 100_000;
        enemy.stunned = u32::MAX;
    }
    let hp = world.player.hp;
    wait(&mut world, 30 * 5 - 1);
    assert!(world.sawing);
    assert_eq!(world.player.hp, hp, "charging, unhurt");
    wait(&mut world, 2);
    assert!(!world.sawing, "its tank empty");
    assert!(world.player.hp < hp, "hurt again");
}

#[test]
fn the_fists_punch_the_nearest_enemy_every_third_punch_heavier_throwing_it_back() {
    let mut world = armed_with(Weapon::Fists, 1);
    world.next_horde = u32::MAX;
    // Within arm's reach, behind the player; further, out of reach.
    for x in [99.0, 102.0] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 1_000 - 8, "a punch, whichever way");
    assert_eq!(world.enemies[1].hp, 1_000, "out of reach");
    let every = world.player.build.cooldown(fists::at(1).every);
    wait(&mut world, every);
    assert_eq!(world.enemies[0].hp, 1_000 - 16);
    assert!(!world.punch.is_some_and(|punch| punch.heavy));
    wait(&mut world, every);
    assert_eq!(world.enemies[0].hp, 1_000 - 32, "the third, twice as hard");
    assert!(world.punch.is_some_and(|punch| punch.heavy));
    wait(&mut world, fists::KNOCK_TICKS);
    assert!(world.enemies[0].position.x < 98.5, "thrown back");
}

#[test]
fn the_truncheon_strikes_ahead_throwing_back_what_it_hits() {
    let mut world = armed_with(Weapon::Truncheon, 1);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // Ahead, within its reach; behind.
    for x in [101.5, 98.5] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 1_000 - 14, "ahead");
    assert_eq!(world.enemies[1].hp, 1_000, "not behind");
    wait(&mut world, 10);
    assert!(world.enemies[0].position.x > 102.0, "thrown back");
}

#[test]
fn an_abomination_felled_by_the_fists_the_baton_or_the_ram_counts_for_demolisher() {
    for weapon in [Weapon::Fists, Weapon::Baton] {
        let mut world = armed_with(weapon, 1);
        world.next_horde = u32::MAX;
        world.player.facing = Facing::Right;
        world
            .enemies
            .push(enemy(EnemyKind::Abomination, Vec2::new(101.0, 50.0)));
        world.enemies[0].hp = 1;
        world.enemies[0].stunned = u32::MAX;
        wait(&mut world, 1);
        assert_eq!(world.abominations_smashed, 1, "{weapon:?}");
    }
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.announcement = None;
    world.player.facing = Facing::Right;
    world.player.build.take(Choice::Action(Action::Ram));
    world
        .enemies
        .push(enemy(EnemyKind::Abomination, Vec2::new(102.0, 50.0)));
    world.enemies[0].hp = 1;
    world.enemies[0].stunned = u32::MAX;
    world.act();
    wait(&mut world, ram::WINDUP_TICKS + 8);
    assert_eq!(world.abominations_smashed, 1, "the ram");
}

#[test]
fn the_knife_strikes_the_living_not_an_enemy_already_felled_this_tick() {
    let mut world = armed_with(Weapon::Knife, 1);
    world.next_horde = u32::MAX;
    // Felled, nearer; standing, a little further.
    for x in [100.8, 101.4] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    world.enemies[0].hp = 0;
    world.enemies[1].hp = 1_000;
    world.enemies[1].stunned = u32::MAX;
    wait(&mut world, 1);
    assert!(
        world
            .enemies
            .iter()
            .any(|enemy| enemy.hp < 1_000 && enemy.hp > 0),
        "the living one struck"
    );
}
