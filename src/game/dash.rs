//! The dash, from the Dash card: a few tiles in a blink, nothing hurting the player meanwhile;
//! ready again a while after, two at most.

use super::vec2::Vec2;

/// How far a dash goes, in tiles, and how long it lasts, in ticks.
const DASH_TILES: f32 = 3.0;
const DASH_TICKS: u32 = 8;

/// The dashes the player has ready, and how far the next one has come back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dashes {
    pub ready: u8,
    /// The ticks the next dash has been coming back for, while one is missing.
    pub charging: u32,
}

impl Dashes {
    /// One dash ready, for when the Dash card is taken.
    pub fn new() -> Self {
        Self {
            ready: 1,
            charging: 0,
        }
    }

    /// A tick: a missing dash comes back once `recharge` ticks have gone by, one after the
    /// other, up to `most`.
    pub fn tick(&mut self, most: u8, recharge: u32) {
        if self.ready >= most {
            self.charging = 0;
            return;
        }
        self.charging += 1;
        if self.charging >= recharge {
            self.ready += 1;
            self.charging = 0;
        }
    }

    /// Takes a dash, if one is ready: whether the player dashes.
    pub fn take(&mut self) -> bool {
        if self.ready == 0 {
            return false;
        }
        self.ready -= 1;
        true
    }
}

/// A dash under way: its way, and the ticks it has left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dash {
    pub way: Vec2,
    pub left: u32,
}

impl Dash {
    /// A dash the way `way` goes, whole.
    pub fn new(way: Vec2) -> Self {
        Self {
            way: way.normalized(),
            left: DASH_TICKS,
        }
    }

    /// The step a tick of dashing takes, in tiles.
    pub fn step(self) -> Vec2 {
        self.way * (DASH_TILES / f32::from(u16::try_from(DASH_TICKS).unwrap_or(1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dash_taken_comes_back_after_its_recharge() {
        let mut dashes = Dashes::new();
        assert!(dashes.take());
        assert!(!dashes.take(), "one only");
        for _ in 0..89 {
            dashes.tick(1, 90);
        }
        assert_eq!(dashes.ready, 0);
        dashes.tick(1, 90);
        assert_eq!(dashes.ready, 1);
    }

    #[test]
    fn missing_dashes_come_back_one_after_the_other() {
        let mut dashes = Dashes {
            ready: 0,
            charging: 0,
        };
        for _ in 0..60 {
            dashes.tick(3, 30);
        }
        assert_eq!(dashes.ready, 2);
        for _ in 0..100 {
            dashes.tick(3, 30);
        }
        assert_eq!((dashes.ready, dashes.charging), (3, 0));
    }

    #[test]
    fn a_dash_goes_three_tiles() {
        let dash = Dash::new(Vec2::new(1.0, 0.0));
        let gone = dash.step() * f32::from(u16::try_from(DASH_TICKS).unwrap_or(1));
        assert!((gone.x - DASH_TILES).abs() < 1e-4);
    }
}
