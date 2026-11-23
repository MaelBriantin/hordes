//! The first-aid kit, the medic's own action: HP back at once, its numbers the medic's (the
//! Healer perk not counted again); then it waits its recharge.

use super::TICKS_PER_SECOND;

/// The first-aid kit at a level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kit {
    /// The HP it gives back.
    pub heal: u32,
    /// The ticks after it is used before it can be again, before Haste.
    pub recharge: u32,
}

/// The first-aid kit at a level: more HP at 2 and 4, sooner back at 3 and 5. Dosed: the
/// game's survival rests on healing being scarce.
pub fn at(level: u8) -> Kit {
    let second = u32::from(TICKS_PER_SECOND);
    Kit {
        heal: match level {
            0 | 1 => 15,
            2 | 3 => 20,
            _ => 25,
        },
        recharge: match level {
            0..=2 => 20,
            3 | 4 => 15,
            _ => 12,
        } * second,
    }
}
