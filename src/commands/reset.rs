//! `hordes reset`: erases the settings, the leaderboard and the progress, after asking. The game itself stays.

use std::error::Error;

use super::{confirm, erase_data, holds_data};
use crate::app::data_folder;

pub fn run() -> Result<(), Box<dyn Error>> {
    let Some(folder) = data_folder().filter(|folder| holds_data(folder)) else {
        println!("Nothing to erase: no settings nor leaderboard kept yet.");
        return Ok(());
    };
    println!(
        "Your settings, your leaderboard and your progress are kept in {}.",
        folder.display()
    );
    if !confirm("Erase them?")? {
        println!("Nothing erased.");
        return Ok(());
    }
    erase_data(&folder)?;
    println!("Erased: the next run starts with the default settings and an empty leaderboard.");
    Ok(())
}
