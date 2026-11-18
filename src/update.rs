//! `hordes update`: fetches the latest release and installs it in place of this one. Only for a
//! game installed by its installer, which leaves a receipt saying where it came from; built
//! from source, it is updated the way it was built.

use std::error::Error;

use axoupdater::AxoUpdater;

const APP: &str = "hordes";
/// A GitHub token to ask for releases with, if set: for tests, which might hit the rate limit
/// of anonymous requests.
const TOKEN_VARIABLE: &str = "HORDES_GITHUB_TOKEN";

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
    match updater.run_sync()? {
        Some(update) => println!("{APP} is now version {}.", update.new_version),
        None => println!(
            "{APP} is up to date (version {}).",
            env!("CARGO_PKG_VERSION")
        ),
    }
    Ok(())
}
