//! Which way the player goes: of the ways held down, the last one pressed (no diagonal). A game needs to know when a key is let go, which a terminal
//! only says with kitty's keyboard protocol (Ghostty, kitty and others). Without it, a held key
//! comes as a burst of presses: each one then counts as held for a while.

use std::time::{Duration, Instant};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::game::{Input, Vec2};

/// Without kitty's protocol, how long a first press holds: past the terminal's first repeat
/// (375 ms on macOS by default).
const FIRST_HOLD: Duration = Duration::from_millis(500);
/// Without kitty's protocol, how long a repeat holds: past the next one (90 ms on macOS by
/// default).
const REPEAT_HOLD: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hold {
    Released,
    /// Held until the terminal says it is released (kitty's protocol).
    Pressed,
    /// Held until then, unless pressed again (no protocol).
    Until(Instant),
}

impl Hold {
    fn is_held(self, now: Instant) -> bool {
        match self {
            Hold::Released => false,
            Hold::Pressed => true,
            Hold::Until(end) => now < end,
        }
    }
}

#[derive(Debug)]
pub struct Keys {
    reports_release: bool,
    /// One per `Way`, in its order.
    held: [Hold; 4],
    /// When each way was last pressed (not repeated), counting presses: the latest wins.
    pressed_at: [u64; 4],
    presses: u64,
}

impl Keys {
    /// `reports_release`: whether the terminal speaks kitty's keyboard protocol.
    pub fn new(reports_release: bool) -> Self {
        Self {
            reports_release,
            held: [Hold::Released; 4],
            pressed_at: [0; 4],
            presses: 0,
        }
    }

    pub fn handle(&mut self, key: KeyEvent, now: Instant) {
        let Some(way) = way(key.code) else {
            return;
        };
        let hold = &mut self.held[way as usize];
        // Without the protocol, a repeat is a press that comes while the key still holds.
        let repeat =
            key.kind == KeyEventKind::Repeat || (!self.reports_release && hold.is_held(now));
        *hold = match key.kind {
            KeyEventKind::Release => Hold::Released,
            _ if self.reports_release => Hold::Pressed,
            _ if repeat => Hold::Until(now + REPEAT_HOLD),
            _ => Hold::Until(now + FIRST_HOLD),
        };
        if key.kind != KeyEventKind::Release && !repeat {
            self.presses += 1;
            self.pressed_at[way as usize] = self.presses;
        }
    }

    /// Where the keys held down at `now` ask to go: the way pressed last, if any is held.
    pub fn input(&self, now: Instant) -> Input {
        let way = [Way::Up, Way::Down, Way::Left, Way::Right]
            .into_iter()
            .filter(|way| self.held[*way as usize].is_held(now))
            .max_by_key(|way| self.pressed_at[*way as usize]);
        let direction = match way {
            None => Vec2::ZERO,
            Some(Way::Up) => Vec2::new(0.0, -1.0),
            Some(Way::Down) => Vec2::new(0.0, 1.0),
            Some(Way::Left) => Vec2::new(-1.0, 0.0),
            Some(Way::Right) => Vec2::new(1.0, 0.0),
        };
        Input { direction }
    }
}

/// The way a key moves: the arrows, WASD (QWERTY) or ZQSD (AZERTY).
fn way(code: KeyCode) -> Option<Way> {
    match code {
        KeyCode::Up => Some(Way::Up),
        KeyCode::Down => Some(Way::Down),
        KeyCode::Left => Some(Way::Left),
        KeyCode::Right => Some(Way::Right),
        KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'w' | 'z' => Some(Way::Up),
            's' => Some(Way::Down),
            'a' | 'q' => Some(Way::Left),
            'd' => Some(Way::Right),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::KeyModifiers;

    use super::*;

    fn key(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind)
    }

    fn direction(keys: &Keys, now: Instant) -> (f32, f32) {
        let direction = keys.input(now).direction;
        (direction.x, direction.y)
    }

    #[test]
    fn with_the_protocol_a_key_holds_until_released() {
        let mut keys = Keys::new(true);
        let start = Instant::now();
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Press), start);
        assert_eq!(direction(&keys, start + Duration::from_secs(5)), (1.0, 0.0));
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Release), start);
        assert_eq!(direction(&keys, start), (0.0, 0.0));
    }

    #[test]
    fn without_the_protocol_a_press_holds_until_the_first_repeat() {
        let mut keys = Keys::new(false);
        let start = Instant::now();
        keys.handle(key(KeyCode::Up, KeyEventKind::Press), start);
        assert_eq!(
            direction(&keys, start + Duration::from_millis(400)),
            (0.0, -1.0)
        );
        assert_eq!(direction(&keys, start + FIRST_HOLD), (0.0, 0.0));
    }

    #[test]
    fn without_the_protocol_a_repeat_holds_a_short_while() {
        let mut keys = Keys::new(false);
        let start = Instant::now();
        let repeat = start + Duration::from_millis(400);
        keys.handle(key(KeyCode::Up, KeyEventKind::Press), start);
        keys.handle(key(KeyCode::Up, KeyEventKind::Press), repeat);
        assert_eq!(
            direction(&keys, repeat + Duration::from_millis(100)),
            (0.0, -1.0)
        );
        assert_eq!(direction(&keys, repeat + REPEAT_HOLD), (0.0, 0.0));
    }

    #[test]
    fn the_last_way_pressed_wins() {
        let mut keys = Keys::new(true);
        let now = Instant::now();
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('z'), KeyEventKind::Press), now);
        assert_eq!(direction(&keys, now), (0.0, -1.0));
        keys.handle(key(KeyCode::Left, KeyEventKind::Press), now);
        assert_eq!(direction(&keys, now), (-1.0, 0.0));
    }

    #[test]
    fn letting_go_goes_back_to_the_way_still_held() {
        let mut keys = Keys::new(true);
        let now = Instant::now();
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('w'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('w'), KeyEventKind::Release), now);
        assert_eq!(direction(&keys, now), (1.0, 0.0));
    }

    #[test]
    fn a_repeat_does_not_take_the_lead() {
        let mut keys = Keys::new(true);
        let now = Instant::now();
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('s'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Repeat), now);
        assert_eq!(direction(&keys, now), (0.0, 1.0));
    }

    #[test]
    fn without_the_protocol_a_repeat_does_not_take_the_lead_either() {
        let mut keys = Keys::new(false);
        let now = Instant::now();
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('s'), KeyEventKind::Press), now);
        keys.handle(key(KeyCode::Char('d'), KeyEventKind::Press), now);
        assert_eq!(direction(&keys, now), (0.0, 1.0));
    }
}
