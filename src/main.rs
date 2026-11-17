// First, so `fl!` is there for every other module.
#[macro_use]
mod i18n;

mod app;
mod game;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sprites = ui::Sprites::load()?;
    ratatui::run(|terminal| app::App::new(sprites).run(terminal))?;
    Ok(())
}
