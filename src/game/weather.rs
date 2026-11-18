//! The weather: the sun sets at the start of a run, in a minute; then the night's weather worsens
//! as the run goes on, stage by stage, every 2 minutes, as the difficulty does, until the sun
//! rises. Rain hides the horde, lightning shows it all, and from the storm on, strikes of
//! lightning hurt the enemies they fall among.

use super::TICKS_PER_SECOND;

/// How long the dusk lasts, in seconds: the sun lights the map all of it but its last seconds,
/// when the night falls, all of a sudden.
const DUSK_SECONDS: u32 = 30;
/// How long the night takes to fall, at the end of the dusk, in seconds.
const NIGHTFALL_SECONDS: u32 = 6;
/// The daylight left as the night falls: the dusk dims to it, little by little, then falls from
/// it to nothing.
const DUSK_LOW: f32 = 0.5;
/// How long each stage of weather after the dusk lasts, in seconds.
const STAGE_SECONDS: u32 = 120;
/// The seconds into the run the sun has risen at, after the tempest's two minutes: the run is
/// won.
pub const DAWN_SECONDS: u32 = DUSK_SECONDS + 5 * STAGE_SECONDS;
/// How long the sun takes to rise, before the dawn, in seconds.
const SUNRISE_SECONDS: u32 = 60;
/// How long the enemies flee the risen sun, burning one by one, before the run is won, in
/// seconds.
const DAYBREAK_SECONDS: u32 = 4;
/// What a strike of lightning takes from each enemy near where it falls, in HP, from the storm.
pub const STRIKE_DAMAGE: u32 = 30;
/// How near where it falls a strike hurts enemies, in tiles.
pub const STRIKE_RADIUS: f32 = 2.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weather {
    Dusk,
    Clear,
    Drizzle,
    Rain,
    Storm,
    Tempest,
}

impl Weather {
    /// The weather `seconds` into a run.
    pub fn at(seconds: u32) -> Self {
        let Some(after_dusk) = seconds.checked_sub(DUSK_SECONDS) else {
            return Weather::Dusk;
        };
        match after_dusk / STAGE_SECONDS {
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
            Weather::Dusk => Some(Weather::Clear),
            Weather::Clear => Some(Weather::Drizzle),
            Weather::Drizzle => Some(Weather::Rain),
            Weather::Rain => Some(Weather::Storm),
            Weather::Storm => Some(Weather::Tempest),
            Weather::Tempest => None,
        }
    }

    /// The ticks between two strikes of lightning, on average: only from the storm on, the rain
    /// alone before it.
    pub fn lightning_every(self) -> Option<u32> {
        let seconds = match self {
            Weather::Dusk | Weather::Clear | Weather::Drizzle | Weather::Rain => return None,
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
            Weather::Dusk | Weather::Clear | Weather::Drizzle => 0.0,
            Weather::Rain => 1.0,
            Weather::Storm => 2.0,
            Weather::Tempest => 3.0,
        }
    }

