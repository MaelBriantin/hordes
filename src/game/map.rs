//! The map: endless, its obstacles drawn chunk by chunk the first time something looks there.
//! Each chunk's obstacles come from the run's seed and the chunk's place alone, so a chunk is
//! the same whenever it is drawn. Obstacles stand on a grid of their own, each covering 2 × 2
//! tiles, gathered in clumps: groves and heaps of rocks. On the free ground, now and then a herb,
//! and more rarely still a campfire in a clearing.

use std::cell::RefCell;
use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// The tiles an obstacle covers, each way.
pub const OBSTACLE_SIZE: i32 = 2;
/// A chunk's size, in places of the obstacle grid, each way: 32 tiles.
const CHUNK_SIZE: i32 = 16;
/// The share of a chunk obstacles cover, in percent.
const COVER_PERCENT: i32 = 8;
/// How many obstacles a clump tries to place.
const CLUMP_SIZE: std::ops::RangeInclusive<u32> = 2..=8;
/// At most this many clumps per chunk, should its places be taken (near the start).
const MAX_CLUMPS: u32 = 40;
/// The chance a clump is a grove rather than rocks.
const GROVE_CHANCE: f64 = 0.7;
/// No obstacle this close to where the player starts, in tiles.
const CLEAR_RADIUS: i32 = 8;
/// The chance a chunk holds a herb, and a campfire, in percent: about one chunk in 33, and one
/// in 100.
const HERB_PERCENT: u32 = 3;
const CAMPFIRE_PERCENT: u32 = 1;
/// How many tiles around a campfire are free of obstacles, each way: its clearing.
const CLEARING: i32 = 2;
/// How many spots a chunk tries for its herb and its campfire.
const FEATURE_TRIES: u32 = 40;

/// What lies on the ground, besides obstacles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Heals when picked up.
    Herb,
    /// Heals while the player stays by it, until it dies out.
    Campfire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Obstacle {
    Tree,
    Rock,
}

/// A chunk: its obstacle grid's places, row by row, and what lies on its free ground, by tile.
#[derive(Debug, Default)]
struct Chunk {
    obstacles: Vec<Option<Obstacle>>,
    features: Vec<((i32, i32), Feature)>,
}

#[derive(Debug)]
pub struct Map {
    /// Where the obstacles come from; none without it.
    seed: Option<u64>,
    /// The tile the player starts on: no obstacle near it.
    start: (i32, i32),
    /// The chunks drawn so far, by their place.
    chunks: RefCell<HashMap<(i32, i32), Chunk>>,
}

impl Map {
    /// An endless map drawn from `seed`, clear around `start`, the tile the player starts on.
    pub fn new(seed: u64, start: (i32, i32)) -> Self {
        Self {
            seed: Some(seed),
            start,
            chunks: RefCell::default(),
        }
    }

    /// An endless map with no obstacle.
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            seed: None,
            start: (0, 0),
            chunks: RefCell::default(),
        }
    }

    /// The obstacle covering a tile, if any.
    pub fn obstacle_at(&self, tile: (i32, i32)) -> Option<Obstacle> {
        let (chunk, index) = locate(tile);
        let mut chunks = self.chunks.borrow_mut();
        chunks
            .entry(chunk)
            .or_insert_with(|| self.draw_chunk(chunk))
            .obstacles[index]
    }

    /// What lies on the ground between two tiles (both included), by tile.
    pub fn features_in(&self, from: (i32, i32), to: (i32, i32)) -> Vec<((i32, i32), Feature)> {
        let chunk_of = |tile: (i32, i32)| locate(tile).0;
        let (first, last) = (chunk_of(from), chunk_of(to));
        let mut chunks = self.chunks.borrow_mut();
        let mut found = Vec::new();
        for cy in first.1..=last.1 {
            for cx in first.0..=last.0 {
                let chunk = chunks
                    .entry((cx, cy))
                    .or_insert_with(|| self.draw_chunk((cx, cy)));
                found.extend(chunk.features.iter().copied().filter(|((x, y), _)| {
                    (from.0..=to.0).contains(x) && (from.1..=to.1).contains(y)
                }));
            }
        }
        found
    }

    /// Lays something on the ground of a tile.
    #[cfg(test)]
    pub fn put_feature(&mut self, tile: (i32, i32), feature: Feature) {
        let (chunk, _) = locate(tile);
        let seed = self.seed;
        let start = self.start;
        self.chunks
            .get_mut()
            .entry(chunk)
            .or_insert_with(|| draw_chunk(seed, start, chunk))
            .features
            .push((tile, feature));
    }

    /// Whether nothing can stand on a tile: an obstacle covers it.
    pub fn blocks(&self, tile: (i32, i32)) -> bool {
        self.obstacle_at(tile).is_some()
    }

    /// Puts an obstacle on the place of the grid that holds `tile`.
    #[cfg(test)]
    pub fn put(&mut self, tile: (i32, i32), kind: Obstacle) {
        let (chunk, index) = locate(tile);
        let seed = self.seed;
        let chunks = self.chunks.get_mut();
        let start = self.start;
        chunks
            .entry(chunk)
            .or_insert_with(|| draw_chunk(seed, start, chunk))
            .obstacles[index] = Some(kind);
    }

    fn draw_chunk(&self, chunk: (i32, i32)) -> Chunk {
        draw_chunk(self.seed, self.start, chunk)
    }
}

