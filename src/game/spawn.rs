//! Where and what enemies appear: just beyond what the player sees, more and more of them.

use std::f32::consts::TAU;

use rand::{Rng, RngExt};

use super::enemy::EnemyKind;
use super::{Facing, Vec2};

/// The most enemies at once.
pub const MAX_ENEMIES: usize = 300;
/// How far beyond what the player sees enemies appear, in tiles: past the edge with their whole
/// sprite.
pub const MARGIN: f32 = 3.0;

/// How many enemies appear per second: 1, and 1 more each minute.
pub fn rate(seconds: u32) -> f32 {
    f32::from(u16::try_from(1 + seconds / 60).unwrap_or(u16::MAX))
}

/// A kind of enemy, at random among those arrived by then, by their weights.
pub fn pick_kind(rng: &mut impl Rng, seconds: u32) -> EnemyKind {
    let arrived = || {
        EnemyKind::ALL
            .into_iter()
            .filter(move |kind| kind.stats().arrives_at <= seconds)
    };
    let total: u32 = arrived().map(|kind| kind.stats().weight).sum();
    let mut roll = rng.random_range(0..total.max(1));
    for kind in arrived() {
        let weight = kind.stats().weight;
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

/// A bat's own start in its zigzag.
pub fn phase(rng: &mut impl Rng) -> f32 {
    rng.random_range(0.0..TAU)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    #[test]
    fn the_rate_grows_each_minute() {
        assert!((rate(0) - 1.0).abs() < f32::EPSILON);
        assert!((rate(59) - 1.0).abs() < f32::EPSILON);
        assert!((rate(60) - 2.0).abs() < f32::EPSILON);
        assert!((rate(9 * 60 + 30) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn only_arrived_kinds_appear() {
        let mut rng = StdRng::seed_from_u64(3);
        let early: Vec<_> = (0..200).map(|_| pick_kind(&mut rng, 59)).collect();
        assert!(early.iter().all(|kind| *kind == EnemyKind::Zombie));
        let late: Vec<_> = (0..500).map(|_| pick_kind(&mut rng, 180)).collect();
        for kind in EnemyKind::ALL {
            assert!(late.contains(&kind), "no {kind:?}");
        }
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
