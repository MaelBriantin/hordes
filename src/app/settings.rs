//! The player's settings, kept between runs in `settings.txt` in the game's data folder
//! (`~/Library/Application Support/hordes` on macOS, `~/.local/share/hordes` on Linux), or in
//! `HORDES_DATA_DIR` when it is set. One setting per line: `language = fr`, `blood = mayo`.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::game::{Mode, WeatherNews};
use crate::ui::{Arrow, BloodLook, RainDisplay, SKINS, Skin, colours_of};

/// Where the settings go instead of the data folder, if set: for tests.
const DATA_DIR_VARIABLE: &str = "HORDES_DATA_DIR";
const FILE: &str = "settings.txt";

/// A language the game's texts come in: a folder of `i18n/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    French,
}

impl Language {
    /// Its folder in `i18n/`, and its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::French => "fr",
        }
    }

    /// Its name in its own language, so a player finds it whatever language is on.
    pub fn name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::French => "Français",
        }
    }

    /// The other one, round and round.
    pub fn next(self) -> Self {
        match self {
            Language::English => Language::French,
            Language::French => Language::English,
        }
    }

    /// The system's language, if the game speaks it: from `LC_ALL`, `LC_MESSAGES` or `LANG`
    /// (`fr_FR.UTF-8`); English otherwise.
    fn of_system() -> Self {
        let speaks_french = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.is_empty())
            .is_some_and(|value| value.starts_with("fr"));
        if speaks_french {
            Language::French
        } else {
            Language::English
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools, reason = "settings, each on its own")]
pub struct Settings {
    /// The language the game speaks: the system's at first.
    pub language: Language,
    /// How blood shows: stains where enemies fall and the player is hit.
    pub blood: BloodLook,
    /// How the rain shows during a run.
    pub rain: RainDisplay,
    /// The arrow over the player, in its colour.
    pub arrow: Arrow,
    /// How the hero looks; and in which colours each hero shows, by their place among the skins.
    pub skin: Skin,
    pub colours: [u8; SKINS.len()],
    /// The mode runs are played in.
    pub mode: Mode,
    /// Whether the gear shows over the top left of the map; as a list rather than icons.
    pub gear: bool,
    pub gear_list: bool,
    /// Whether the player's HP shows in a small bar over their head.
    pub mini_bars: bool,
    /// Whether the ground is coloured, grass speckled with earth: an experiment, off at first.
    pub ground: bool,
    /// How a new weather is told.
    pub weather_news: WeatherNews,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::of_system(),
            blood: BloodLook::Red,
            rain: RainDisplay::Full,
            arrow: Arrow::Off,
            skin: Skin::Survivor,
            colours: [0; SKINS.len()],
            mode: Mode::Standard,
            gear: false,
            gear_list: false,
            mini_bars: false,
            ground: false,
            weather_news: WeatherNews::Big,
        }
    }
}

impl Settings {
    /// The settings kept, or the default ones if there are none or they can't be read.
    pub fn load() -> Self {
        path()
            .and_then(|path| fs::read_to_string(path).ok())
            .map_or_else(Self::default, |text| Self::parse(&text))
    }

    /// Keeps the settings for the next runs.
    pub fn save(self) -> io::Result<()> {
        let path = path().ok_or_else(|| io::Error::other("no data folder"))?;
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder)?;
        }
        fs::write(path, self.text())
    }

    /// Settings read from their text; an unknown or broken line is left out.
    fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match (key.trim(), value.trim()) {
                ("language", "en") => settings.language = Language::English,
                ("language", "fr") => settings.language = Language::French,
                // `on`, the setting of before there was ketchup.
                ("blood", "red" | "on") => settings.blood = BloodLook::Red,
                ("blood", "ketchup") => settings.blood = BloodLook::Ketchup,
                ("blood", "mayo") => settings.blood = BloodLook::Mayo,
                ("blood", "pesto") => settings.blood = BloodLook::Pesto,
                ("blood", "off") => settings.blood = BloodLook::Off,
                ("rain", "full" | "heavy") => settings.rain = RainDisplay::Full,
                ("rain", "reduced" | "light") => settings.rain = RainDisplay::Reduced,
                ("rain", "off") => settings.rain = RainDisplay::Off,
                // `outline`, the line around the player before there was the arrow.
                ("arrow" | "outline", "off") => settings.arrow = Arrow::Off,
                ("arrow" | "outline", "white") => settings.arrow = Arrow::White,
                ("arrow" | "outline", "yellow") => settings.arrow = Arrow::Yellow,
                ("arrow" | "outline", "red") => settings.arrow = Arrow::Red,
                ("skin", "survivor") => settings.skin = Skin::Survivor,
                ("skin", "classic") => settings.skin = Skin::Classic,
                ("skin", "exterminator") => settings.skin = Skin::Exterminator,
                ("skin", "demolisher") => settings.skin = Skin::Demolisher,
                ("skin", "agent") => settings.skin = Skin::Agent,
                ("skin", "ninja") => settings.skin = Skin::Ninja,
                ("skin", "medic") => settings.skin = Skin::Medic,
                ("skin", "infiltrator") => settings.skin = Skin::Infiltrator,
                ("mode", "standard") => settings.mode = Mode::Standard,
                ("mode", "arcade") => settings.mode = Mode::Arcade,
                ("gear", "shown") => settings.gear = true,
                ("gear", "hidden") => settings.gear = false,
                ("gear_layout", "icons") => settings.gear_list = false,
                ("gear_layout", "list") => settings.gear_list = true,
                ("mini_bars", "shown") => settings.mini_bars = true,
                ("mini_bars", "hidden") => settings.mini_bars = false,
                ("ground", "coloured") => settings.ground = true,
                ("ground", "plain") => settings.ground = false,
                ("announcements", "paused") => settings.weather_news = WeatherNews::Paused,
                ("announcements", "big") => settings.weather_news = WeatherNews::Big,
                // `quiet`, the setting of before there were three.
                ("announcements", "banner" | "quiet") => {
                    settings.weather_news = WeatherNews::Banner;
                }
                ("colours", list) => settings.colours = colours_from(list),
                _ => {}
            }
        }
        settings
    }

    fn text(self) -> String {
        format!(
            "language = {}\nblood = {}\nrain = {}\narrow = {}\nskin = {}\ncolours = {}\nmode = {}\ngear = {}\ngear_layout = {}\nmini_bars = {}\nground = {}\nannouncements = {}\n",
            self.language.id(),
            self.blood.id(),
            self.rain.id(),
            self.arrow.id(),
            self.skin.id(),
            colours_text(self.colours),
            self.mode.id(),
            if self.gear { "shown" } else { "hidden" },
            if self.gear_list { "list" } else { "icons" },
            if self.mini_bars { "shown" } else { "hidden" },
            if self.ground { "coloured" } else { "plain" },
            self.weather_news.id()
        )
    }
}

