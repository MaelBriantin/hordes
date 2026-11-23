//! Revival, a legendary unlocked by Close call: down to 0 HP, the hero comes back once, half
//! their HP back, untouchable a while, a wave throwing the enemies around back; their soul out
//! of their body, up, then back into it.

use super::TICKS_PER_SECOND;

/// The ticks the soul takes, out of the body and back into it; then the ring and the crosses.
pub const SOUL_TICKS: u32 = 12;
/// How long the whole of it shows, in ticks.
pub const SHOW_TICKS: u32 = SOUL_TICKS + 12;
/// The share of their HP the hero comes back with, in percent.
pub const PERCENT: u32 = 50;
/// The wave: how far it reaches, how far it throws, in tiles, over how many ticks.
pub const WAVE_REACH: f32 = 4.0;
pub const WAVE_TILES: f32 = 3.0;
pub const WAVE_TICKS: u32 = 8;

/// The ticks the hero is untouchable, come back: 2 s.
pub fn untouchable() -> u32 {
    2 * u32::from(TICKS_PER_SECOND)
}
