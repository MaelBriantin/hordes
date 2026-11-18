use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

/// A point on the map or a move, in tiles.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// The same direction at length 1, or zero for zero.
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f32::EPSILON {
            Self::ZERO
        } else {
            self * (1.0 / length)
        }
    }

    /// The same direction, no longer than `max`.
    pub fn clamp_length(self, max: f32) -> Self {
        let length = self.length();
        if length > max {
            self * (max / length)
        } else {
            self
        }
    }

    /// The middle of a tile.
    #[allow(
        clippy::cast_precision_loss,
        reason = "tiles stay far within what an f32 holds exactly"
    )]
    pub fn middle_of((x, y): (i32, i32)) -> Self {
        Self::new(x as f32 + 0.5, y as f32 + 0.5)
    }

    /// The tile this point is on.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "points stay on the map, far within i32"
    )]
    pub fn tile(self) -> (i32, i32) {
        (self.x.floor() as i32, self.y.floor() as i32)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
