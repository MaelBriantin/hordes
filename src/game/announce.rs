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

/// How a new weather is told: in big letters, the world standing still or going on; or on a
/// banner on the top of the map, the world going on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherNews {
    #[default]
    Paused,
    Big,
    Banner,
}

impl WeatherNews {
    /// The next way, round and round.
    pub fn next(self) -> Self {
        match self {
            WeatherNews::Paused => WeatherNews::Big,
            WeatherNews::Big => WeatherNews::Banner,
            WeatherNews::Banner => WeatherNews::Paused,
        }
    }

    /// The way before, round and round.
    pub fn previous(self) -> Self {
        self.next().next()
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            WeatherNews::Paused => "paused",
            WeatherNews::Big => "big",
            WeatherNews::Banner => "banner",
        }
    }
}
