//! The player's progress, kept between runs in `progress.ron` in the game's data folder: the
//! achievements reached, and the counts towards those that take several runs.

use std::fs;
use std::io;

use super::settings::data_file;
use crate::game::Progress;

const FILE: &str = "progress.ron";

/// The progress kept, or none reached yet if there is none or it can't be read.
pub fn load() -> Progress {
    data_file(FILE)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| ron::from_str(&text).ok())
        .unwrap_or_default()
}

/// Keeps the progress for the next runs.
pub fn save(progress: &Progress) -> io::Result<()> {
    let path = data_file(FILE).ok_or_else(|| io::Error::other("no data folder"))?;
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let text = ron::ser::to_string_pretty(progress, ron::ser::PrettyConfig::default())
        .map_err(io::Error::other)?;
    fs::write(path, text)
}
