//! Hordes: once a stage of weather, in its middle, a wave of enemies comes all at once, announced
//! a moment before. A pack
//! out of one side at first; from the storm on, a ring closing in from every side. Wiped out to
//! the last, a horde doubles the points its kills were worth.

use rand::{Rng, RngExt};

use super::enemy::EnemyKind;
use super::mode::Mode;
use super::spawn::MARGIN;
use super::weather::{self, Weather};
use super::{Facing, TICKS_PER_SECOND, Vec2};

/// The seconds a horde is announced before it comes.
const WARNING_SECONDS: u32 = 3;
/// The enemies of the first horde, and how many more each one after brings.
const FIRST_SIZE: u32 = 30;
const GROWTH: u32 = 10;
/// How wide and how deep a pack is, in tiles: it comes in a band this wide, this deep.
const PACK_WIDTH: f32 = 16.0;
const PACK_DEPTH: f32 = 5.0;

/// A horde on its way: announced, it comes at `arrives_at`, out of `side`, or out of every side
/// when it closes in as a ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Warning {
    pub arrives_at: u32,
    pub side: Option<Facing>,
    /// Which horde it is, from 0.
    pub number: u32,
}

/// The share of a horde, in percent, its very last ones: the veil pulses on them.
pub const LAST_PERCENT: u32 = 10;

/// What became of a horde come: how many of its enemies came, how many still stand, and the
/// points its kills have been worth so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    pub size: u32,
    pub left: u32,
    pub points: u64,
}

impl Tally {
    /// Whether those still standing are its last `percent` of it, the last one at least.
    pub fn last(&self, percent: u32) -> bool {
        self.left <= (self.size * percent / 100).max(1)
    }
}

/// The side a horde comes out of: the other side from the enemies about the player, `view`
/// around them, to catch the player between; at random while none is about.
pub fn pincer(
    rng: &mut impl Rng,
    player: Vec2,
    view: Vec2,
    enemies: impl Iterator<Item = Vec2>,
) -> Facing {
    let (reach_x, reach_y) = (view.x + MARGIN, view.y + MARGIN);
    // Where the enemies about lie, all together, each side as wide as the view is.
    let mut lean = Vec2::ZERO;
    for enemy in enemies {
        let apart = enemy - player;
        if apart.x.abs() <= reach_x && apart.y.abs() <= reach_y {
            lean += Vec2::new(apart.x / reach_x, apart.y / reach_y);
        }
    }
    if lean.x == 0.0 && lean.y == 0.0 {
        return [Facing::Up, Facing::Down, Facing::Left, Facing::Right][rng.random_range(0..4)];
    }
    if lean.x.abs() >= lean.y.abs() {
        if lean.x > 0.0 {
            Facing::Left
        } else {
            Facing::Right
        }
    } else if lean.y > 0.0 {
        Facing::Up
    } else {
        Facing::Down
    }
}

/// The tick the first horde comes at: in the middle of the first night, once it has fallen.
pub fn first(mode: Mode) -> u32 {
    (weather::DUSK_SECONDS + weather::stage_seconds(mode) / 2) * u32::from(TICKS_PER_SECOND)
}

/// The tick the first horde due after `ticks` comes at, by the clock: one in the middle of each
/// stage of weather.
pub fn due_after(ticks: u32, mode: Mode) -> u32 {
    let (first, every) = (first(mode), every(mode));
    if ticks < first {
        first
    } else {
        first + ((ticks - first) / every + 1) * every
    }
}

/// The ticks between two hordes: a stage of weather.
pub fn every(mode: Mode) -> u32 {
    weather::stage_seconds(mode) * u32::from(TICKS_PER_SECOND)
}