/// The chunk holding a tile, and where in the chunk.
fn locate(tile: (i32, i32)) -> ((i32, i32), usize) {
    let place = (
        tile.0.div_euclid(OBSTACLE_SIZE),
        tile.1.div_euclid(OBSTACLE_SIZE),
    );
    let chunk = (
        place.0.div_euclid(CHUNK_SIZE),
        place.1.div_euclid(CHUNK_SIZE),
    );
    let (x, y) = (
        place.0.rem_euclid(CHUNK_SIZE),
        place.1.rem_euclid(CHUNK_SIZE),
    );
    (chunk, usize::try_from(y * CHUNK_SIZE + x).unwrap_or(0))
}

/// A chunk's obstacles: clumps until they cover their share, none near `start`. A clump
/// wanders from a place to a neighbour within the chunk, leaving an obstacle where it can.
fn draw_chunk(seed: Option<u64>, start: (i32, i32), chunk: (i32, i32)) -> Chunk {
    let mut places = vec![None; usize::try_from(CHUNK_SIZE * CHUNK_SIZE).unwrap_or(0)];
    let Some(seed) = seed else {
        return Chunk {
            obstacles: places,
            features: Vec::new(),
        };
    };
    let mut rng = chunk_luck(seed, chunk);
    let wanted = CHUNK_SIZE * CHUNK_SIZE * COVER_PERCENT / 100;
    let mut count = 0;
    for _ in 0..MAX_CLUMPS {
        if count >= wanted {
            break;
        }
        let kind = if rng.random_bool(GROVE_CHANCE) {
            Obstacle::Tree
        } else {
            Obstacle::Rock
        };
        let mut at = (
            rng.random_range(0..CHUNK_SIZE),
            rng.random_range(0..CHUNK_SIZE),
        );
        for _ in 0..rng.random_range(CLUMP_SIZE) {
            let index = usize::try_from(at.1 * CHUNK_SIZE + at.0).unwrap_or(0);
            let place = (chunk.0 * CHUNK_SIZE + at.0, chunk.1 * CHUNK_SIZE + at.1);
            if places[index].is_none() && !near_start(place, start) {
                places[index] = Some(kind);
                count += 1;
            }
            let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)][rng.random_range(0..4)];
            at = (
                (at.0 + dx).clamp(0, CHUNK_SIZE - 1),
                (at.1 + dy).clamp(0, CHUNK_SIZE - 1),
            );
        }
    }
    // Drawn after the obstacles, from the same luck: the obstacles stay as they were.
    let mut features = Vec::new();
    let herb = if rng.random_range(0..100) < HERB_PERCENT {
        free_spot(&mut rng, &places, chunk, 0)
    } else {
        None
    };
    features.extend(herb.map(|tile| (tile, Feature::Herb)));
    if rng.random_range(0..100) < CAMPFIRE_PERCENT {
        let fire = free_spot(&mut rng, &places, chunk, CLEARING).filter(|tile| Some(*tile) != herb);
        features.extend(fire.map(|tile| (tile, Feature::Campfire)));
    }
    Chunk {
        obstacles: places,
        features,
    }
}

/// A tile of the chunk with no obstacle within `clearing` tiles, if one turns up in a few tries.
/// Only the chunk's own obstacles count: a clearing on its edge may touch the next chunk's.
fn free_spot(
    rng: &mut StdRng,
    places: &[Option<Obstacle>],
    chunk: (i32, i32),
    clearing: i32,
) -> Option<(i32, i32)> {
    let tiles = CHUNK_SIZE * OBSTACLE_SIZE;
    let blocked = |(x, y): (i32, i32)| {
        let inside = (0..tiles).contains(&x) && (0..tiles).contains(&y);
        let index = (y / OBSTACLE_SIZE) * CHUNK_SIZE + x / OBSTACLE_SIZE;
        inside && places[usize::try_from(index).unwrap_or(0)].is_some()
    };
    (0..FEATURE_TRIES)
        .map(|_| (rng.random_range(0..tiles), rng.random_range(0..tiles)))
        .find(|(x, y)| {
            (-clearing..=clearing)
                .all(|dy| (-clearing..=clearing).all(|dx| !blocked((x + dx, y + dy))))
        })
        .map(|(x, y)| (chunk.0 * tiles + x, chunk.1 * tiles + y))
}

