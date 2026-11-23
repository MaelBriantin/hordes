//! The gatling burst, the veteran's own action: a while of fire straight ahead, the way they
//! face, ten bullets a second; they may turn as it fires, but go at half their pace. Each
//! bullet a tracer, as the machine gun's; each spent case falls at their feet.

use super::TICKS_PER_SECOND;
use super::build::Weapon;
use super::guns::{self, Gun};
use super::vec2::Vec2;

/// The ticks between two bullets: ten a second.
pub const INTERVAL: u32 = 3;
/// How much slower the veteran goes while it fires, in percent of their pace.
pub const PACE_PERCENT: u32 = 50;
/// How long a spent case lies at the veteran's feet, in ticks, its fall counted.
pub const CASE_TICKS: u32 = 36;

/// The gatling burst at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gatling {
    /// The ticks it fires.
    pub burst: u32,
    /// What each bullet takes, before Might.
    pub damage: u32,
    /// How many enemies each bullet goes through before stopping at the next.
    pub pierce: u32,
    /// The ticks after it starts before it can again, before Haste.
    pub recharge: u32,
}

/// The gatling burst at a level: harder at 2, longer at 3, sooner back at 4, through more at 5.
pub fn at(level: u8) -> Gatling {
    let second = u32::from(TICKS_PER_SECOND);
    Gatling {
        burst: if level >= 3 { 3 } else { 2 } * second,
        damage: if level >= 2 { 10 } else { 8 },
        pierce: if level >= 5 { 3 } else { 1 },
        recharge: if level >= 4 { 9 } else { 12 } * second,
    }
}

/// Its bullets: the machine gun's, but for their damage and how many they go through.
pub fn gun(stats: Gatling) -> Gun {
    let base = guns::gun(Weapon::MachineGun, 1).expect("the machine gun is a gun");
    Gun {
        damage: stats.damage,
        pierce: stats.pierce,
        ..base
    }
}

/// A spent case, out of the gun, falling to the veteran's feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Case {
    /// Where it left the gun, and where it lands.
    pub from: Vec2,
    pub to: Vec2,
    pub age: u32,
}