    /// The HP of the enemies appearing, against their own, in percent.
    pub fn hp_percent(self) -> u32 {
        match self {
            Weather::Dusk | Weather::Clear | Weather::Drizzle => 100,
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

/// The seconds into the run a weather starts at.
pub fn starts_at(weather: Weather) -> u32 {
    match weather {
        Weather::Dusk => 0,
        Weather::Clear => DUSK_SECONDS,
        Weather::Drizzle => DUSK_SECONDS + STAGE_SECONDS,
        Weather::Rain => DUSK_SECONDS + 2 * STAGE_SECONDS,
        Weather::Storm => DUSK_SECONDS + 3 * STAGE_SECONDS,
        Weather::Tempest => DUSK_SECONDS + 4 * STAGE_SECONDS,
    }
}

/// The tick the sun has risen at.
pub fn dawn() -> u32 {
    DAWN_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// The tick the sun starts to rise at: no more lightning from then on.
pub fn sunrise() -> u32 {
    (DAWN_SECONDS - SUNRISE_SECONDS) * u32::from(TICKS_PER_SECOND)
}

/// The ticks the enemies flee the risen sun for, before the run is won.
pub fn daybreak() -> u32 {
    DAYBREAK_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// How much daylight there is `ticks` into a run, from 1 (the setting or rising sun) to 0 (the
/// night): less and less through the dusk, down to `DUSK_LOW`, then, in its last seconds, all of
/// a sudden, nothing; more and more again over the last minute before the dawn.
pub fn daylight(ticks: u32) -> f32 {
    let second = u32::from(TICKS_PER_SECOND);
    let share = |part: u32, whole: u32| {
        f32::from(u16::try_from(part.min(whole)).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(whole.max(1)).unwrap_or(u16::MAX))
    };
    let nightfall = NIGHTFALL_SECONDS * second;
    let sunset = DUSK_SECONDS * second - nightfall;
    if ticks >= sunrise() {
        share(ticks - sunrise(), SUNRISE_SECONDS * second)
    } else if ticks < sunset {
        1.0 - (1.0 - DUSK_LOW) * share(ticks, sunset)
    } else {
        DUSK_LOW * (1.0 - share(ticks - sunset, nightfall))
    }
}

/// How far `seconds` into a run is through its stage of weather, from 0 to 1.
pub fn progress(seconds: u32) -> f32 {
    let (into, length) = match seconds.checked_sub(DUSK_SECONDS) {
        None => (seconds, DUSK_SECONDS),
        Some(after_dusk) if Weather::at(seconds) == Weather::Tempest => {
            (after_dusk - 4 * STAGE_SECONDS, STAGE_SECONDS)
        }
        Some(after_dusk) => (after_dusk % STAGE_SECONDS, STAGE_SECONDS),
    };
    f32::from(u16::try_from(into.min(length)).unwrap_or(0))
        / f32::from(u16::try_from(length).unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weather_worsens_after_half_a_minute_of_dusk_then_every_two_minutes() {
        let stages: Vec<Weather> = [0, 29, 30, 149, 150, 270, 390, 510, 3_000]
            .map(Weather::at)
            .into();
        assert_eq!(
            stages,
            [
                Weather::Dusk,
                Weather::Dusk,
                Weather::Clear,
                Weather::Clear,
                Weather::Drizzle,
                Weather::Rain,
                Weather::Storm,
                Weather::Tempest,
                Weather::Tempest
            ]
        );
        assert_eq!(DAWN_SECONDS, 630);
    }

    #[test]
    fn a_clear_night_has_no_lightning_and_a_storm_hurts() {
        assert_eq!(Weather::Clear.lightning_every(), None);
        assert!(!Weather::Rain.strikes_hurt());
        assert!(Weather::Storm.strikes_hurt());
    }

    #[test]
    fn the_sun_sets_then_the_night_falls_then_the_sun_rises() {
        let second = u32::from(TICKS_PER_SECOND);
        assert!((daylight(0) - 1.0).abs() < 1e-4);
        assert!((daylight(12 * second) - 0.75).abs() < 1e-4);
        assert!((daylight(24 * second) - 0.5).abs() < 1e-4);
        assert!((daylight(27 * second) - 0.25).abs() < 1e-4);
        assert!(daylight(30 * second) < f32::EPSILON);
        assert!(daylight(569 * second) < f32::EPSILON);
        assert!((daylight(600 * second) - 0.5).abs() < 1e-4);
        assert!((daylight(630 * second) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn progress_runs_through_each_stage() {
        assert!(progress(0) < f32::EPSILON);
        assert!((progress(15) - 0.5).abs() < 1e-4);
        assert!(progress(30) < f32::EPSILON);
        assert!((progress(90) - 0.5).abs() < 1e-4);
        assert!((progress(570) - 0.5).abs() < 1e-4);
        assert!((progress(3_000) - 1.0).abs() < 1e-4);
    }
}
