//! `hordes update`: fetches the latest release and installs it in place of this one. Only for a
//! game installed by its installer, which leaves a receipt saying where it came from; built
//! from source, it is updated the way it was built.
//!
//! The game used to install in `~/.cargo/bin`, and installs in `~/.local/bin` now: a game
//! installed in the old place is offered the move on its update.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use axoupdater::AxoUpdater;

use super::{APP, confirm, receipt_path};
/// A GitHub token to ask for releases with, if set: for tests, which might hit the rate limit
/// of anonymous requests.
const TOKEN_VARIABLE: &str = "HORDES_GITHUB_TOKEN";

/// Where the update goes, against where the game is.
#[derive(Debug, PartialEq, Eq)]
enum Target {
    /// Where it is, as the updater finds it.
    Same,
    /// Where it is, told the updater in so many words: a game the installer put straight in a
    /// `bin` folder, which the updater would otherwise put in a `bin` folder within it.
    Root(PathBuf),
    /// The old place: the new one offered, its root given.
    Old(PathBuf),
}

/// Where the update of a game installed under `prefix` goes, the installer's layout `flat` or
/// not, `home` the player's home folder. The updater installs under a root, in its `bin` folder.
fn target(prefix: &Path, flat: bool, home: &Path) -> Target {
    if prefix == home.join(".cargo") {
        Target::Old(home.join(".local"))
    } else if flat && prefix.ends_with("bin") {
        prefix
            .parent()
            .map_or(Target::Same, |root| Target::Root(root.to_path_buf()))
    } else {
        Target::Same
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut updater = AxoUpdater::new_for(APP);
    if let Ok(token) = std::env::var(TOKEN_VARIABLE) {
        updater.set_github_token(&token);
    }
    if updater.load_receipt().is_err() {
        println!(
            "This copy of {APP} wasn't installed by its installer: update it the way you got it."
        );
        return Ok(());
    }
    let prefix = PathBuf::from(updater.install_prefix_root()?.as_str());
    let flat = receipt_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .is_some_and(|receipt| receipt.contains(r#""install_layout":"flat""#));
    let home = dirs::home_dir().unwrap_or_default();
    let mut moving = false;
    match target(&prefix, flat, &home) {
        Target::Same => {}
        Target::Root(root) => {
            updater.set_install_dir(root.to_string_lossy().as_ref());
        }
        Target::Old(root) => {
            println!(
                "{APP} now installs in ~/.local/bin, rather than ~/.cargo/bin where this copy is."
            );
            if confirm("Move it there with this update?")? {
                updater.set_install_dir(root.to_string_lossy().as_ref());
                moving = true;
            }
        }
    }
    match updater.run_sync()? {
        Some(update) => {
            println!("{APP} is now version {}.", update.new_version);
            if moving {
                leave_old_place(&prefix)?;
                println!(
                    "{APP} is in ~/.local/bin now: open a new terminal for your shell to find it there."
                );
            }
        }
        None => println!(
            "{APP} is up to date (version {}).",
            env!("CARGO_PKG_VERSION")
        ),
    }
    Ok(())
}

/// Removes this copy from the old place, now the update is in the new one: only if it runs
/// from there, not a copy built from source. On Windows, the updater has already removed it.
fn leave_old_place(prefix: &Path) -> Result<(), Box<dyn Error>> {
    let here = std::env::current_exe()?;
    if cfg!(unix) && here.parent() == Some(prefix.join("bin").as_path()) {
        self_replace::self_delete()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_place_is_offered_the_move_the_flat_one_given_its_root() {
        let home = Path::new("/home/player");
        assert_eq!(
            target(&home.join(".cargo"), false, home),
            Target::Old(home.join(".local"))
        );
        assert_eq!(
            target(&home.join(".local/bin"), true, home),
            Target::Root(home.join(".local"))
        );
        assert_eq!(target(&home.join(".local"), false, home), Target::Same);
    }
}
