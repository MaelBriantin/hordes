// First, so `fl!` is there for every other module.
#[macro_use]
mod i18n;

mod app;
mod commands;
mod game;
mod ui;

const USAGE: &str = "Usage: hordes            play
       hordes update     install the latest version
       hordes reset      erase your settings and your leaderboard
       hordes uninstall  remove the game from this computer
       hordes --version  show this version";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        None => {}
        Some("update") => return commands::update::run(),
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
    ratatui::run(|terminal| app::App::new(sprites).run(terminal))?;
    Ok(())
}
