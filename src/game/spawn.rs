//! Where and what enemies appear: just beyond what the player sees, more and more of them, and
//! tougher kinds, as the weather worsens.

use std::f32::consts::TAU;

use rand::{Rng, RngExt};

use super::enemy::EnemyKind;
use super::weather::Weather;
use super::{Facing, Vec2};

/// The most enemies at once.
pub const MAX_ENEMIES: usize = 300;
/// How far beyond what the player sees enemies appear, in tiles: past the edge with their whole
/// sprite.
pub const MARGIN: f32 = 3.0;

/// How many enemies appear per second in a weather, at its start.
fn rate_of(weather: Weather) -> f32 {
    match weather {
        Weather::Dusk => 1.0,
        Weather::Clear => 1.5,
        Weather::Drizzle => 3.0,
        Weather::Rain => 4.5,
        Weather::Storm => 6.0,
        Weather::Tempest => 7.5,
    }
}

/// How many enemies appear per second in a weather, `progress` through it: from its own rate to
/// the next weather's, little by little.
pub fn rate(weather: Weather, progress: f32) -> f32 {
    let now = rate_of(weather);
    let next = weather.next().map_or(now, rate_of);
    now + (next - now) * progress
}

/// How often each kind of enemy appears in a weather, against the others: zombie, Shadow,
/// abomination, kamikaze, spitter, charger.
fn weights(weather: Weather) -> [u32; 6] {
    match weather {
        Weather::Dusk => [10, 0, 0, 0, 0, 0],
        Weather::Clear => [9, 1, 0, 0, 0, 0],
        Weather::Drizzle => [8, 1, 1, 1, 0, 0],
        Weather::Rain => [7, 2, 1, 1, 1, 0],
        Weather::Storm => [6, 2, 2, 2, 1, 1],
        Weather::Tempest => [5, 2, 3, 2, 2, 2],
    }
}

/// A kind of enemy, at random by the weather's weights.
pub fn pick_kind(rng: &mut impl Rng, weather: Weather) -> EnemyKind {
    let weights = weights(weather);
    let mut roll = rng.random_range(0..weights.iter().sum::<u32>().max(1));
    for (kind, weight) in EnemyKind::ALL.into_iter().zip(weights) {
        if roll < weight {
            return kind;
        }
        roll -= weight;
    }
    EnemyKind::Zombie
}

/// A point just beyond what the player sees (`view`: half its size, in tiles, around
/// `centre`): on the `side` given, or on one at random.
pub fn edge_point(rng: &mut impl Rng, centre: Vec2, view: Vec2, side: Option<Facing>) -> Vec2 {
    let side = side.unwrap_or_else(|| {
        [Facing::Up, Facing::Down, Facing::Left, Facing::Right][rng.random_range(0..4)]
    });
    let (reach_x, reach_y) = (view.x + MARGIN, view.y + MARGIN);
    let along_x = rng.random_range(-reach_x..=reach_x);
    let along_y = rng.random_range(-reach_y..=reach_y);
    let offset = match side {
        Facing::Up => Vec2::new(along_x, -reach_y),
        Facing::Down => Vec2::new(along_x, reach_y),
        Facing::Left => Vec2::new(-reach_x, along_y),
        Facing::Right => Vec2::new(reach_x, along_y),
    };
    centre + offset
}

/// An enemy's own number, drawn as it appears: where it starts to wander.
pub fn phase(rng: &mut impl Rng) -> f32 {
    rng.random_range(0.0..TAU)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    #[test]
    fn the_rate_grows_through_the_weather() {
        assert!((rate(Weather::Dusk, 0.0) - 1.0).abs() < f32::EPSILON);
        assert!((rate(Weather::Clear, 0.5) - 2.25).abs() < f32::EPSILON);
        assert!((rate(Weather::Tempest, 1.0) - 7.5).abs() < f32::EPSILON);
    }

    #[test]
    fn the_dusk_brings_zombies_only_and_the_tempest_every_kind() {
        let mut rng = StdRng::seed_from_u64(3);
        let early: Vec<_> = (0..200)
            .map(|_| pick_kind(&mut rng, Weather::Dusk))
            .collect();
        assert!(early.iter().all(|kind| *kind == EnemyKind::Zombie));
        let late: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Tempest))
            .collect();
        for kind in EnemyKind::ALL {
            assert!(late.contains(&kind), "no {kind:?}");
        }
    }

    #[test]
    fn the_kamikazes_come_with_the_drizzle() {
        let mut rng = StdRng::seed_from_u64(6);
        let clear: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Clear))
            .collect();
        assert!(!clear.contains(&EnemyKind::Kamikaze));
        let drizzle: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Drizzle))
            .collect();
        assert!(drizzle.contains(&EnemyKind::Kamikaze));
    }

    #[test]
    fn the_spitters_come_with_the_rain() {
        let mut rng = StdRng::seed_from_u64(7);
        let drizzle: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Drizzle))
            .collect();
        assert!(!drizzle.contains(&EnemyKind::Spitter));
        let rain: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Rain))
            .collect();
        assert!(rain.contains(&EnemyKind::Spitter));
    }

    #[test]
    fn the_chargers_come_with_the_storm() {
        let mut rng = StdRng::seed_from_u64(8);
        let rain: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Rain))
            .collect();
        assert!(!rain.contains(&EnemyKind::Charger));
        let storm: Vec<_> = (0..500)
            .map(|_| pick_kind(&mut rng, Weather::Storm))
            .collect();
        assert!(storm.contains(&EnemyKind::Charger));
    }

    #[test]
    fn edge_points_lie_beyond_the_view() {
        let mut rng = StdRng::seed_from_u64(4);
        let (centre, view) = (Vec2::new(100.0, 50.0), Vec2::new(30.0, 12.0));
        for _ in 0..200 {
            let point = edge_point(&mut rng, centre, view, None);
            let (dx, dy) = ((point.x - centre.x).abs(), (point.y - centre.y).abs());
            assert!(dx >= view.x + MARGIN - 1e-3 || dy >= view.y + MARGIN - 1e-3);
        }
    }

    #[test]
    fn an_edge_point_on_a_side_is_on_that_side() {
        let mut rng = StdRng::seed_from_u64(5);
        let (centre, view) = (Vec2::new(100.0, 50.0), Vec2::new(30.0, 12.0));
        let point = edge_point(&mut rng, centre, view, Some(Facing::Left));
        assert!((point.x - (centre.x - view.x - MARGIN)).abs() < 1e-3);
    }
}
