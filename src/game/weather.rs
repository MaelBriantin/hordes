//! The weather: the sun sets at the start of a run, in half a minute; then the night's weather
//! worsens as the run goes on, stage by stage, every 2 minutes, until the sun rises. In arcade,
//! the stages last 3 minutes, up to the tempest and back down to the night, again and again, with
//! no dawn. Rain hides the horde, lightning shows it all, and from the storm on, strikes of
//! lightning hurt the enemies they fall among.

use super::TICKS_PER_SECOND;
use super::mode::Mode;

/// How long the dusk lasts, in seconds: the sun lights the map all of it but its last seconds,
/// when the night falls, all of a sudden.
pub const DUSK_SECONDS: u32 = 30;
/// How long the night takes to fall, at the end of the dusk, in seconds.
const NIGHTFALL_SECONDS: u32 = 6;
/// The daylight left as the night falls: the dusk dims to it, little by little, then falls from
/// it to nothing.
const DUSK_LOW: f32 = 0.5;
/// How long each stage of weather after the dusk lasts, in seconds: in a standard run, and in
/// arcade.
const STAGE_SECONDS: u32 = 120;
const ARCADE_STAGE_SECONDS: u32 = 180;
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
/// How near where it falls a strike hurts enemies, and the player, in tiles.
pub const STRIKE_RADIUS: f32 = 2.5;
/// What a strike of lightning takes from the player, in HP.
pub const STRIKE_PLAYER_DAMAGE: u32 = 20;
/// How long a strike's ring shows where it will fall before it does, in ticks: half a second,
/// time enough to run out of it from its middle.
pub const STRIKE_WARNING_TICKS: u32 = 15;

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
    /// The weather `seconds` into a run: in arcade, up to the tempest and back down to the night,
    /// again and again.
    pub fn at(seconds: u32, mode: Mode) -> Self {
        let Some(after_dusk) = seconds.checked_sub(DUSK_SECONDS) else {
            return Weather::Dusk;
        };
        let stage = after_dusk / stage_seconds(mode);
        let stage = match mode {
            Mode::Standard => stage.min(4),
            // There and back: up to the tempest, down to the night, and up again.
            Mode::Arcade => match stage % 8 {
                up @ 0..=4 => up,
                down => 8 - down,
            },
        };
        match stage {
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

    /// Whether its strikes of lightning hurt the enemies they fall among, and the player.
    pub fn strikes_hurt(self) -> bool {
        self >= Weather::Storm
    }

    /// How often a strike that hurts falls where the player stands, in percent: the others fall
    /// anywhere they see.
    pub fn aimed_strike_percent(self) -> u32 {
        match self {
            Weather::Tempest => 50,
            Weather::Storm => 33,
            _ => 0,
        }
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
            Weather::Dusk | Weather::Clear => 100,
            Weather::Drizzle => 115,
            Weather::Rain => 130,
            Weather::Storm => 150,
            Weather::Tempest => 200,
        }
    }

    /// The enemies' speed, against their own, in percent.
    pub fn speed_percent(self) -> u16 {
        if self == Weather::Tempest { 110 } else { 100 }
    }
}

/// How long a stage of weather lasts in that mode, in seconds.
pub fn stage_seconds(mode: Mode) -> u32 {
    match mode {
        Mode::Standard => STAGE_SECONDS,
        Mode::Arcade => ARCADE_STAGE_SECONDS,
    }
}

/// The seconds into the run the next stage of weather starts at, after `seconds`: none after the
/// tempest of a standard run.
pub fn next_stage_at(seconds: u32, mode: Mode) -> Option<u32> {
    let Some(after_dusk) = seconds.checked_sub(DUSK_SECONDS) else {
        return Some(DUSK_SECONDS);
    };
    let stage = after_dusk / stage_seconds(mode);
    (mode == Mode::Arcade || stage < 4).then(|| DUSK_SECONDS + (stage + 1) * stage_seconds(mode))
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
/// a sudden, nothing; more and more again over the last minute before the dawn, in a standard
/// run.
pub fn daylight(ticks: u32, mode: Mode) -> f32 {
    let second = u32::from(TICKS_PER_SECOND);
    let share = |part: u32, whole: u32| {
        f32::from(u16::try_from(part.min(whole)).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(whole.max(1)).unwrap_or(u16::MAX))
    };
    let nightfall = NIGHTFALL_SECONDS * second;
    let sunset = DUSK_SECONDS * second - nightfall;
    if mode == Mode::Standard && ticks >= sunrise() {
        share(ticks - sunrise(), SUNRISE_SECONDS * second)
    } else if ticks < sunset {
        1.0 - (1.0 - DUSK_LOW) * share(ticks, sunset)
    } else {
        DUSK_LOW * (1.0 - share(ticks - sunset, nightfall))
    }
}

/// How far `seconds` into a run is through its stage of weather, from 0 to 1: the tempest of a
/// standard run lasts until the dawn.
pub fn progress(seconds: u32, mode: Mode) -> f32 {
    let stage = stage_seconds(mode);
    let (into, length) = match seconds.checked_sub(DUSK_SECONDS) {
        None => (seconds, DUSK_SECONDS),
        Some(after_dusk) if mode == Mode::Standard && after_dusk >= 4 * stage => {
            (after_dusk - 4 * stage, stage)
        }
        Some(after_dusk) => (after_dusk % stage, stage),
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
            .map(|seconds| Weather::at(seconds, Mode::Standard))
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
        assert!((daylight(0, Mode::Standard) - 1.0).abs() < 1e-4);
        assert!((daylight(12 * second, Mode::Standard) - 0.75).abs() < 1e-4);
        assert!((daylight(24 * second, Mode::Standard) - 0.5).abs() < 1e-4);
        assert!((daylight(27 * second, Mode::Standard) - 0.25).abs() < 1e-4);
        assert!(daylight(30 * second, Mode::Standard) < f32::EPSILON);
        assert!(daylight(569 * second, Mode::Standard) < f32::EPSILON);
        assert!((daylight(600 * second, Mode::Standard) - 0.5).abs() < 1e-4);
        assert!((daylight(630 * second, Mode::Standard) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn progress_runs_through_each_stage() {
        assert!(progress(0, Mode::Standard) < f32::EPSILON);
        assert!((progress(15, Mode::Standard) - 0.5).abs() < 1e-4);
        assert!(progress(30, Mode::Standard) < f32::EPSILON);
        assert!((progress(90, Mode::Standard) - 0.5).abs() < 1e-4);
        assert!((progress(570, Mode::Standard) - 0.5).abs() < 1e-4);
        assert!((progress(3_000, Mode::Standard) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn in_arcade_the_weather_goes_there_and_back_every_three_minutes() {
        let at = |seconds| Weather::at(seconds, Mode::Arcade);
        assert_eq!(at(29), Weather::Dusk);
        assert_eq!(at(30), Weather::Clear);
        assert_eq!(at(30 + 180), Weather::Drizzle);
        assert_eq!(at(30 + 4 * 180), Weather::Tempest);
        assert_eq!(at(30 + 5 * 180), Weather::Storm);
        assert_eq!(at(30 + 8 * 180), Weather::Clear);
        assert_eq!(at(30 + 9 * 180), Weather::Drizzle);
        assert_eq!(
            next_stage_at(30 + 4 * 180, Mode::Arcade),
            Some(30 + 5 * 180)
        );
        assert_eq!(next_stage_at(30 + 4 * 120, Mode::Standard), None);
        assert!(daylight(3_000 * u32::from(TICKS_PER_SECOND), Mode::Arcade) < f32::EPSILON);
    }
}
