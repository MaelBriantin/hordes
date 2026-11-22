//! The weapons that strike from afar: the guns, the shurikens, the drones.

use super::*;

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
fn nothing_breaks_a_drone_its_shots_come_from_a_magazine_reloaded_once_empty() {
    let mut world = armed_with(Weapon::Drones, 1);
    // Right where the drone comes: it shoots on, unharmed.
    let next = drones::positions(world.player.position, 1, drones::turn_per_tick(1))[0];
    world.enemies.push(enemy(EnemyKind::Abomination, next));
    world.enemies[0].hp = 100_000;
    world.enemies[0].stunned = u32::MAX;
    let drone = |world: &World| world.ammo()[0];
    let (weapon, full, _, reload) = {
        let (weapon, magazine, full, reload) = drone(&world);
        (weapon, magazine, full, reload)
    };
    assert_eq!(weapon, Weapon::Drones);
    assert_eq!(full.rounds, 6, "six shots for one drone");
    // A shot every 0.4 s: six, then the reload.
    wait(&mut world, 12 * 5 + 1);
    let (_, magazine, _, _) = drone(&world);
    assert_eq!(magazine.rounds, 0);
    assert!(magazine.reloading > 0, "reloading");
    assert_eq!(reload, 90, "in three seconds");
    // The last shot landed, none while reloading.
    wait(&mut world, 10);
    let hp = world.enemies[0].hp;
    wait(&mut world, magazine.reloading - 11);
    assert_eq!(world.enemies[0].hp, hp, "none while reloading");
    wait(&mut world, 2);
    assert_eq!(drone(&world).1.reloading, 0, "reloaded");
    wait(&mut world, 12);
    assert!(drone(&world).1.rounds < 6, "shooting on");
}

#[test]
fn without_drones_there_are_no_drones() {
    let world = on(Map::empty(), Vec2::new(100.0, 50.0));
    assert_eq!(world.drones().len(), 0);
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
fn from_level_4_the_shurikens_ricochet_on_an_enemy_off_their_line() {
    for (level, ricochets) in [(3, false), (4, true)] {
        let mut world = armed_with(Weapon::Shuriken, level);
        world.next_horde = u32::MAX;
        // One within range, shot at; one beyond range, off its line, near it.
        for (x, y) in [(107.0, 50.0), (109.0, 52.5)] {
            world
                .enemies
                .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
        }
        for enemy in &mut world.enemies {
            enemy.hp = 100_000;
            enemy.stunned = u32::MAX;
        }
        wait(&mut world, 30);
        assert!(world.enemies[0].hp < 100_000, "level {level}: shot at");
        assert_eq!(
            world.enemies[1].hp < 100_000,
            ricochets,
            "level {level}: ricocheted on"
        );
    }
}

#[test]
fn haste_makes_the_guns_fire_sooner_and_the_barricade_come_back_sooner() {
    let fire_wait = |haste: u8| {
        let mut world = armed_with(Weapon::Pistol, 1);
        world.next_horde = u32::MAX;
        for _ in 0..haste {
            world.player.build.take(Choice::Passive(Passive::Haste));
        }
        world
            .enemies
            .push(enemy(EnemyKind::Abomination, Vec2::new(104.0, 50.0)));
        world.enemies[0].hp = 100_000;
        world.enemies[0].stunned = u32::MAX;
        wait(&mut world, 1);
        world.magazines[&Weapon::Pistol].wait
    };
    assert!(fire_wait(5) < fire_wait(0), "faster fire");
    let barricade_wait = |haste: u8| {
        let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
        disarm(&mut world);
        world.announcement = None;
        world.player.build.take(Choice::Action(Action::Barricade));
        for _ in 0..haste {
            world.player.build.take(Choice::Passive(Passive::Haste));
        }
        world.act();
        world.barricade_wait
    };
    assert!(barricade_wait(5) < barricade_wait(0), "sooner back");
}
