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

/// Where the update of the game at `binary` goes, the installer's layout `flat` or not: a flat
/// layout, in the binary's own folder, whatever the receipt says, since the installer puts the
/// update exactly in the folder it is given; any other, where the receipt says, as the updater
/// finds it.
fn target(binary: &Path, flat: bool) -> Option<PathBuf> {
    flat.then(|| binary.parent().map(Path::to_path_buf))
        .flatten()
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
    let flat = receipt_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .is_some_and(|receipt| receipt.contains(r#""install_layout":"flat""#));
    let binary = std::env::current_exe()?.canonicalize()?;
    if let Some(folder) = target(&binary, flat) {
        updater.set_install_dir(folder.to_string_lossy().as_ref());
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
    fn a_flat_layout_is_updated_in_the_binary_s_folder() {
        let home = Path::new("/home/player");
        assert_eq!(
            target(&home.join(".local/bin/hordes"), true),
            Some(home.join(".local/bin"))
        );
    }

    #[test]
    fn another_layout_is_updated_where_the_receipt_says() {
        assert_eq!(
            target(Path::new("/home/player/.cargo/bin/hordes"), false),
            None
        );
    }
}
