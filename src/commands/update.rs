//! `hordes update`: fetches the latest release and installs it in place of this one. Only for a
//! game installed by its installer, which leaves a receipt saying where it came from; built
//! from source, it is updated the way it was built.
//!
//! The game used to install in `~/.cargo/bin`, and installs in `~/.local/bin` now: a game
//! installed in the old place is updated there, as uv does.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use axoupdater::AxoUpdater;

use super::{APP, receipt_path};
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
}

/// Where the update of a game installed under `prefix` goes, the installer's layout `flat` or
/// not. The updater installs under a root, in its `bin` folder.
fn target(prefix: &Path, flat: bool) -> Target {
    if flat && prefix.ends_with("bin") {
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
    if let Target::Root(root) = target(&prefix, flat) {
        updater.set_install_dir(root.to_string_lossy().as_ref());
    }
    match updater.run_sync()? {
        Some(update) => println!("{APP} is now version {}.", update.new_version),
        None => println!(
            "{APP} is up to date (version {}).",
            env!("CARGO_PKG_VERSION")
        ),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flat_layout_is_given_its_root_the_others_stay_where_they_are() {
        let home = Path::new("/home/player");
        assert_eq!(
            target(&home.join(".local/bin"), true),
            Target::Root(home.join(".local"))
        );
        assert_eq!(target(&home.join(".cargo"), false), Target::Same);
        assert_eq!(target(&home.join(".local"), false), Target::Same);
    }
}
