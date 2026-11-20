//! The terminal side of the game: the loop and the keys. It moves the world on one tick at a time
//! with the keys held down, and has the ui draw it.

mod debug;
mod keys;
mod progress;
mod scores;
mod settings;

pub use settings::data_folder;

use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableFocusChange, EnableFocusChange, Event, KeyCode, KeyEvent, KeyModifiers,
    KeyboardEnhancementFlags, ModifierKeyCode, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::supports_keyboard_enhancement;

use crate::game::{
    Achievement, COLLECTION, Entry, Leaderboard, Mode, Progress, TICKS_PER_SECOND, WeatherNews,
    World,
};
use crate::ui::{
    self, Arrow, BloodLook, OptionItem, PauseItem, RainDisplay, Screen, Sprites, ViewState,
};
use keys::Keys;
use settings::Settings;

pub struct App {
    world: World,
    sprites: Sprites,
    screen: Screen,
    settings: Settings,
    leaderboard: Leaderboard,
    /// What the player has reached so far, every run counted.
    progress: Progress,
    /// The achievements reached and still to be told, the first being told since a frame.
    toasts: VecDeque<Achievement>,
    toast_since: Option<u32>,
    /// The collection's tile the arrows are on, on the title screen. And how many tiles a row of
    /// it holds, as the screen is wide.
    picked: usize,
    collection_row: usize,
    /// Whether the collection shows: the screen big enough for it.
    collection_shows: bool,
    /// Whether the run over has been weighed against the leaderboard.
    scored: bool,
    /// The run's place on the leaderboard, if it made it.
    highlight: Option<usize>,
    /// The letters typed last, offered again for the next high score.
    last_name: [u8; 3],
    /// The card selected when a level-up is on offer.
    selected: usize,
    /// When the cards on offer were dealt.
    dealt_at: Option<Instant>,
    /// The frames drawn so far.
    frames: u32,
    /// The run's tick the player was last pinged at: an arrow over their head a moment.
    pinged_at: Option<u32>,
    keys: Keys,
    should_quit: bool,
}

