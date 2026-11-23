//! What burns and blows up: the Molotov, the flamethrower, the hammer, the C4, the grenade, the napalm, the abominations' bursts.

use super::*;

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
fn the_flamethrower_s_tank_empties_in_two_seconds_and_fills_again_in_three() {
    let mut world = flaming(1);
    wait(&mut world, 30 * 2);
    let tank = world.magazines[&Weapon::Flamethrower];
    assert_eq!(tank.rounds, 0);
    assert!(tank.reloading > 0);
    wait(&mut world, 1);
    assert!(!world.flaming, "empty");
    wait(&mut world, 30 * 3);
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
    wait(&mut world, 30 * 2 - 2);
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
fn at_level_5_the_flamethrower_s_tank_no_longer_leaks() {
    let mut world = flaming(5);
    world.enemies.clear();
    for _ in 0..45 {
        world.spawn_debt = 0.0;
        world.update(moving(1.0, 0.0));
    }
    assert!(
        world.puddles.is_empty(),
        "the wall of fire is the exterminator's action now"
    );
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
fn from_level_3_shocks_follow_the_hammer_s_blow_on_ahead_one_after_the_other_each_enemy_hit_once() {
    let mut world = armed_with(Weapon::Hammer, 3);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // Where it lands, 2.5 tiles ahead; where its first shock does, 2 tiles on; its second; aside.
    for (x, y) in [(102.5, 50.0), (104.5, 50.0), (106.5, 50.0), (104.5, 53.0)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1 + hammer::RAISE_TICKS);
    assert_eq!(world.enemies[0].hp, 1_000 - 40, "the blow");
    assert_eq!(world.enemies[2].hp, 1_000, "not yet");
    wait(&mut world, hammer::SHOCK_EVERY * 2);
    assert_eq!(world.enemies[2].hp, 1_000 - 40, "the second shock");
    assert_eq!(world.enemies[0].hp, 1_000 - 40, "hit once");
    assert_eq!(world.enemies[3].hp, 1_000, "aside");
    assert!(!world.quake_waves.is_empty(), "their waves show");
}

#[test]
fn below_level_3_no_shock_follows_the_hammer_s_blow() {
    let mut world = armed_with(Weapon::Hammer, 2);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    for x in [102.5, 106.5] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(
        &mut world,
        1 + hammer::RAISE_TICKS + hammer::SHOCK_EVERY * 3,
    );
    assert_eq!(world.enemies[0].hp, 1_000 - 40, "the blow");
    assert_eq!(world.enemies[1].hp, 1_000, "no shock");
}

#[test]
fn at_level_5_the_hammer_s_shocks_go_left_and_right_too() {
    let mut world = armed_with(Weapon::Hammer, 5);
    world.next_horde = u32::MAX;
    world.player.facing = Facing::Right;
    // Where it lands; 4 tiles to its left and its right, beyond the blow, where its second
    // shocks land.
    for (x, y) in [(102.5, 50.0), (102.5, 46.0), (102.5, 54.0)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1 + hammer::RAISE_TICKS);
    assert_eq!(world.enemies[1].hp, 1_000, "beyond the blow");
    wait(&mut world, hammer::SHOCK_EVERY * 2);
    assert!(world.enemies.iter().all(|enemy| enemy.hp < 1_000));
}

#[test]
fn the_napalm_drops_a_line_of_bombs_on_the_crowd_setting_its_strip_on_fire() {
    let mut world = napalm_on_a_crowd(1);
    wait(&mut world, 1);
    assert_eq!(world.bombs.len(), 5, "a bomb a tile, 5 tiles");
    // The last one lands once the others have, a bomb every few ticks.
    wait(&mut world, napalm::FALL_TICKS + 4 * napalm::BOMB_EVERY);
    assert!(world.bombs.is_empty(), "all landed");
    let crowd = &world.enemies[..3];
    assert!(
        crowd
            .iter()
            .all(|enemy| enemy.hp < 1_000 && enemy.burning > 0)
    );
    assert_eq!(world.enemies[3].hp, 1_000, "off its strip");
    assert_eq!(world.enemies[4].hp, 1_000, "out of reach");
    assert_eq!(world.puddles.len(), 5, "its strip burning");
    wait(&mut world, napalm::fire_lasts());
    assert!(world.puddles.is_empty(), "burnt out");
}

#[test]
fn the_napalm_strikes_nothing_with_no_enemy_within_reach() {
    let mut world = napalm_on_a_crowd(1);
    world.enemies.clear();
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(130.0, 50.0)));
    wait(&mut world, 30);
    assert!(world.bombs.is_empty(), "none within reach");
}

#[test]
fn at_level_5_the_napalm_drops_two_lines_crossed() {
    let mut world = napalm_on_a_crowd(5);
    wait(&mut world, 1);
    assert_eq!(world.bombs.len(), 14, "two lines of 7");
}

