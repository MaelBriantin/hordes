//! A debug tool, in debug builds only: in a run, Tab opens a console to try at once what comes
//! late in it (the next weather, a horde, the dawn…), the run standing still meanwhile.

use ratatui::crossterm::event::KeyCode;

use crate::game::World;
use crate::ui::{DebugAction, Screen};

/// Whether the key opens the console: Tab, in a debug build.
pub fn opens(code: KeyCode) -> bool {
    cfg!(debug_assertions) && code == KeyCode::Tab
}

/// Keys in the console: ↑↓ choose an action, Enter does it and goes back to the run (but for
/// the switch, which stays), Esc or Tab go back. The screen it leaves the app on.
pub fn handle_key(world: &mut World, code: KeyCode, selected: usize) -> Screen {
    let count = DebugAction::ALL.len();
    match code {
        KeyCode::Up => Screen::Debug {
            selected: (selected + count - 1) % count,
        },
        KeyCode::Down => Screen::Debug {
            selected: (selected + 1) % count,
        },
        KeyCode::Enter => {
            match DebugAction::ALL[selected] {
                DebugAction::NextWeather => world.debug_next_weather(),
                DebugAction::Horde => world.debug_horde(),
                DebugAction::Dawn => world.debug_dawn(),
                DebugAction::LevelUp => world.debug_level_up(),
                DebugAction::Heal => world.debug_heal(),
                DebugAction::Campfire => world.debug_campfire(),
                DebugAction::Fire => world.debug_fire(),
                DebugAction::UnlockAll => world.debug_unlock_all(),
                DebugAction::Unhurt => {
                    world.unhurt = !world.unhurt;
                    return Screen::Debug { selected };
                }
            }
            Screen::Playing
        }
        KeyCode::Esc | KeyCode::Tab => Screen::Playing,
        _ => Screen::Debug { selected },
    }
}
