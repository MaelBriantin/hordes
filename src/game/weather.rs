//! The weather: it worsens as the run goes on, stage by stage, every 2 minutes, as the
//! difficulty does. Rain hides the horde, lightning shows it all, and from the storm on, strikes
//! of lightning hurt the enemies they fall among.

use super::TICKS_PER_SECOND;

/// How long a stage of weather lasts, in seconds.
const STAGE_SECONDS: u32 = 120;
/// What a strike of lightning takes from each enemy near where it falls, in HP, from the storm.
pub const STRIKE_DAMAGE: u32 = 30;
/// How near where it falls a strike hurts enemies, in tiles.
pub const STRIKE_RADIUS: f32 = 2.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weather {
    Clear,
    Drizzle,
    Rain,
    Storm,
    Tempest,
}

impl Weather {
    /// The weather `seconds` into a run.
    pub fn at(seconds: u32) -> Self {
        match seconds / STAGE_SECONDS {
            0 => Weather::Clear,
            1 => Weather::Drizzle,
            2 => Weather::Rain,
            3 => Weather::Storm,
            _ => Weather::Tempest,
        }
    }

    /// The weather after this one; none after the tempest.
    pub fn next(self) -> Option<Self> {
        match self {
            Weather::Clear => Some(Weather::Drizzle),
            Weather::Drizzle => Some(Weather::Rain),
            Weather::Rain => Some(Weather::Storm),
            Weather::Storm => Some(Weather::Tempest),
            Weather::Tempest => None,
        }
    }

    /// The ticks between two strikes of lightning, on average; none on a clear night.
    pub fn lightning_every(self) -> Option<u32> {
        let seconds = match self {
            Weather::Clear => return None,
            Weather::Drizzle => 8,
            Weather::Rain => 5,
            Weather::Storm => 3,
            Weather::Tempest => 2,
        };
        Some(seconds * u32::from(TICKS_PER_SECOND))
    }

    /// Whether its strikes of lightning hurt the enemies they fall among.
    pub fn strikes_hurt(self) -> bool {
        self >= Weather::Storm
    }

    /// How much less far the player's light reaches, in tiles.
    pub fn light_loss(self) -> f32 {
        match self {
            Weather::Clear | Weather::Drizzle => 0.0,
            Weather::Rain => 1.0,
            Weather::Storm => 2.0,
            Weather::Tempest => 3.0,
        }
    }

    /// The HP of the enemies appearing, against their own, in percent.
    pub fn hp_percent(self) -> u32 {
        match self {
            Weather::Clear | Weather::Drizzle => 100,
            Weather::Rain => 110,
            Weather::Storm => 125,
            Weather::Tempest => 150,
        }
    }

    /// The enemies' speed, against their own, in percent.
    pub fn speed_percent(self) -> u16 {
        if self == Weather::Tempest { 110 } else { 100 }
    }
}

/// How far `seconds` into a run is through its stage of weather, from 0 to 1.
pub fn progress(seconds: u32) -> f32 {
    f32::from(u16::try_from(seconds % STAGE_SECONDS).unwrap_or(0))
        / f32::from(u16::try_from(STAGE_SECONDS).unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weather_worsens_every_two_minutes() {
        let stages: Vec<Weather> = [0, 119, 120, 240, 360, 480, 3_000].map(Weather::at).into();
        assert_eq!(
            stages,
            [
                Weather::Clear,
                Weather::Clear,
                Weather::Drizzle,
                Weather::Rain,
                Weather::Storm,
                Weather::Tempest,
                Weather::Tempest
            ]
        );
    }

    #[test]
    fn a_clear_night_has_no_lightning_and_a_storm_hurts() {
        assert_eq!(Weather::Clear.lightning_every(), None);
        assert!(!Weather::Rain.strikes_hurt());
        assert!(Weather::Storm.strikes_hurt());
    }

    #[test]
    fn progress_runs_through_each_stage() {
        assert!(progress(0) < f32::EPSILON);
        assert!((progress(60) - 0.5).abs() < 1e-4);
        assert!(progress(120) < f32::EPSILON);
    }
}
