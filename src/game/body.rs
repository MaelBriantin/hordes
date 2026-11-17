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

/// Whether two square bodies overlap.
pub fn touch(a: Vec2, a_half: f32, b: Vec2, b_half: f32) -> bool {
    let reach = a_half + b_half;
    (a.x - b.x).abs() < reach && (a.y - b.y).abs() < reach
}