impl App {
    /// A new run on a map drawn at random.
    pub fn new(sprites: Sprites) -> Self {
        let settings = Settings::load();
        // The language first, so every text from here on is in it.
        crate::i18n::use_language(settings.language.id());
        Self {
            world: World::new(rand::random()),
            sprites,
            screen: Screen::Title,
            settings,
            leaderboard: scores::load(),
            progress: progress::load(),
            toasts: VecDeque::new(),
            toast_since: None,
            picked: 0,
            collection_row: 1,
            collection_shows: false,
            scored: false,
            highlight: None,
            last_name: *b"AAA",
            selected: 0,
            dealt_at: None,
            frames: 0,
            pinged_at: None,
            keys: Keys::new(false),
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let reports_release = supports_keyboard_enhancement().unwrap_or(false);
        if reports_release {
            execute!(io::stdout(), PushKeyboardEnhancementFlags(KEY_REPORTING))?;
        }
        // Told when the terminal loses the focus: the keys let go then never come.
        execute!(io::stdout(), EnableFocusChange)?;
        self.keys = Keys::new(reports_release);
        let result = self.game_loop(terminal);
        // The counts of the run left unfinished kept too.
        let _ = progress::save(&self.progress);
        execute!(io::stdout(), DisableFocusChange)?;
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
            self.collection_row = ui::collection_row(terminal.size()?.into());
            self.collection_shows = ui::collection_shows(terminal.size()?.into());
            let view =
                ViewState {
                    selected: self.selected,
                    shown: self.dealt_at.map_or(Duration::ZERO, |at| at.elapsed()),
                    frame: self.frames,
                    blood: self.settings.blood,
                    rain: self.settings.rain,
                    arrow: self.settings.arrow,
                    pinged: self
                        .pinged_at
                        .is_some_and(|at| self.world.ticks().saturating_sub(at) < PING_TICKS),
                    skin: self.settings.skin,
                    mode: self.settings.mode,
                    skip_dusk: self.settings.skip_dusk,
                    gear: self.settings.gear,
                    gear_list: self.settings.gear_list,
                    mini_bars: self.settings.mini_bars,
                    ground: self.settings.ground,
                    weather_news: self.settings.weather_news,
                    language: self.settings.language.name(),
                    highlight: self.highlight,
                    toast: self.toasts.front().zip(self.toast_since).map(
                        |(&achievement, since)| (achievement, self.frames.wrapping_sub(since)),
                    ),
                    picked: (self.settings.mode == Mode::Standard && self.collection_shows)
                        .then_some(self.picked),
                };
            self.frames = self.frames.wrapping_add(1);
            terminal.draw(|frame| {
                ui::render(
                    frame,
                    &self.world,
                    &self.sprites,
                    &self.leaderboard,
                    &self.progress,
                    self.screen,
                    view,
                );
            })?;
            while event::poll(next_tick.saturating_duration_since(Instant::now()))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key, Instant::now()),
                    // Away from the game, a key let go goes elsewhere: none is held any more.
                    Event::FocusLost => self.keys.release_all(),
                    _ => {}
                }
            }
            let now = Instant::now();
            // The run stands still on the title screen and in the pause menu.
            if self.screen == Screen::Playing {
                self.world.update(self.keys.input(now));
                self.note_progress();
                self.tell_achievements();
            }
            if self.world.is_over() && !self.scored {
                self.weigh_score();
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
                Screen::NameEntry { letters, cursor } => {
                    self.on_name_entry(key.code, letters, cursor);
                }
                Screen::Debug { selected } => {
                    self.screen = debug::handle_key(&mut self.world, key.code, selected);
                }
                Screen::Options {
                    selected,
                    from_pause,
                } => self.on_options(key.code, selected, from_pause),
                // Any of these keys leaves the list of keys, back to the run.
                Screen::Keys => {
                    if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter) {
                        self.screen = Screen::Playing;
                    }
                }
            }
        }
        // Held keys are followed in menus too: still held after them, they keep moving.
        self.keys.handle(key, now);
    }

    fn on_title(&mut self, code: KeyCode) {
        let collection = self.settings.mode == Mode::Standard && self.collection_shows;
        match code {
            // A new run, in the mode chosen.
            KeyCode::Enter | KeyCode::Char(' ') => self.new_run(),
            KeyCode::Tab | KeyCode::BackTab => {
                self.settings.mode = self.settings.mode.other();
                self.keep_settings();
            }
            // The options, where the game is quit too.
            KeyCode::Char('o' | 'O') | KeyCode::Esc => {
                self.screen = Screen::Options {
                    selected: 0,
                    from_pause: false,
                };
            }
            _ if collection => self.on_collection(code),
            _ => {}
        }
    }

    /// On the collection, in standard: the arrows go from tile to tile.
    fn on_collection(&mut self, code: KeyCode) {
        let (at, last, row) = (self.picked, COLLECTION.len() - 1, self.collection_row);
        self.picked = match code {
            KeyCode::Left | KeyCode::Char('a' | 'A' | 'q' | 'Q') => at.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('d' | 'D') => (at + 1).min(last),
            KeyCode::Down | KeyCode::Char('s' | 'S') if at + row <= last => at + row,
            KeyCode::Up | KeyCode::Char('w' | 'W' | 'z' | 'Z') if at >= row => at - row,
            _ => at,
        };
    }

    /// Changes how blood shows, and keeps the choice for the next runs.
    fn change_blood(&mut self, change: fn(BloodLook) -> BloodLook) {
        self.settings.blood = change(self.settings.blood);
        self.keep_settings();
    }

    /// Changes the line around the player, and keeps the choice for the next runs.
    fn change_arrow(&mut self, change: fn(Arrow) -> Arrow) {
        self.settings.arrow = change(self.settings.arrow);
        self.keep_settings();
    }

    /// Shows or hides the gear over the map: kept for the next runs.
    fn switch_gear(&mut self) {
        self.settings.gear = !self.settings.gear;
        self.keep_settings();
    }

    /// Changes how a new weather is told, from now on: kept for the next runs.
    fn change_weather_news(&mut self, change: fn(WeatherNews) -> WeatherNews) {
        self.settings.weather_news = change(self.settings.weather_news);
        self.world.weather_news = self.settings.weather_news;
        self.keep_settings();
    }

    /// Changes how much it rains, and keeps the choice for the next runs.
    fn change_rain(&mut self, change: fn(RainDisplay) -> RainDisplay) {
        self.settings.rain = change(self.settings.rain);
        self.keep_settings();
    }

    /// Speaks the other language, and keeps the choice for the next runs.
    fn change_language(&mut self) {
        self.settings.language = self.settings.language.next();
        crate::i18n::use_language(self.settings.language.id());
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
                PauseItem::Options => {
                    self.screen = Screen::Options {
                        selected: 0,
                        from_pause: true,
                    };
                }
                PauseItem::NewRun => self.new_run(),
                PauseItem::Title => self.screen = Screen::Title,
                PauseItem::Quit => self.should_quit = true,
            },
            KeyCode::Esc | KeyCode::Char('p' | 'P') => self.screen = Screen::Playing,
            _ => {}
        }
    }

    /// In the options menu: ↑↓ move, ←→ or Enter change the setting selected, Esc or Back go back
    /// where the menu was opened from.
    fn on_options(&mut self, code: KeyCode, selected: usize, from_pause: bool) {
        let items = OptionItem::items(from_pause);
        let count = items.len();
        let back = if from_pause {
            let options = PauseItem::ALL
                .iter()
                .position(|item| *item == PauseItem::Options)
                .unwrap_or(0);
            Screen::Paused { selected: options }
        } else {
            Screen::Title
        };
        let item = items[selected];
        let forward = match code {
            KeyCode::Up | KeyCode::Char('w' | 'z') => {
                self.screen = Screen::Options {
                    selected: (selected + count - 1) % count,
                    from_pause,
                };
                return;
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.screen = Screen::Options {
                    selected: (selected + 1) % count,
                    from_pause,
                };
                return;
            }
            KeyCode::Esc => {
                self.screen = back;
                return;
            }
            KeyCode::Enter | KeyCode::Char(' ') if item == OptionItem::Back => {
                self.screen = back;
                return;
            }
            KeyCode::Enter | KeyCode::Char(' ') if item == OptionItem::Quit => {
                self.should_quit = true;
                return;
            }
            KeyCode::Enter | KeyCode::Char(' ' | 'd') | KeyCode::Right => true,
            KeyCode::Left | KeyCode::Char('q' | 'a') => false,
            _ => return,
        };
        match item {
            OptionItem::Blood if forward => self.change_blood(BloodLook::next),
            OptionItem::Blood => self.change_blood(BloodLook::previous),
            OptionItem::Rain if forward => self.change_rain(RainDisplay::next),
            OptionItem::Rain => self.change_rain(RainDisplay::previous),
            OptionItem::Arrow if forward => self.change_arrow(Arrow::next),
            OptionItem::Arrow => self.change_arrow(Arrow::previous),
            OptionItem::Announcements if forward => self.change_weather_news(WeatherNews::next),
            OptionItem::Announcements => self.change_weather_news(WeatherNews::previous),
            OptionItem::Dusk => {
                self.settings.skip_dusk = !self.settings.skip_dusk;
                self.keep_settings();
            }
            OptionItem::Skin => {
                self.settings.skin = self.settings.skin.next();
                self.keep_settings();
            }
            // Hidden, as icons, as a list, and round again.
            OptionItem::Gear => {
                let settings = &mut self.settings;
                (settings.gear, settings.gear_list) = match (settings.gear, settings.gear_list) {
                    (false, _) => (true, false),
                    (true, false) => (true, true),
                    (true, true) => (false, false),
                };
                self.keep_settings();
            }
            OptionItem::Ground => {
                self.settings.ground = !self.settings.ground;
                self.keep_settings();
            }
            OptionItem::MiniBars => {
                self.settings.mini_bars = !self.settings.mini_bars;
                self.keep_settings();
            }
            OptionItem::Language => self.change_language(),
            OptionItem::Back | OptionItem::Quit => {}
        }
    }

    /// In the run: P or Esc pause it, F switches the torch, Shift dashes, or once it is over, R starts anew and
    /// Esc quits; on a level-up, the cards.
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
        if debug::opens(code) {
            self.screen = Screen::Debug { selected: 0 };
            return;
        }
        if code == KeyCode::Char('?') {
            self.screen = Screen::Keys;
            return;
        }
        if matches!(code, KeyCode::Char('i' | 'I')) {
            self.switch_gear();
            return;
        }
        if matches!(code, KeyCode::Char('e' | 'E')) {
            self.pinged_at = Some(self.world.ticks());
            return;
        }
        // F switches the torch, but for the cards of a level-up.
        if matches!(code, KeyCode::Char('f' | 'F')) && self.world.offer.is_none() {
            self.world.switch_torch();
            return;
        }
        // Shift dashes: alone with kitty's protocol; elsewhere, Shift only shows on the keys it
        // changes, a way in capitals.
        let shift = match code {
            KeyCode::Modifier(ModifierKeyCode::LeftShift | ModifierKeyCode::RightShift) => true,
            KeyCode::Char('W' | 'A' | 'S' | 'D' | 'Z' | 'Q') => !self.keys.reports_release(),
            _ => false,
        };
        if shift && self.world.offer.is_none() {
            self.world.dash();
            return;
        }
        self.choose(code);
    }

    /// Takes in what the run has reached: the achievements new, to be told and kept at once; the
    /// counts, kept as the run ends.
    fn note_progress(&mut self) {
        if self.world.progress == self.progress {
            return;
        }
        let new: Vec<Achievement> = self
            .world
            .progress
            .achieved
            .difference(&self.progress.achieved)
            .copied()
            .collect();
        self.progress = self.world.progress.clone();
        if !new.is_empty() {
            self.toasts.extend(new);
            let _ = progress::save(&self.progress);
        }
    }

    /// The achievements told one after the other, each its time on screen.
    fn tell_achievements(&mut self) {
        match self.toast_since {
            None if !self.toasts.is_empty() => self.toast_since = Some(self.frames),
            Some(since) if self.frames.wrapping_sub(since) >= ui::TOAST_FRAMES => {
                self.toasts.pop_front();
                self.toast_since = None;
            }
            _ => {}
        }
    }

    /// The run is over: if its score makes the leaderboard, three letters to type in.
    fn weigh_score(&mut self) {
        self.scored = true;
        let _ = progress::save(&self.progress);
        // A standard run keeps no score.
        if self.world.mode == Mode::Arcade && self.leaderboard.qualifies(self.world.score) {
            self.screen = Screen::NameEntry {
                letters: self.last_name,
                cursor: 0,
            };
        }
    }

    /// Typing a name in, arcade style: ↑↓ turn the letter, ←→ move, a letter key types it and
    /// moves on, Enter keeps the run on the leaderboard, Esc leaves it off.
    fn on_name_entry(&mut self, code: KeyCode, mut letters: [u8; 3], mut cursor: usize) {
        let turn = |letter: u8, by: i16| {
            let index = (i16::from(letter) - i16::from(b'A') + by).rem_euclid(26);
            b'A' + u8::try_from(index).unwrap_or(0)
        };
        match code {
            KeyCode::Up => letters[cursor] = turn(letters[cursor], 1),
            KeyCode::Down => letters[cursor] = turn(letters[cursor], -1),
            KeyCode::Left | KeyCode::Backspace => cursor = cursor.saturating_sub(1),
            KeyCode::Right => cursor = (cursor + 1).min(2),
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                letters[cursor] = u8::try_from(c.to_ascii_uppercase()).unwrap_or(b'A');
                cursor = (cursor + 1).min(2);
            }
            KeyCode::Enter => {
                self.last_name = letters;
                let entry = Entry {
                    name: String::from_utf8_lossy(&letters).into_owned(),
                    score: self.world.score,
                    seconds: self.world.seconds(),
                    level: self.world.player.level,
                };
                self.highlight = self.leaderboard.insert(entry);
                // Not kept, the run still shows on the leaderboard this session.
                let _ = scores::save(&self.leaderboard);
                self.screen = Screen::Playing;
                return;
            }
            KeyCode::Esc => {
                self.screen = Screen::Playing;
                return;
            }
            _ => {}
        }
        self.screen = Screen::NameEntry { letters, cursor };
    }

    /// A new run on a new map, started at once.
    fn new_run(&mut self) {
        self.scored = false;
        self.highlight = None;
        self.world = World::with(
            rand::random(),
            self.settings.mode,
            self.settings.skip_dusk,
            self.progress.clone(),
        );
        self.world.weather_news = self.settings.weather_news;
        self.selected = 0;
        self.dealt_at = None;
        self.pinged_at = None;
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
            // Dealt again, if the run has a reroll left.
            KeyCode::Char('r' | 'R') if self.world.rerolls > 0 => {
                self.world.reroll();
                self.selected = 0;
                self.dealt_at = Some(Instant::now());
                None
            }
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

/// How long the arrow shows over the player once pinged, in ticks: 2 s.
const PING_TICKS: u32 = 2 * TICKS_PER_SECOND as u32;

/// What the app asks of terminals with kitty's keyboard protocol (Ghostty, kitty, WezTerm…):
/// every key as a code, with whether it is pressed, held down (repeated) or released.
const KEY_REPORTING: KeyboardEnhancementFlags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    .union(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    .union(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
    // The shifted key too: `?`, not `/` with Shift, whatever the keyboard's layout.
    .union(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS);