/// The ticks a horde is announced before it comes.
pub fn warning() -> u32 {
    WARNING_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// How many enemies the horde `number` (from 0) brings.
pub fn size(number: u32) -> u32 {
    FIRST_SIZE + GROWTH * number
}

/// Whether the horde coming in that weather closes in from every side: from the storm on.
pub fn rings(weather: Weather) -> bool {
    weather >= Weather::Storm
}

/// The kind of one of a horde's enemies, in that weather: zombies; a Shadow in ten from the rain
/// on; an abomination in ten in the tempest.
pub fn kind(rng: &mut impl Rng, weather: Weather) -> EnemyKind {
    let roll = rng.random_range(0..100);
    if weather == Weather::Tempest && roll < 10 {
        EnemyKind::Abomination
    } else if weather >= Weather::Rain && roll >= 90 {
        EnemyKind::Shadow
    } else {
        EnemyKind::Zombie
    }
}

/// Where the enemies of a horde come from, just beyond what the player sees (`view`: half its
/// size around `centre`): a dense pack in a band out of `side`, or, without one, spread round
/// every side in turn.
pub fn spots(
    rng: &mut impl Rng,
    centre: Vec2,
    view: Vec2,
    side: Option<Facing>,
    count: u32,
) -> Vec<Vec2> {
    const SIDES: [Facing; 4] = [Facing::Up, Facing::Right, Facing::Down, Facing::Left];
    let (reach_x, reach_y) = (view.x + MARGIN, view.y + MARGIN);
    (0..count)
        .map(|index| {
            // A pack keeps to a band around the middle of its side; a ring takes every side whole.
            let (side, spread_x, spread_y) = match side {
                Some(side) => (side, PACK_WIDTH / 2.0, PACK_WIDTH / 2.0),
                None => (
                    SIDES[usize::try_from(index % 4).unwrap_or(0)],
                    reach_x,
                    reach_y,
                ),
            };
            let along_x = rng.random_range(-spread_x..=spread_x);
            let along_y = rng.random_range(-spread_y..=spread_y);
            let deeper = rng.random_range(0.0..=PACK_DEPTH);
            let offset = match side {
                Facing::Up => Vec2::new(along_x, -reach_y - deeper),
                Facing::Down => Vec2::new(along_x, reach_y + deeper),
                Facing::Left => Vec2::new(-reach_x - deeper, along_y),
                Facing::Right => Vec2::new(reach_x + deeper, along_y),
            };
            centre + offset
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    #[test]
    fn the_next_horde_is_due_in_the_middle_of_a_stage() {
        let second = u32::from(TICKS_PER_SECOND);
        assert_eq!(due_after(0, Mode::Standard), 90 * second);
        assert_eq!(due_after(90 * second, Mode::Standard), 210 * second);
        assert_eq!(due_after(329 * second, Mode::Standard), 330 * second);
        assert_eq!(due_after(0, Mode::Arcade), 120 * second);
    }

    #[test]
    fn hordes_grow_one_after_the_other() {
        assert_eq!([0, 1, 4].map(size), [30, 40, 70]);
    }

    #[test]
    fn a_pack_comes_in_a_band_out_of_its_side() {
        let mut rng = StdRng::seed_from_u64(1);
        let (centre, view) = (Vec2::new(0.0, 0.0), Vec2::new(30.0, 12.0));
        for spot in spots(&mut rng, centre, view, Some(Facing::Right), 50) {
            assert!(spot.x >= view.x + MARGIN && spot.x <= view.x + MARGIN + PACK_DEPTH);
            assert!(spot.y.abs() <= PACK_WIDTH / 2.0);
        }
    }

    #[test]
    fn a_ring_comes_out_of_every_side() {
        let mut rng = StdRng::seed_from_u64(2);
        let spots = spots(&mut rng, Vec2::ZERO, Vec2::new(30.0, 12.0), None, 40);
        assert!(spots.iter().any(|spot| spot.x > 30.0));
        assert!(spots.iter().any(|spot| spot.x < -30.0));
        assert!(spots.iter().any(|spot| spot.y > 12.0));
        assert!(spots.iter().any(|spot| spot.y < -12.0));
    }

    #[test]
    fn a_horde_comes_from_the_other_side_than_the_enemies_about() {
        let mut rng = StdRng::seed_from_u64(1);
        let view = Vec2::new(30.0, 12.0);
        let below = [Vec2::new(2.0, 8.0), Vec2::new(-3.0, 10.0)];
        assert_eq!(
            pincer(&mut rng, Vec2::ZERO, view, below.into_iter()),
            Facing::Up
        );
        let right = [Vec2::new(20.0, 1.0), Vec2::new(25.0, -2.0)];
        assert_eq!(
            pincer(&mut rng, Vec2::ZERO, view, right.into_iter()),
            Facing::Left
        );
        // Far off, they count for nothing.
        let far = [Vec2::new(500.0, 0.0)];
        let side = pincer(&mut rng, Vec2::ZERO, view, far.into_iter());
        assert!([Facing::Up, Facing::Down, Facing::Left, Facing::Right].contains(&side));
    }

    #[test]
    fn the_last_tenth_of_a_horde_or_its_last_one_are_its_last_ones() {
        let tally = |size, left| Tally {
            size,
            left,
            points: 0,
        };
        assert!(!tally(30, 4).last(LAST_PERCENT));
        assert!(tally(30, 3).last(LAST_PERCENT));
        assert!(!tally(5, 2).last(LAST_PERCENT));
        assert!(tally(5, 1).last(LAST_PERCENT));
        assert!(tally(30, 30).last(100));
    }

    #[test]
    fn from_the_storm_on_a_horde_closes_in_as_a_ring() {
        assert!(!rings(Weather::Rain));
        assert!(rings(Weather::Storm));
    }
}
