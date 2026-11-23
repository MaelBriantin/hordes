//! Revival, a rare passive unlocked by Close call: down to 0 HP, the hero comes back once, part
//! of their HP back, untouchable a while; their soul out of their body, up, then back into it.
//! At its level 4, a wave throws the enemies around back as they come back; at its last, twice.

use super::TICKS_PER_SECOND;

/// The ticks the soul takes, out of the body and back into it; then the ring and the crosses.
pub const SOUL_TICKS: u32 = 12;
/// How long the whole of it shows, in ticks.
pub const SHOW_TICKS: u32 = SOUL_TICKS + 12;
/// The wave at level 4: how far it reaches, how far it throws, in tiles, over how many ticks.
pub const WAVE_REACH: f32 = 4.0;
pub const WAVE_TILES: f32 = 3.0;
pub const WAVE_TICKS: u32 = 8;

/// The ticks the hero is untouchable, come back: 2 s.
pub fn untouchable() -> u32 {
    2 * u32::from(TICKS_PER_SECOND)
}

/// The share of their HP the hero comes back with, in percent, by Revival's level.
pub fn percent(level: u8) -> u32 {
    match level {
        0 | 1 => 25,
        2 => 35,
        _ => 50,
    }
}

/// How many times a run the hero comes back, by Revival's level.
pub fn times(level: u8) -> u8 {
    match level {
        0 => 0,
        1..=4 => 1,
        _ => 2,
    }
}
