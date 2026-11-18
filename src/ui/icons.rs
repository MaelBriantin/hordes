//! Icons from the Nerd Fonts (<https://www.nerdfonts.com>), for the status line: the game wants
//! a Nerd Font in the terminal (see the README). Each icon is written as its code point, next to
//! its Nerd Font name.

use crate::game::Weather;

/// `nf-md-heart`: the HP.
pub const HP: &str = "\u{F02D1}";
/// `nf-md-timer`: the time survived.
pub const TIME: &str = "\u{F13AB}";
/// `nf-md-weather_sunset_up`: the dawn, at the end of the weathers.
pub const DAWN: &str = "\u{F059C}";
/// `nf-md-trophy`: the score.
pub const SCORE: &str = "\u{F0538}";

/// The weather's icon.
pub fn weather(weather: Weather) -> &'static str {
    match weather {
        Weather::Dusk => "\u{F059B}",    // nf-md-weather_sunset_down
        Weather::Clear => "\u{F0594}",   // nf-md-weather_night
        Weather::Drizzle => "\u{F0597}", // nf-md-weather_rainy
        Weather::Rain => "\u{F0596}",    // nf-md-weather_pouring
        Weather::Storm => "\u{F067E}",   // nf-md-weather_lightning_rainy
        Weather::Tempest => "\u{F0898}", // nf-md-weather_hurricane
    }
}
