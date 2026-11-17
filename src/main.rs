mod app;
mod game;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sprites = ui::Sprites::load()?;
    ratatui::run(|terminal| app::App::new(sprites).run(terminal))?;
    Ok(())
}
