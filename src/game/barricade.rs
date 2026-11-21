//! The barricade, from the Barricade card: crates just ahead of the player, across their way:
//! heaped wide when they face up or down, in a file going away when they face left or right. It
//! stops the enemies that walk, which batter it until it breaks; the Shadows fly over it, the
//! shots go over it. A few at most stand at once, the oldest taken away for a new one.

use super::{Facing, TICKS_PER_SECOND, Vec2};

/// The tiles a barricade covers across the player's way, and along it.
const ACROSS: i32 = 4;
const ALONG: i32 = 2;
/// How far ahead of the player's body it is set down, in tiles.
const GAP: f32 = 0.1;
/// How close to a barricade an enemy batters it, in tiles: its body stopped against it.
const REACH: f32 = 0.1;
/// The HP of a barricade, by the card's level.
const HPS: [u32; 5] = [80, 120, 120, 180, 180];
/// How long a hit barricade shows white, in ticks.
pub const FLASH_TICKS: u32 = 3;
/// How long a barricade that can't be set down shows where it would have stood, in ticks.
pub const REFUSED_TICKS: u32 = 12;

/// A barricade set down: the top left tile it covers, the way the player faced, its HP left,
/// and whether it is of metal: unbreakable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Barricade {
    pub corner: (i32, i32),
    pub facing: Facing,
    pub hp: u32,
    pub metal: bool,
    /// Ticks left showing white, after a hit.
    pub flash: u32,
}

impl Barricade {
    pub fn new(corner: (i32, i32), facing: Facing, level: u8) -> Self {
        Self {
            corner,
            facing,
            hp: hp(level),
            metal: level >= 5,
            flash: 0,
        }
    }

    /// Whether it is a file of crates going away: set down facing left or right.
    pub fn file(&self) -> bool {
        matches!(self.facing, Facing::Left | Facing::Right)
    }

    /// The tiles it covers, each way.
    pub fn size(&self) -> (i32, i32) {
        size(self.facing)
    }

    /// The tiles it covers.
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32)> {
        let (left, top) = self.corner;
        let (width, height) = self.size();
        (top..top + height).flat_map(move |y| (left..left + width).map(move |x| (x, y)))
    }

    /// Its top left corner and its size, in tiles.
    fn bounds(&self) -> (Vec2, Vec2) {
        let (width, height) = self.size();
        (
            corner_point(self.corner),
            Vec2::new(tiles(width), tiles(height)),
        )
    }

    /// Its middle.
    pub fn centre(&self) -> Vec2 {
        let (corner, size) = self.bounds();
        corner + size * 0.5
    }

    /// Whether a square body (half as wide as `half`) centred on `centre` overlaps it, or more
    /// than `margin` around it.
    fn within(&self, centre: Vec2, half: f32, margin: f32) -> bool {
        let (corner, size) = self.bounds();
        let reach = half + margin;
        centre.x > corner.x - reach
            && centre.x < corner.x + size.x + reach
            && centre.y > corner.y - reach
            && centre.y < corner.y + size.y + reach
    }

    /// Whether a body overlaps it: in the way of setting it down.
    pub fn overlaps(&self, centre: Vec2, half: f32) -> bool {
        self.within(centre, half, 0.0)
    }

    /// Whether a body touches it, stopped against it: an enemy batters it.
    pub fn touches(&self, centre: Vec2, half: f32) -> bool {
        self.within(centre, half, REACH)
    }

    /// Where a body in its way as it is set down is thrown: just beyond it, the way the player
    /// faced.
    pub fn beyond(&self, centre: Vec2, half: f32) -> Vec2 {
        let (corner, size) = self.bounds();
        let clear = half + REACH;
        match self.facing {
            Facing::Right => Vec2::new(corner.x + size.x + clear, centre.y),
            Facing::Left => Vec2::new(corner.x - clear, centre.y),
            Facing::Down => Vec2::new(centre.x, corner.y + size.y + clear),
            Facing::Up => Vec2::new(centre.x, corner.y - clear),
        }
    }
}

/// The tiles a barricade set down facing `facing` covers, each way: across the way, 4.
fn size(facing: Facing) -> (i32, i32) {
    match facing {
        Facing::Left | Facing::Right => (ALONG, ACROSS),
        Facing::Up | Facing::Down => (ACROSS, ALONG),
    }
}

fn tiles(count: i32) -> f32 {
    f32::from(u8::try_from(count).unwrap_or(u8::MAX))
}

fn corner_point((x, y): (i32, i32)) -> Vec2 {
    Vec2::middle_of((x, y)) - Vec2::new(0.5, 0.5)
}

/// The top left tile of a barricade set down just ahead of a body (half as wide as `half`)
/// centred on `position`, facing `facing`: as centred on it the other way as the tiles allow.
pub fn corner_ahead(position: Vec2, facing: Facing, half: f32) -> (i32, i32) {
    let edge = position + facing.way() * (half + GAP);
    let (width, height) = size(facing);
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a tile of the map, well within an i32"
    )]
    let (ahead, behind, centred) = (
        |at: f32| at.ceil() as i32,
        |at: f32, size: i32| at.floor() as i32 - size,
        |at: f32, size: i32| (at - tiles(size) / 2.0).round() as i32,
    );
    match facing {
        Facing::Right => (ahead(edge.x), centred(position.y, height)),
        Facing::Left => (behind(edge.x, width), centred(position.y, height)),
        Facing::Down => (centred(position.x, width), ahead(edge.y)),
        Facing::Up => (centred(position.x, width), behind(edge.y, height)),
    }
}

/// The HP of a barricade, by the card's level.
pub fn hp(level: u8) -> u32 {
    HPS[usize::from(level.clamp(1, 5) - 1)]
}

/// How many barricades stand at once, at most, by the card's level: two, three from level 3.
pub fn most(level: u8) -> usize {
    if level >= 3 { 3 } else { 2 }
}

/// How long the next barricade takes to come, in ticks, by the card's level: 8 s, 6 s from
/// level 3.
pub fn recharge(level: u8) -> u32 {
    let seconds = if level >= 3 { 6 } else { 8 };
    seconds * u32::from(TICKS_PER_SECOND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_down_ahead_it_never_overlaps_the_body() {
        for x in [10.0, 10.3, 10.5, 10.9, 10.99] {
            let position = Vec2::new(x, 20.5);
            for facing in [Facing::Right, Facing::Left, Facing::Down, Facing::Up] {
                let barricade = Barricade::new(corner_ahead(position, facing, 0.45), facing, 1);
                assert!(!barricade.overlaps(position, 0.45), "{x} {facing:?}");
                let apart = barricade.centre() - position;
                let (along, across) = match facing {
                    Facing::Left | Facing::Right => (apart.x, apart.y),
                    Facing::Up | Facing::Down => (apart.y, apart.x),
                };
                assert!(along.abs() < 2.6, "{x} {facing:?}: {along}");
                assert!(across.abs() <= 0.5, "{x} {facing:?}: {across}");
            }
        }
    }
}
