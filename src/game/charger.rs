//! The charger: a stocky zombie in a football player's pads and helmet. Having found the player,
//! near enough, it stops and scrapes the ground a second, then charges straight at where they
//! stood, fast, turning no more: on them, it hurts them and throws them back; a wooden barricade
//! in its way breaks; any other obstacle stops it, stunned a while. After a charge, it walks a
//! while before it scrapes again.

use super::Vec2;

/// How near the player it starts to scrape, in tiles.
pub const REACH: f32 = 8.0;
/// The ticks it scrapes the ground: 1 s.
pub const SCRAPE_TICKS: u32 = 30;
/// Its pace charging, against its speed: 12 tiles a second; the ticks its charge lasts at most,
/// some 10 tiles.
pub const CHARGE_PACE: f32 = 6.0;
pub const CHARGE_TICKS: u32 = 25;
/// What its charge costs the player, in HP, and how far it throws them, in tiles.
pub const DAMAGE: u32 = 20;
pub const PUSH: f32 = 2.0;
/// How far ahead of its body, charging, it finds what it runs into, in tiles.
pub const AHEAD: f32 = 0.15;
/// The ticks it stands stunned, run into an obstacle: 2 s.
pub const STUN_TICKS: u32 = 60;
/// The ticks it walks after a charge before it may scrape again: 3 s.
pub const REST_TICKS: u32 = 90;

/// What a charger is about.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Charge {
    /// Walking, ready to scrape the player near.
    #[default]
    Ready,
    /// Scraping the ground, the ticks left before it charges.
    Scraping(u32),
    /// Charging the `way` it set off (length 1), the ticks left.
    Charging { way: Vec2, left: u32 },
    /// Walking after a charge, the ticks left before it is ready again.
    Resting(u32),
}

impl Charge {
    /// Whether it is about to strike or striking: scraping or charging. It shows in the dark.
    pub fn showing(self) -> bool {
        matches!(self, Charge::Scraping(_) | Charge::Charging { .. })
    }
}
