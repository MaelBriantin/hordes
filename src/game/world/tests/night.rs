//! The night: the torch, the dark, the Shadows, lightning, the weather, the campfires.

use super::*;

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
    // Half a second on, it falls: the player, still in the ring, unhurt, their torch out.
    wait(&mut world, 1);
    assert!(world.lightning.is_some());
    assert_eq!(world.strikes_coming, []);
    assert_eq!(world.player.hp, BASE_HP);
    assert_eq!(world.torch_on(), Some(false));
}

#[test]
fn a_strike_never_hurts_the_player() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.ticks = 30 * 480;
    world.strike(world.player.position, true);
    assert_eq!(world.player.hp, BASE_HP);
}

#[test]
fn the_weather_dims_the_light() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    let clear = world.light_radius();
    world.ticks = 30 * 480;
    assert!((clear - world.light_radius() - 2.0).abs() < 1e-4);
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
fn from_the_storm_on_a_horde_comes_from_every_side() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.ticks = 30 * 480;
    world.next_horde = 30 * 482;
    wait(&mut world, 1);
    assert_eq!(world.horde_warning.map(|warning| warning.side), Some(None));
}

#[test]
fn at_its_last_level_horde_hunter_veils_the_whole_horde_and_shows_it_in_the_dark() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.set_view(Vec2::new(30.0, 12.0));
    for _ in 0..5 {
        world
            .player
            .build
            .take(Choice::Passive(Passive::HordeHunter));
    }
    world.bring_horde(Warning {
        arrives_at: 0,
        side: Some(Facing::Right),
        number: 0,
    });
    assert!(world.player.build.horde_veil_in_dark());
    assert_eq!(world.horde_veil(&world.enemies[0]), Some(Veil::Steady));
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
fn the_night_fallen_the_torch_waits_for_the_player_to_light_it_f_shown_till_then() {
    let mut world = World::with(1, Mode::Standard, false, Progress::everything());
    world.announcement = None;
    world.next_horde = u32::MAX;
    disarm(&mut world);
    assert!(!world.torch_hint(), "not at dusk");
    world.switch_torch();
    assert_eq!(world.torch_on(), None, "nothing to light at dusk");
    world.ticks = 30 * 26;
    assert_eq!(world.torch_on(), Some(false), "out as the night falls");
    assert!(world.torch_hint());
    world.switch_torch();
    assert!(!world.torch_hint());
    // Holding once it has flickered, a second and a half at most.
    wait(&mut world, 46);
    assert!(world.torch_lit());
    world.switch_torch();
    assert!(!world.torch_lit() && !world.torch_shines());
    assert!(!world.torch_hint(), "lit once this run: no more F");
}

#[test]
fn the_first_time_lightning_puts_the_torch_out_f_is_shown_again_till_it_is_lit() {
    let mut world = in_the_dark();
    world.switch_torch();
    wait(&mut world, 46);
    assert!(world.torch_lit() && !world.torch_hint());
    world.cut_torch();
    assert!(world.torch_hint(), "put out by lightning: F again");
    world.switch_torch();
    assert!(!world.torch_hint());
    wait(&mut world, 46);
    world.cut_torch();
    assert!(!world.torch_hint(), "the second time, no more");
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

#[test]
fn a_shadow_hunting_in_the_dark_goes_faster_and_faster() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(160.0, 50.0)));
    wait(&mut world, 1);
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let first = at - world.enemies[0].position.x;
    wait(&mut world, 150);
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let later = at - world.enemies[0].position.x;
    assert!(
        (later / first - 1.5).abs() < 0.05,
        "{later} against {first}"
    );
}

#[test]
fn a_shadow_caught_in_the_light_bolts_out_of_it() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Shadow, Vec2::new(110.0, 50.0)));
    wait(&mut world, 1);
    world.switch_torch();
    // The light holds after its flicker: the Shadow, lit, bolts.
    while !world.torch_lit() {
        wait(&mut world, 1);
    }
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let bolted = world.enemies[0].position.x - at;
    let at = world.enemies[0].position.x;
    wait(&mut world, 10);
    let fled = world.enemies[0].position.x - at;
    assert!(bolted > fled * 2.0, "{bolted} against {fled}");
}

#[test]
fn in_the_dark_a_parasite_finds_the_player_from_afar() {
    let mut world = in_the_dark();
    world
        .enemies
        .push(enemy(EnemyKind::Parasite, Vec2::new(125.0, 50.0)));
    wait(&mut world, 1);
    assert!(world.enemies[0].noticed);
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
fn with_life_drain_some_enemies_felled_give_a_hp_back() {
    assert_eq!(drained(0, 200), 0, "none without it");
    let first = drained(1, 200);
    assert!((5..=40).contains(&first), "about 10%: {first}");
    let last = drained(5, 200);
    assert!((35..=90).contains(&last), "about 30%: {last}");
}

#[test]
fn a_strike_with_the_player_in_its_ring_puts_their_torch_out_up_to_them_to_light_it_again() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.ticks = 30 * 40;
    world.torch_steady_at = 0;
    assert!(world.torch_lit());
    world.strike(world.player.position + Vec2::new(3.0, 0.0), false);
    assert!(world.torch_lit(), "out of its ring");
    world.strike(world.player.position + Vec2::new(2.0, 0.0), false);
    assert_eq!(world.torch_on(), Some(false), "out");
    wait(&mut world, 30 * 5);
    assert_eq!(world.torch_on(), Some(false), "not back of itself");
    world.switch_torch();
    assert_eq!(world.torch_on(), Some(true));
}