/// The luck of a chunk: the run's seed and the chunk's place, each in its own bytes.
fn chunk_luck(seed: u64, (x, y): (i32, i32)) -> StdRng {
    let mut key = [0; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    key[8..12].copy_from_slice(&x.to_le_bytes());
    key[12..16].copy_from_slice(&y.to_le_bytes());
    StdRng::from_seed(key)
}

/// Whether an obstacle at a place of the grid would come too close to `start`.
fn near_start(place: (i32, i32), start: (i32, i32)) -> bool {
    let centre = (
        place.0 * OBSTACLE_SIZE + OBSTACLE_SIZE / 2,
        place.1 * OBSTACLE_SIZE + OBSTACLE_SIZE / 2,
    );
    // In i64: far out, the squares outgrow an i32.
    let dx = i64::from(centre.0) - i64::from(start.0);
    let dy = i64::from(centre.1) - i64::from(start.1);
    let reach = i64::from(CLEAR_RADIUS + OBSTACLE_SIZE);
    dx * dx + dy * dy <= reach * reach
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: (i32, i32) = (0, 0);

    /// The obstacles of a square of tiles, row by row.
    fn obstacles(map: &Map, left: i32, top: i32, size: i32) -> Vec<Option<Obstacle>> {
        (top..top + size)
            .flat_map(|y| (left..left + size).map(move |x| (x, y)))
            .map(|tile| map.obstacle_at(tile))
            .collect()
    }

    #[test]
    fn a_seed_always_gives_the_same_map() {
        let (a, b, other) = (Map::new(7, START), Map::new(7, START), Map::new(8, START));
        // Drawn in another order: the same.
        let far = obstacles(&b, 500, -300, 64);
        assert_eq!(obstacles(&a, -64, -64, 128), obstacles(&b, -64, -64, 128));
        assert_eq!(obstacles(&a, 500, -300, 64), far);
        assert_ne!(
            obstacles(&a, -64, -64, 128),
            obstacles(&other, -64, -64, 128)
        );
    }

    #[test]
    fn obstacles_cover_their_share_of_a_chunk() {
        let map = Map::new(1, START);
        let tiles = CHUNK_SIZE * OBSTACLE_SIZE;
        for (cx, cy) in [(3, 2), (-5, 7), (40, -40)] {
            let covered = obstacles(&map, cx * tiles, cy * tiles, tiles)
                .iter()
                .flatten()
                .count();
            // 4 tiles to an obstacle.
            let count = i32::try_from(covered).unwrap() / (OBSTACLE_SIZE * OBSTACLE_SIZE);
            let wanted = CHUNK_SIZE * CHUNK_SIZE * COVER_PERCENT / 100;
            assert!(count >= wanted && count < wanted + 8, "{count} obstacles");
        }
    }

    #[test]
    fn herbs_and_campfires_are_rare_and_on_free_ground() {
        let map = Map::new(3, START);
        let tiles = CHUNK_SIZE * OBSTACLE_SIZE;
        let (mut herbs, mut fires) = (0, 0);
        for chunk in 0..600 {
            let from = (chunk * tiles, 16 * tiles);
            for ((x, y), feature) in map.features_in(from, (from.0 + tiles - 1, from.1 + tiles - 1))
            {
                assert!(!map.blocks((x, y)), "{feature:?} on an obstacle");
                match feature {
                    Feature::Herb => herbs += 1,
                    Feature::Campfire => fires += 1,
                }
            }
        }
        // About one chunk in 33 for a herb, one in 100 for a campfire.
        assert!((8..=32).contains(&herbs), "{herbs} herbs in 600 chunks");
        assert!((1..=14).contains(&fires), "{fires} campfires in 600 chunks");
    }

    #[test]
    fn the_map_goes_on_and_on() {
        let map = Map::new(1, START);
        assert!(
            obstacles(&map, 100_000, -100_000, 32)
                .iter()
                .any(Option::is_some)
        );
    }

    #[test]
    fn the_start_is_clear() {
        for seed in 0..20 {
            let map = Map::new(seed, START);
            for y in -CLEAR_RADIUS..=CLEAR_RADIUS {
                for x in -CLEAR_RADIUS..=CLEAR_RADIUS {
                    if x * x + y * y > CLEAR_RADIUS * CLEAR_RADIUS {
                        continue;
                    }
                    assert!(!map.blocks((x, y)), "seed {seed}: ({x}, {y}) is blocked");
                }
            }
        }
    }

    #[test]
    fn an_obstacle_covers_two_tiles_each_way() {
        let mut map = Map::empty();
        map.put((-3, -1), Obstacle::Rock);
        for tile in [(-4, -2), (-3, -2), (-4, -1), (-3, -1)] {
            assert_eq!(map.obstacle_at(tile), Some(Obstacle::Rock));
        }
        assert_eq!(map.obstacle_at((-2, -2)), None);
    }
}