/// Each hero's colours, as written: `medic:1`, comma apart, only those not in their first.
fn colours_text(colours: [u8; SKINS.len()]) -> String {
    SKINS
        .iter()
        .zip(colours)
        .filter(|(_, colour)| *colour > 0)
        .map(|(skin, colour)| format!("{}:{colour}", skin.id()))
        .collect::<Vec<_>>()
        .join(",")
}

/// Each hero's colours, as read; one unknown or past their colours stays at their first.
fn colours_from(list: &str) -> [u8; SKINS.len()] {
    let mut colours = [0; SKINS.len()];
    for pair in list.split(',') {
        let Some((id, colour)) = pair.split_once(':') else {
            continue;
        };
        let place = SKINS.iter().position(|skin| skin.id() == id.trim());
        let colour = colour.trim().parse::<u8>().ok();
        if let (Some(place), Some(colour)) = (place, colour)
            && colour < colours_of(SKINS[place])
        {
            colours[place] = colour;
        }
    }
    colours
}

fn path() -> Option<PathBuf> {
    data_file(FILE)
}

/// A file of the game's data folder.
pub fn data_file(name: &str) -> Option<PathBuf> {
    Some(data_folder()?.join(name))
}

/// The game's data folder, or `HORDES_DATA_DIR` when it is set.
pub fn data_folder() -> Option<PathBuf> {
    match std::env::var_os(DATA_DIR_VARIABLE) {
        Some(folder) => Some(PathBuf::from(folder)),
        None => Some(dirs::data_dir()?.join("hordes")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_read_back_what_they_wrote() {
        let settings = Settings {
            language: Language::French,
            blood: BloodLook::Mayo,
            rain: RainDisplay::Reduced,
            arrow: Arrow::Yellow,
            skin: Skin::Exterminator,
            colours: {
                let mut colours = [0; SKINS.len()];
                colours[6] = 1;
                colours
            },
            mode: Mode::Arcade,
            gear: false,
            gear_list: false,
            mini_bars: true,
            ground: true,
            weather_news: WeatherNews::Big,
        };
        assert_eq!(Settings::parse(&settings.text()), settings);
        assert_eq!(
            Settings::parse(&Settings::default().text()),
            Settings::default()
        );
    }

    #[test]
    fn blood_reads_its_four_looks_and_the_old_on() {
        let blood = |line: &str| Settings::parse(line).blood;
        assert_eq!(blood("blood = ketchup"), BloodLook::Ketchup);
        assert_eq!(blood("blood = mayo"), BloodLook::Mayo);
        assert_eq!(blood("blood = pesto"), BloodLook::Pesto);
        assert_eq!(blood("blood = on"), BloodLook::Red);
        assert_eq!(blood("blood = off"), BloodLook::Off);
    }

    #[test]
    fn a_hero_s_colours_read_back_and_one_out_of_their_range_stays_at_their_first() {
        let colours = colours_from("medic:1, ninja:2, nobody:3, survivor:x");
        let medic = SKINS.iter().position(|skin| *skin == Skin::Medic).unwrap();
        assert_eq!(colours[medic], 1);
        assert_eq!(
            colours.iter().filter(|colour| **colour > 0).count(),
            1,
            "the ninja has two, not three"
        );
        assert_eq!(colours_text(colours), "medic:1");
    }

    #[test]
    fn a_broken_file_gives_the_default_settings() {
        assert_eq!(
            Settings::parse("blood: maybe\nnonsense"),
            Settings::default()
        );
    }
}
