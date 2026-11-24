//! The chargers: scraping the ground, charging, running into the player, a barricade, a rock.

use super::super::super::barricade::Barricade;
use super::super::super::charger::{
    CHARGE_TICKS, Charge, DAMAGE, REST_TICKS, SCRAPE_TICKS, STUN_TICKS,
};
use super::*;

/// A charger at `at`, having found the player at (100, 50), no weapon about.
fn facing_a_charger(map: Map, at: Vec2) -> World {
    let mut world = on(map, Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    let mut charger = enemy(EnemyKind::Charger, at);
    charger.noticed = true;
    world.enemies.push(charger);
    world
}

/// The same, already charging left.
fn charged_by(map: Map, at: Vec2) -> World {
    let mut world = facing_a_charger(map, at);
    world.enemies[0].charge = Charge::Charging {
        way: Vec2::new(-1.0, 0.0),
        left: CHARGE_TICKS,
    };
    world
}

#[test]
fn a_charger_near_the_player_scrapes_a_second_then_charges_straight_the_way_they_were() {
    let mut world = facing_a_charger(Map::empty(), Vec2::new(106.0, 50.0));
    wait(&mut world, SCRAPE_TICKS);
    assert_eq!(
        world.enemies[0].position,
        Vec2::new(106.0, 50.0),
        "scraping"
    );
    assert_eq!(world.enemies[0].facing, Facing::Left);
    assert!(matches!(world.enemies[0].charge, Charge::Charging { .. }));
    // Stepping aside: it charges on, turning no more.
    world.player.position = Vec2::new(100.0, 44.0);
    wait(&mut world, 8);
    let charger = world.enemies[0].position;
    assert!((charger.y - 50.0).abs() < 1e-3, "{charger:?}");
    assert!(charger.x < 103.0, "{charger:?}");
}

#[test]
fn a_charger_away_from_the_player_walks_on() {
    let mut world = facing_a_charger(Map::empty(), Vec2::new(115.0, 50.0));
    wait(&mut world, 5);
    assert_eq!(world.enemies[0].charge, Charge::Ready);
    assert!(world.enemies[0].position.x < 115.0);
}

#[test]
fn a_charge_hurts_the_player_and_throws_them_back() {
    let mut world = charged_by(Map::empty(), Vec2::new(106.0, 50.0));
    wait(&mut world, 15);
    assert_eq!(
        world.player.hp,
        BASE_HP - DAMAGE,
        "its charge only, not its blow too"
    );
    assert!(
        world.player.position.x < 98.5,
        "{:?}",
        world.player.position
    );
    assert!(matches!(world.enemies[0].charge, Charge::Resting(left) if left <= REST_TICKS));
}

#[test]
fn a_charger_smashes_a_wooden_barricade_and_charges_on_but_one_of_metal_stops_it() {
    for (level, smashed) in [(1, true), (5, false)] {
        let mut world = charged_by(Map::empty(), Vec2::new(110.0, 50.0));
        world.player.position = Vec2::new(90.0, 50.0);
        let barricade = Barricade::new((104, 49), Facing::Up, level);
        for tile in barricade.tiles() {
            world.map.barricade(tile, true);
        }
        world.barricades.push(barricade);
        wait(&mut world, 20);
        let charger = &world.enemies[0];
        assert_eq!(world.barricades.is_empty(), smashed, "level {level}");
        assert_eq!(charger.stunned > 0, !smashed, "level {level}");
        if smashed {
            assert!(!world.map.blocks((105, 50)));
            assert!(charger.position.x < 104.0, "{:?}", charger.position);
        }
    }
}

#[test]
fn a_charger_running_into_a_rock_is_stunned_two_seconds_stars_over_its_head() {
    let mut map = Map::empty();
    map.put((96, 50), Obstacle::Rock);
    let mut world = charged_by(map, Vec2::new(104.0, 51.0));
    world.player.position = Vec2::new(90.0, 40.0);
    wait(&mut world, 15);
    let charger = &world.enemies[0];
    assert!(
        (charger.position.x - 98.65).abs() < 0.01,
        "{:?}",
        charger.position
    );
    assert!(charger.stunned > STUN_TICKS - 15 && charger.dazed > 0);
    assert!(matches!(charger.charge, Charge::Resting(_)));
}
