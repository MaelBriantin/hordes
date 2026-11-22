//! The napalm strike, from the Legend achievement: now and then, a carpet of fire bombs falling
//! in a line over the biggest crowd near the player, one after the other; each bomb hurts and
//! sets on fire what is around where it lands, and leaves the ground burning a while. Two lines
//! crossed at the last level.

use super::{TICKS_PER_SECOND, Vec2};

/// How near the player the crowd struck has to be, in tiles.
pub const RANGE: f32 = 8.0;
/// How near one another enemies are to count as a crowd, in tiles.
const CROWD: f32 = 2.0;
/// The ticks a bomb falls, its streak showing, before it lands; the ticks between two bombs.
pub const FALL_TICKS: u32 = 8;
pub const BOMB_EVERY: u32 = 3;
/// How far around where it lands a bomb hurts, in tiles; how far its fire spreads on the
/// ground, and how long it burns.
pub const BOMB_RADIUS: f32 = 1.25;
pub const FIRE_RADIUS: f32 = 0.9;
const FIRE_SECONDS: u32 = 2;

/// The napalm at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Napalm {
    /// The ticks between two strikes, before Haste.
    pub every: u32,
    /// What each bomb takes from each enemy around where it lands, before Might.
    pub damage: u32,
    /// How long a line of bombs is, in tiles: a bomb a tile.
    pub length: u8,
    /// Whether a second line crosses the first.
    pub crossed: bool,
}

/// The napalm at a level.
pub fn at(level: u8) -> Napalm {
    let second = u32::from(TICKS_PER_SECOND);
    Napalm {
        every: if level >= 4 { 15 } else { 20 } * second,
        damage: if level >= 3 { 45 } else { 30 },
        length: if level >= 2 { 7 } else { 5 },
        crossed: level >= 5,
    }
}

/// The ticks the ground burns where a bomb landed.
pub fn fire_lasts() -> u32 {
    FIRE_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// A bomb falling, and the ticks left before it lands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bomb {
    pub at: Vec2,
    pub left: u32,
}

/// Where the crowd struck stands among `enemies`, from `player`: the enemy within reach with
/// the most others near it, the nearest of those; none within reach, none.
pub fn crowd(player: Vec2, enemies: &[Vec2]) -> Option<Vec2> {
    let near = |a: Vec2, b: Vec2| (a - b).length() <= CROWD;
    enemies
        .iter()
        .filter(|at| (**at - player).length() <= RANGE)
        .map(|at| {
            let others = enemies.iter().filter(|other| near(*at, **other)).count();
            (others, *at)
        })
        .max_by(|(a, at_a), (b, at_b)| {
            a.cmp(b).then(
                (*at_b - player)
                    .length()
                    .total_cmp(&(*at_a - player).length()),
            )
        })
        .map(|(_, at)| at)
}

/// The bombs of a strike on `target` from `player`, in the order they fall: a line `length`
/// tiles long through the target, the way from the player to it, from the player's side; a
/// second across it, `crossed`.
pub fn strike(player: Vec2, target: Vec2, length: u8, crossed: bool) -> Vec<Bomb> {
    let way = match (target - player).normalized() {
        Vec2::ZERO => Vec2::new(1.0, 0.0),
        way => way,
    };
    let across = Vec2::new(-way.y, way.x);
    let half = f32::from(length - 1) / 2.0;
    let lines = if crossed {
        vec![way, across]
    } else {
        vec![way]
    };
    lines
        .into_iter()
        .flat_map(|line| {
            (0..length).map(move |bomb| (bomb, target + line * (f32::from(bomb) - half)))
        })
        .map(|(bomb, at)| Bomb {
            at,
            left: FALL_TICKS + u32::from(bomb) * BOMB_EVERY,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_napalm_grows_with_its_levels() {
        let (first, last) = (at(1), at(5));
        assert_eq!(
            (first.every, first.damage, first.length, first.crossed),
            (600, 30, 5, false)
        );
        assert_eq!(
            (last.every, last.damage, last.length, last.crossed),
            (450, 45, 7, true)
        );
    }

    #[test]
    fn it_strikes_the_biggest_crowd_within_reach() {
        let player = Vec2::new(0.0, 0.0);
        let enemies = [
            Vec2::new(2.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(5.5, 5.0),
            Vec2::new(5.0, 5.5),
            Vec2::new(20.0, 0.0),
            Vec2::new(20.5, 0.0),
            Vec2::new(20.0, 0.5),
            Vec2::new(20.5, 0.5),
        ];
        assert_eq!(crowd(player, &enemies), Some(Vec2::new(5.0, 5.0)));
        assert_eq!(crowd(player, &enemies[4..]), None, "out of reach");
    }

    #[test]
    fn its_bombs_fall_in_a_line_through_the_target_from_the_player_s_side() {
        let bombs = strike(Vec2::new(0.0, 0.0), Vec2::new(6.0, 0.0), 5, false);
        let places: Vec<f32> = bombs.iter().map(|bomb| bomb.at.x).collect();
        assert_eq!(places, [4.0, 5.0, 6.0, 7.0, 8.0]);
        assert!(bombs.iter().all(|bomb| bomb.at.y == 0.0));
        assert!(bombs.windows(2).all(|two| two[0].left < two[1].left));
        let crossed = strike(Vec2::new(0.0, 0.0), Vec2::new(6.0, 0.0), 5, true);
        assert_eq!(crossed.len(), 10);
        assert!(
            crossed[5..]
                .iter()
                .all(|bomb| (bomb.at.x - 6.0).abs() < 1e-5)
        );
    }
}
