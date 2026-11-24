// First, so `fl!` is there for every other module.
#[macro_use]
mod i18n;

mod app;
mod commands;
mod game;
mod ui;

const USAGE: &str = "Usage: hordes            play
       hordes update     install the latest version
       hordes reset      erase your settings, your leaderboard and your progress
       hordes uninstall  remove the game from this computer
       hordes --version  show this version";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        None => {}
        Some("update") => return commands::update::run().map(|_| ()),
        Some("reset") => return commands::reset::run(),
        Some("uninstall") => return commands::uninstall::run(),
        Some("--version" | "-V") => {
            println!("hordes {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            return Ok(());
        }
        Some(other) => {
            eprintln!("Unknown argument: {other}\n{USAGE}");
            std::process::exit(2);
        }
    }
    let sprites = ui::Sprites::load()?;
    let mut game = app::App::new(sprites);
    ratatui::run(|terminal| game.run(terminal))?;
    // `U` on the title screen, a newer version out: installed now, the game quit, then opened
    // again in its new version. Where it is read before the update: on Linux, the old binary
    // replaced, it reads as deleted.
    if game.wants_update() {
        let binary = std::env::current_exe()?;
        if commands::update::run()? {
            return relaunch(&binary);
        }
    }
    Ok(())
}

/// Opens the game at `binary` in place of this one: on Unix, this process becomes it; on
/// Windows, it runs till it is quit.
fn relaunch(binary: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = std::process::Command::new(binary);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec().into())
    }
    #[cfg(not(unix))]
    {
        let status = command.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}
