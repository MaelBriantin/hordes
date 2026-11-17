//! The terminal side of the game: the loop and the keys. It moves the world on one tick at a time
//! with the keys held down, and has the ui draw it.

mod keys;

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
use crate::ui::{self, LevelUpView, Sprites};
use keys::Keys;

pub struct App {
    world: World,
    sprites: Sprites,
    /// The card selected when a level-up is on offer.
    selected: usize,
    /// When the cards on offer were dealt.
    dealt_at: Option<Instant>,
    keys: Keys,
    should_quit: bool,
}

impl App {
    /// A new run on a map drawn at random.
    pub fn new(sprites: Sprites) -> Self {
        Self {
            world: World::new(rand::random()),
            sprites,
            selected: 0,
            dealt_at: None,
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
            let level_up = LevelUpView {
                selected: self.selected,
                shown: self.dealt_at.map_or(Duration::ZERO, |at| at.elapsed()),
            };
            terminal.draw(|frame| ui::render(frame, &self.world, &self.sprites, level_up))?;
            while event::poll(next_tick.saturating_duration_since(Instant::now()))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key, Instant::now());
                }
            }
            let now = Instant::now();
            self.world.update(self.keys.input(now));
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
        if key.is_press() && (key.code == KeyCode::Esc || ctrl_c) {
            self.should_quit = true;
            return;
        }
        let restart = matches!(key.code, KeyCode::Char('r' | 'R'));
        if self.world.is_over() && key.is_press() && restart {
            self.world = World::new(rand::random());
            self.selected = 0;
            self.dealt_at = None;
            return;
        }
        if key.is_press() {
            self.choose(key.code);
        }
        // Held keys are followed on a level-up too: still held after it, they keep moving.
        self.keys.handle(key, now);
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
