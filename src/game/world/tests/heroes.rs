//! The heroes, their outfits, perks and actions: the dash, the barricade, the shield ram.

use super::*;

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
    assert!(super::super::super::body::in_sight(
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
fn the_torch_out_the_enemies_lose_the_infiltrator_at_once_but_those_right_on_them() {
    for (hero, lost) in [(Skin::Survivor, false), (Skin::Infiltrator, true)] {
        let mut world = in_the_dark();
        world.play_as(hero);
        world.announcement = None;
        // Further than the infiltrator is noticed, nearer than any other hero is lost.
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(104.0, 50.0)));
        world.enemies[0].noticed = true;
        world.enemies[0].stunned = u32::MAX;
        wait(&mut world, 1);
        assert_eq!(!world.enemies[0].noticed, lost, "{hero:?}");
    }
    let mut world = infiltrator_in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Zombie, Vec2::new(101.5, 50.0)));
    world.enemies[0].stunned = u32::MAX;
    world.enemies[0].hp = 100_000;
    wait(&mut world, 1);
    assert!(world.enemies[0].noticed, "right on them");
}

#[test]
fn the_shadows_hunt_the_infiltrator_only_after_four_seconds_of_dark() {
    let mut world = infiltrator_in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(125.0, 50.0)));
    wait(&mut world, 30 * 4 - 2);
    assert!(!world.enemies[0].noticed, "not yet");
    wait(&mut world, 3);
    assert!(world.enemies[0].noticed, "now they hunt");
}

#[test]
fn lightning_blanks_the_infiltrator_s_goggles_a_while() {
    let mut world = infiltrator_in_the_dark();
    wait(&mut world, 1);
    assert!(world.night_vision());
    world.cut_torch();
    assert!(!world.night_vision(), "blank");
    wait(&mut world, 30 * 4);
    assert!(world.night_vision(), "back");
    let mut world = in_the_dark();
    assert!(!world.night_vision(), "only the infiltrator");
    world.switch_torch();
    world.play_as(Skin::Infiltrator);
    assert!(!world.night_vision(), "not with the torch on");
}

#[test]
fn the_night_fallen_f_is_shown_to_every_hero_but_the_infiltrator() {
    for (hero, hint) in [(Skin::Survivor, true), (Skin::Infiltrator, false)] {
        let mut world = World::with(1, Mode::Standard, false, Progress::everything());
        world.play_as(hero);
        world.ticks = 30 * 26;
        assert_eq!(world.torch_on(), Some(false));
        assert_eq!(world.torch_hint(), hint, "{hero:?}");
    }
}

#[test]
fn the_riot_officer_s_shield_takes_a_share_of_the_blows_from_ahead_not_from_behind() {
    let hurt = |x: f32| {
        let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
        world.play_as(Skin::Riot);
        world.player.build = Build::playing(Skin::Riot);
        world.next_horde = u32::MAX;
        world.announcement = None;
        // The truncheon held back: the enemy stays touching.
        world.melee.insert(
            Weapon::Truncheon,
            Swinging {
                wait: u32::MAX,
                done: 0,
                left: 0,
                gap: 0,
            },
        );
        world.player.facing = Facing::Right;
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(x, 50.0)));
        world.enemies[0].stunned = u32::MAX;
        world.enemies[0].hp = 100_000;
        let hp = world.player.hp;
        wait(&mut world, 1);
        hp - world.player.hp
    };
    // Touching the player, not overlapping them.
    let (ahead, behind) = (hurt(100.88), hurt(99.12));
    assert!(behind > 0);
    assert_eq!(ahead, behind * 70 / 100, "30% less from ahead");
}

#[test]
fn the_shield_ram_charges_ahead_unhurt_parting_the_enemies_in_a_v() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.announcement = None;
    world.player.facing = Facing::Right;
    world.player.build.take(Choice::Action(Action::Ram));
    // Right in the way of the charge, a little to its left and to its right.
    for y in [49.7, 50.3] {
        world
            .enemies
            .push(enemy(EnemyKind::Zombie, Vec2::new(101.5, y)));
    }
    for enemy in &mut world.enemies {
        enemy.hp = 1_000;
        // Held where they stand while the player braces.
        enemy.stunned = 12;
    }
    let hp = world.player.hp;
    world.act();
    // Braced a moment, standing still whatever the keys say, then off.
    for _ in 0..ram::WINDUP_TICKS - 1 {
        world.update(moving(0.0, 1.0));
    }
    assert!(world.bracing());
    assert_eq!(
        world.player.position,
        Vec2::new(100.0, 50.0),
        "braced, still"
    );
    wait(&mut world, 1 + 8);
    assert!(world.player.position.x > 102.5, "charged ahead");
    assert_eq!(world.player.hp, hp, "unhurt");
    assert!(world.enemies.iter().all(|enemy| enemy.hp == 1_000 - 20));
    wait(&mut world, ram_throw_ticks());
    assert!(
        world.enemies[0].position.y < 47.0,
        "the left one far to the left"
    );
    assert!(
        world.enemies[1].position.y > 53.0,
        "the right one far to the right"
    );
    // Not again before its recharge.
    let at = world.player.position;
    world.act();
    wait(&mut world, 1);
    assert!((world.player.position.x - at.x).abs() < 0.5, "not ready");
}
