//! What the game stops a moment to announce, in big letters: a new weather, a horde coming. The
//! world stands still while it shows: the clock, the weather and the horde wait.

use super::TICKS_PER_SECOND;
use super::weather::Weather;

/// How long an announcement stands, in seconds.
const SECONDS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum News {
    /// The weather the run comes into.
    Weather(Weather),
    /// A horde on its way, out of one side, or of every side.
    Horde { everywhere: bool },
}

/// News being told, and the ticks it still stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Announcement {
    pub news: News,
    pub left: u32,
}

impl Announcement {
    pub fn new(news: News) -> Self {
        Self {
            news,
            left: SECONDS * u32::from(TICKS_PER_SECOND),
        }
    }
}
