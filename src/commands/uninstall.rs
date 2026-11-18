//! `hordes uninstall`: removes the game installed by its installer, after asking: the program
//! itself and the installer's receipt, then, if the player wants, the settings and the
//! leaderboard.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use axoupdater::AxoUpdater;

use super::{APP, confirm, erase_data, holds_data};
use crate::app::data_folder;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut updater = AxoUpdater::new_for(APP);
    let installed = updater.load_receipt().is_ok()
        && updater
            .check_receipt_is_for_this_executable()
            .unwrap_or(false);
    let Some(receipt) = receipt_path().filter(|path| installed && path.exists()) else {
        println!(
            "This copy of {APP} wasn't installed by its installer: remove it the way you got it."
        );
        return Ok(());
    };
    if !confirm(&format!("Remove {APP} from this computer?"))? {
        println!("Nothing removed.");
        return Ok(());
    }
    self_replace::self_delete()?;
    fs::remove_file(&receipt)?;
    if let Some(folder) = receipt.parent() {
        // Gone with it only if nothing else is left in it.
        let _ = fs::remove_dir(folder);
    }
    println!("{APP} is removed.");

    if let Some(folder) = data_folder().filter(|folder| holds_data(folder))
        && confirm("Erase your settings and your leaderboard too?")?
    {
        erase_data(&folder)?;
        println!("Your settings and your leaderboard are erased.");
    }
    println!(
        "The line the installer may have added to your shell's profile, for your PATH, stays: it does no harm."
    );
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