#[test]
fn space_sets_a_c4_charge_down_and_space_again_blows_it_from_afar() {
    let mut world = charged(1);
    world.act();
    assert_eq!(world.charges.len(), 1);
    assert_eq!(
        world.charges[0].at,
        Vec2::new(100.0, 50.0),
        "at the player's feet"
    );
    // The player walks off; it waits.
    world.player.position = Vec2::new(98.0, 50.0);
    wait(&mut world, 20);
    assert_eq!(world.enemies[0].hp, 1_000, "waiting");
    world.act();
    wait(&mut world, 1);
    assert!(
        world.charges.is_empty() && world.c4_blasts.len() == 1,
        "blown"
    );
    assert_eq!(world.enemies[0].hp, 1_000 - 25);
    assert_eq!(world.enemies[1].hp, 1_000 - 25);
    assert_eq!(world.enemies[2].hp, 1_000, "beyond its blast");
    wait(&mut world, c4::WAVE_TICKS);
    assert!(world.enemies[1].position.x > 104.5, "thrown back 3 tiles");
}

#[test]
fn a_c4_charge_left_alone_blows_by_itself_then_the_next_waits_its_recharge() {
    let mut world = charged(1);
    world.act();
    wait(&mut world, c4::FUSE_TICKS - 1);
    assert_eq!(world.enemies[0].hp, 1_000, "not yet");
    wait(&mut world, 1);
    assert_eq!(world.enemies[0].hp, 1_000 - 25, "by itself");
    // Not again before its recharge.
    world.act();
    assert!(world.charges.is_empty(), "recharging");
    wait(&mut world, c4::at(1).recharge);
    world.act();
    assert_eq!(world.charges.len(), 1, "ready again");
}

#[test]
fn at_level_5_two_c4_charges_set_down_one_after_the_other_blow_together() {
    let mut world = charged(5);
    world.act();
    world.player.position = Vec2::new(108.0, 50.0);
    world.act();
    assert_eq!(world.charges.len(), 2);
    world.act();
    wait(&mut world, 1);
    assert_eq!(world.c4_blasts.len(), 2, "together");
    assert!(
        world.enemies.iter().all(|enemy| enemy.hp < 1_000),
        "around either"
    );
}

#[test]
fn the_demolisher_s_c4_comes_at_the_run_s_level_3_its_card_shown_the_run_standing_still() {
    assert_eq!(Skin::Demolisher.action(), Action::C4);
    let mut world = World::with(1, Mode::Standard, false, Progress::everything());
    world.play_as(Skin::Demolisher);
    world.announcement = None;
    world.player.level = 2;
    world.update(Input::default());
    assert_eq!(
        world.player.build.action_level(Action::C4),
        0,
        "not before level 3"
    );
    let dealt: Vec<Choice> = (0..80).flat_map(|_| world.deal_as(25)).collect();
    assert!(
        !dealt.contains(&Choice::Action(Action::C4)),
        "not dealt before it comes"
    );
    world.player.level = 3;
    world.update(Input::default());
    assert_eq!(world.player.build.action_level(Action::C4), 1, "at level 3");
    assert_eq!(world.unlocked, Some(Action::C4), "its card shown");
    // The run stands still until the player has seen it.
    let ticks = world.ticks;
    world.update(Input::default());
    assert_eq!(world.ticks, ticks, "standing still");
    world.seen_action();
    world.update(Input::default());
    assert!(world.ticks > ticks, "going on");
    let dealt: Vec<Choice> = (0..80).flat_map(|_| world.deal_as(25)).collect();
    assert!(
        dealt.contains(&Choice::Action(Action::C4)),
        "its next levels to them"
    );
    world.play_as(Skin::Agent);
    world.player.level = 3;
    world.update(Input::default());
    assert_eq!(
        world.player.build.action_level(Action::C4),
        0,
        "not another hero's"
    );
    assert_eq!(
        world.unlocked,
        Some(Action::Vault),
        "the agent's own instead"
    );
}

#[test]
fn the_grenade_is_thrown_at_the_crowd_and_blows_where_it_lands() {
    let mut world = armed_with(Weapon::Grenade, 1);
    world.next_horde = u32::MAX;
    // A crowd 5 tiles off; one alone, well away from it.
    for (x, y) in [(105.0, 50.0), (105.5, 50.0), (105.0, 50.5), (100.0, 55.0)] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 1);
    assert_eq!(world.grenades.len(), 1, "thrown");
    wait(&mut world, grenade::FLIGHT_TICKS);
    assert!(world.grenades.is_empty(), "blown");
    assert!(
        world.enemies[..3]
            .iter()
            .all(|enemy| enemy.hp == 1_000 - 20)
    );
    assert_eq!(world.enemies[3].hp, 1_000, "away from it");
}
