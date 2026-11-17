//! The terminal side of the game: the loop and the keys. It moves the world on one tick at a time
//! with the keys held down, and has the ui draw it.

mod keys;
mod settings;

use std::io;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::supports_keyboard_enhancement;

use crate::game::{TICKS_PER_SECOND, World};
use crate::ui::{self, PauseItem, RainDisplay, Screen, Sprites, ViewState};
use keys::Keys;
use settings::Settings;

pub struct App {
    world: World,
    sprites: Sprites,
    screen: Screen,
    settings: Settings,
    /// The card selected when a level-up is on offer.
    selected: usize,
    /// When the cards on offer were dealt.
    dealt_at: Option<Instant>,
    /// The frames drawn so far.
    frames: u32,
    keys: Keys,
    should_quit: bool,
}

impl App {
    /// A new run on a map drawn at random.
    pub fn new(sprites: Sprites) -> Self {
        Self {
            world: World::new(rand::random()),
            sprites,
            screen: Screen::Title,
            settings: Settings::load(),
            selected: 0,
            dealt_at: None,
            frames: 0,
            keys: Keys::new(false),
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let reports_release = supports_keyboard_enhancement().unwrap_or(false);
        if reports_release {
            execute!(io::stdout(), PushKeyboardEnhancementFlags(KEY_REPORTING))?;
        }
        self.keys = Keys::new(reports_release);
        let result = self.game_loop(terminal);
        if reports_release {
            execute!(io::stdout(), PopKeyboardEnhancementFlags)?;
        }
        result
    }

    /// Draws, takes the keys until the next tick is due, moves the world on, again.
    fn game_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick = Duration::from_secs(1) / u32::from(TICKS_PER_SECOND);
        let mut next_tick = Instant::now() + tick;
        while !self.should_quit {
            self.world.set_view(ui::view(terminal.size()?.into()));
            let view = ViewState {
                selected: self.selected,
                shown: self.dealt_at.map_or(Duration::ZERO, |at| at.elapsed()),
                frame: self.frames,
                blood: self.settings.blood,
                rain: self.settings.rain,
            };
            self.frames = self.frames.wrapping_add(1);
            terminal.draw(|frame| {
                ui::render(frame, &self.world, &self.sprites, self.screen, view);
            })?;
            while event::poll(next_tick.saturating_duration_since(Instant::now()))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key, Instant::now());
                }
            }
            let now = Instant::now();
            // The run stands still on the title screen and in the pause menu.
            if self.screen == Screen::Playing {
                self.world.update(self.keys.input(now));
            }
            if self.world.offer.is_some() && self.dealt_at.is_none() {
                self.dealt_at = Some(now);
            }
            next_tick += tick;
            // Running late (a slow frame, the process suspended): go on from now rather than
            // play the missed ticks in a burst.
            if next_tick < now {
                next_tick = now + tick;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, now: Instant) {
        let ctrl_c =
            key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c');
        if key.is_press() && ctrl_c {
            self.should_quit = true;
            return;
        }
        if key.is_press() {
            match self.screen {
                Screen::Title => self.on_title(key.code),
                Screen::Paused { selected } => self.on_pause(key.code, selected),
                Screen::Playing => self.on_run(key.code),
            }
        }
        // Held keys are followed in menus too: still held after them, they keep moving.
        self.keys.handle(key, now);
    }

    fn on_title(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter | KeyCode::Char(' ') => self.screen = Screen::Playing,
            KeyCode::Char('b' | 'B') => self.toggle_blood(),
            KeyCode::Char('r' | 'R') => self.change_rain(RainDisplay::next),
            KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    /// Shows blood or hides it, and keeps the choice for the next runs.
    fn toggle_blood(&mut self) {
        self.settings.blood = !self.settings.blood;
        self.keep_settings();
    }

    /// Changes how much it rains, and keeps the choice for the next runs.
    fn change_rain(&mut self, change: fn(RainDisplay) -> RainDisplay) {
        self.settings.rain = change(self.settings.rain);
        self.keep_settings();
    }

    fn keep_settings(&self) {
        // Not kept, the choice still holds for this session: nothing worth stopping the game for.
        let _ = self.settings.save();
    }

    /// In the pause menu: ↑↓ move, Enter picks, P or Esc go back to the run.
    fn on_pause(&mut self, code: KeyCode, selected: usize) {
        let count = PauseItem::ALL.len();
        match code {
            KeyCode::Up | KeyCode::Char('w' | 'z') => {
                self.screen = Screen::Paused {
                    selected: (selected + count - 1) % count,
                };
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.screen = Screen::Paused {
                    selected: (selected + 1) % count,
                };
            }
            KeyCode::Enter | KeyCode::Char(' ') => match PauseItem::ALL[selected] {
                PauseItem::Resume => self.screen = Screen::Playing,
                PauseItem::Blood => self.toggle_blood(),
                PauseItem::Rain => self.change_rain(RainDisplay::next),
                PauseItem::NewRun => self.new_run(),
                PauseItem::Quit => self.should_quit = true,
            },
            KeyCode::Left | KeyCode::Right | KeyCode::Char('q' | 'a' | 'd')
                if PauseItem::ALL[selected] == PauseItem::Blood =>
            {
                self.toggle_blood();
            }
            KeyCode::Left | KeyCode::Char('q' | 'a')
                if PauseItem::ALL[selected] == PauseItem::Rain =>
            {
                self.change_rain(RainDisplay::previous);
            }
            KeyCode::Right | KeyCode::Char('d') if PauseItem::ALL[selected] == PauseItem::Rain => {
                self.change_rain(RainDisplay::next);
            }
            KeyCode::Esc | KeyCode::Char('p' | 'P') => self.screen = Screen::Playing,
            _ => {}
        }
    }

    /// In the run: P or Esc pause it, or once it is over, R starts anew and Esc quits; on a
    /// level-up, the cards.
    fn on_run(&mut self, code: KeyCode) {
        if self.world.is_over() {
            match code {
                KeyCode::Char('r' | 'R') => self.new_run(),
                KeyCode::Esc => self.should_quit = true,
                _ => {}
            }
            return;
        }
        if matches!(code, KeyCode::Esc | KeyCode::Char('p' | 'P')) {
            self.screen = Screen::Paused { selected: 0 };
            return;
        }
        self.choose(code);
    }

    /// A new run on a new map, started at once.
    fn new_run(&mut self) {
        self.world = World::new(rand::random());
        self.selected = 0;
        self.dealt_at = None;
        self.screen = Screen::Playing;
    }

    /// On a level-up, the keys move the selection and take a card.
    fn choose(&mut self, code: KeyCode) {
        let Some(count) = self.world.offer.as_ref().map(Vec::len) else {
            return;
        };
        let take = match code {
            KeyCode::Left | KeyCode::Char('q' | 'a') => {
                self.selected = (self.selected + count - 1) % count;
                None
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.selected = (self.selected + 1) % count;
                None
            }
            KeyCode::Char(digit @ '1'..='9') => digit
                .to_digit(10)
                .and_then(|digit| usize::try_from(digit).ok())
                .map(|digit| digit - 1)
                .filter(|index| *index < count),
            KeyCode::Enter | KeyCode::Char(' ') => Some(self.selected),
            _ => None,
        };
        if let Some(index) = take {
            self.world.choose(index);
            self.selected = 0;
            // The next level's cards, if any, are dealt anew.
            self.dealt_at = self.world.offer.is_some().then(Instant::now);
        }
    }
}

/// What the app asks of terminals with kitty's keyboard protocol (Ghostty, kitty, WezTerm…):
/// every key as a code, with whether it is pressed, held down (repeated) or released.
const KEY_REPORTING: KeyboardEnhancementFlags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    .union(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    .union(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES);
