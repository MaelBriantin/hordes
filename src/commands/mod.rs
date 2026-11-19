//! What `hordes` does when given a command instead of opening the game: `update`, `uninstall`,
//! `reset`. They talk in the terminal, in English, and ask before erasing anything.

pub mod reset;
pub mod uninstall;
pub mod update;

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// The game's name, as the installer and its receipt know it.
const APP: &str = "hordes";

/// The files the game keeps in its data folder: the settings and the leaderboard.
const DATA_FILES: [&str; 2] = ["settings.txt", "scores.txt"];

/// Whether the game keeps any of its files in `folder`.
fn holds_data(folder: &Path) -> bool {
    DATA_FILES.iter().any(|file| folder.join(file).exists())
}

/// Erases the game's files from its data folder, and the folder itself if nothing else is left
/// in it: anything else there stays.
fn erase_data(folder: &Path) -> io::Result<()> {
    for file in DATA_FILES {
        let path = folder.join(file);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    // Gone with them only if it is empty now.
    let _ = fs::remove_dir(folder);
    Ok(())
}

/// Where the installer left its receipt: the folder `XDG_CONFIG_HOME` names if it holds one,
/// else `~/.config/hordes` (`%LOCALAPPDATA%\hordes` on Windows), as the updater looks for it.
fn receipt_path() -> Option<PathBuf> {
    let file = format!("{APP}-receipt.json");
    let xdg = std::env::var_os("XDG_CONFIG_HOME")
        .map(|home| PathBuf::from(home).join(APP).join(&file))
        .filter(|path| path.exists());
    let home = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(|home| PathBuf::from(home).join(APP))
    } else {
        dirs::home_dir().map(|home| home.join(".config").join(APP))
    };
    xdg.or_else(|| home.map(|folder| folder.join(&file)))
}

/// Asks a yes-or-no question in the terminal: yes only if the answer says so.
fn confirm(question: &str) -> io::Result<bool> {
    print!("{question} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    Ok(says_yes(&answer))
}

/// Whether an answer says yes: in English or in French, any case.
fn says_yes(answer: &str) -> bool {
    matches!(
        answer.trim().to_lowercase().as_str(),
        "y" | "yes" | "o" | "oui"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_clear_yes_says_yes() {
        for yes in ["y", "Yes", " oui\n", "O"] {
            assert!(says_yes(yes), "{yes:?}");
        }
        for no in ["", "n", "no", "nope", "yeah maybe"] {
            assert!(!says_yes(no), "{no:?}");
        }
    }
}
