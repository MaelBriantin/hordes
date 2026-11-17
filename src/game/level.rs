/// The XP from `level` to the next: 5 + 3 × (level − 1). 5 from level 1, 8 from level 2, 11
/// from level 3.
pub fn xp_to_next(level: u32) -> u32 {
    5 + 3 * level.saturating_sub(1)
}
