//! The score: every enemy felled is worth points, times the weather's multiplier, which grows
//! as the run worsens, and times the combo, which grows with kills that follow each other close.

use super::TICKS_PER_SECOND;
use super::enemy::EnemyKind;
use super::weather::Weather;

/// What reaching the dawn adds to the score, in percent of it.
pub const DAWN_BONUS_PERCENT: u64 = 50;
/// The most the combo adds to the multiplier, in tenths: up to ×2.
const MAX_COMBO: u32 = 10;

/// What felling an enemy of a kind is worth, before the multipliers.
pub fn points(kind: EnemyKind) -> u64 {
    match kind {
        EnemyKind::Zombie => 10,
        EnemyKind::Shadow => 30,
        EnemyKind::Abomination => 100,
        EnemyKind::Parasite => 1,
    }
}

/// The weather's multiplier, in tenths: ×1 on a clear night up to ×5 in the tempest.
pub fn weather_tenths(weather: Weather) -> u64 {
    match weather {
        Weather::Dusk | Weather::Clear => 10,
        Weather::Drizzle => 15,
        Weather::Rain => 20,
        Weather::Storm => 30,
        Weather::Tempest => 50,
    }
}

/// The combo's multiplier, in tenths, for `combo` kills in a row: ×1, and ×0.1 more per kill,
/// up to ×2.
pub fn combo_tenths(combo: u32) -> u64 {
    10 + u64::from(combo.min(MAX_COMBO))
}

/// How long after a kill the next one keeps the combo going, in ticks: 2 seconds.
pub fn combo_window() -> u32 {
    2 * u32::from(TICKS_PER_SECOND)
}

/// What a kill is worth with both multipliers, rounded down.
pub fn worth(kind: EnemyKind, weather: Weather, combo: u32) -> u64 {
    points(kind) * weather_tenths(weather) * combo_tenths(combo) / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kill_is_worth_its_points_times_the_multipliers() {
        assert_eq!(worth(EnemyKind::Zombie, Weather::Clear, 0), 10);
        assert_eq!(worth(EnemyKind::Zombie, Weather::Storm, 0), 30);
        assert_eq!(worth(EnemyKind::Abomination, Weather::Tempest, 5), 750);
    }

    #[test]
    fn the_combo_stops_growing_at_twice() {
        assert_eq!(combo_tenths(3), 13);
        assert_eq!(combo_tenths(50), 20);
    }
}
