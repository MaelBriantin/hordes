//! A run as it goes: moving, the enemies and the hordes, levels, cards, the score.

use super::*;

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
fn two_levels_at_once_make_level_2_s_offer_and_bring_level_3_s_action() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    world.gems.push(Gem::new(Vec2::new(100.2, 50.0), 13));
    wait(&mut world, 1);
    assert_eq!(world.player.level, 3);
    assert!(world.unlocked.is_some(), "level 3: the hero's action");
    world.seen_action();
    assert!(world.offer.is_some(), "level 2: its cards");
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
    assert_eq!(
        world.enemies[1].hp,
        1_000 - super::super::super::percent_of(18, 120)
    );
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
fn loot_makes_gems_likelier() {
    let mut build = Build::new();
    assert_eq!(build.drop_bonus(), 0);
    build.take(Choice::Passive(Passive::Loot));
    build.take(Choice::Passive(Passive::Loot));
    assert_eq!(build.drop_bonus(), 20);
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
fn the_debug_console_gives_a_card_s_next_level_within_the_room_the_action_and_1_hp() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    let revival = Choice::Passive(Passive::Revival);
    assert!(world.debug_cards().contains(&revival), "locked cards too");
    assert!(world.debug_give(revival));
    assert!(world.debug_give(revival));
    assert_eq!(world.player.build.passive_level(Passive::Revival), 2);
    // Four weapons at most: the hero's own and three more.
    let more: Vec<Choice> = world
        .debug_cards()
        .into_iter()
        .filter(|choice| match choice {
            Choice::Weapon(weapon) => world.player.build.weapon_level(*weapon) == 0,
            _ => false,
        })
        .collect();
    let given = more
        .iter()
        .filter(|choice| world.debug_give(**choice))
        .count();
    assert_eq!(given, 3, "room for three more");
    // The hero's own action at once, then its next level.
    world.debug_action_up();
    world.debug_action_up();
    let action = world.player.build.hero().action();
    assert_eq!(world.player.build.action_level(action), 2);
    world.debug_one_hp();
    assert_eq!(world.player.hp, 1);
}
