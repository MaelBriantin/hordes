//! The player's settings, kept between runs in `settings.txt` in the game's data folder
//! (`~/Library/Application Support/hordes` on macOS, `~/.local/share/hordes` on Linux), or in
//! `HORDES_DATA_DIR` when it is set. One setting per line: `language = fr`, `blood = off`.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::ui::RainDisplay;

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
pub struct Settings {
    /// The language the game speaks: the system's at first.
    pub language: Language,
    /// Whether blood shows: stains where enemies fall and the player is hit.
    pub blood: bool,
    /// How the rain shows during a run.
    pub rain: RainDisplay,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::of_system(),
            blood: true,
            rain: RainDisplay::Full,
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
                ("blood", "on") => settings.blood = true,
                ("blood", "off") => settings.blood = false,
                ("rain", "full" | "heavy") => settings.rain = RainDisplay::Full,
                ("rain", "reduced" | "light") => settings.rain = RainDisplay::Reduced,
                ("rain", "off") => settings.rain = RainDisplay::Off,
                _ => {}
            }
        }
        settings
    }

    fn text(self) -> String {
        format!(
            "language = {}\nblood = {}\nrain = {}\n",
            self.language.id(),
            if self.blood { "on" } else { "off" },
            self.rain.id()
        )
    }
}

fn path() -> Option<PathBuf> {
    let folder = match std::env::var_os(DATA_DIR_VARIABLE) {
        Some(folder) => PathBuf::from(folder),
        None => dirs::data_dir()?.join("hordes"),
    };
    Some(folder.join(FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_read_back_what_they_wrote() {
        let settings = Settings {
            language: Language::French,
            blood: false,
            rain: RainDisplay::Reduced,
        };
        assert_eq!(Settings::parse(&settings.text()), settings);
        assert_eq!(
            Settings::parse(&Settings::default().text()),
            Settings::default()
        );
    }

    #[test]
    fn a_broken_file_gives_the_default_settings() {
        assert_eq!(
            Settings::parse("blood: maybe\nnonsense"),
            Settings::default()
        );
    }
}
