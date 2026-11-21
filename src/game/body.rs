//! Bodies: squares that move on the map, stopped by what blocks them. A walking body is stopped
//! by obstacles, a flying one by nothing.

use super::{Map, Vec2};

/// How far within a body its edges count: a body touching an obstacle does not overlap it.
const SKIN: f32 = 1e-3;

/// Where a square body (half as wide as `half`) centred on `from` ends up moving by `step`,
/// less than a tile each way: one axis, then the other. Stopped on one, it still slides along
/// the other.
pub fn slide(map: &Map, from: Vec2, step: Vec2, half: f32, flies: bool) -> Vec2 {
    let mut at = from;
    if step.x != 0.0 {
        at.x += step.x;
        if !fits(map, at, half, flies) {
            // Back to touching the column it ran into.
            at.x = if step.x > 0.0 {
                (at.x + half - SKIN).floor() - half
            } else {
                (at.x - half + SKIN).floor() + 1.0 + half
            };
        }
    }
    if step.y != 0.0 {
        at.y += step.y;
        if !fits(map, at, half, flies) {
            at.y = if step.y > 0.0 {
                (at.y + half - SKIN).floor() - half
            } else {
                (at.y - half + SKIN).floor() + 1.0 + half
            };
        }
    }
    at
}

/// Whether a square body centred on `centre` stands clear of everything that stops it.
pub fn fits(map: &Map, centre: Vec2, half: f32, flies: bool) -> bool {
    if flies {
        return true;
    }
    let inner = half - SKIN;
    let (left, top) = (centre - Vec2::new(inner, inner)).tile();
    let (right, bottom) = (centre + Vec2::new(inner, inner)).tile();
    (top..=bottom).all(|y| (left..=right).all(|x| !map.blocks((x, y))))
}

/// How far apart the points looked at along a line of sight are, in tiles: well under a tile,
/// so no obstacle's corner slips between two.
const SIGHT_STEP: f32 = 0.25;

/// Whether `to` is in sight from `from`: no obstacle on the line between them, `to`'s own tile
/// included (a Shadow over a rock is out of sight). A barricade hides nothing.
pub fn in_sight(map: &Map, from: Vec2, to: Vec2) -> bool {
    let along = to - from;
    let length = along.length();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a gun's range, a few dozen steps"
    )]
    let steps = (length / SIGHT_STEP).ceil() as u32;
    (1..=steps).all(|step| {
        let share = f32::from(u16::try_from(step).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(steps).unwrap_or(1));
        !map.stops_shots((from + along * share).tile())
    })
}

/// Whether two square bodies overlap.
pub fn touch(a: Vec2, a_half: f32, b: Vec2, b_half: f32) -> bool {
    let reach = a_half + b_half;
    (a.x - b.x).abs() < reach && (a.y - b.y).abs() < reach
}
