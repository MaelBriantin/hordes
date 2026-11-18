use super::{Facing, Vec2, tick_seconds};

/// How fast a bat goes through its zigzag (and beats its wings), in radians per second.
const WOBBLE_SPEED: f32 = 6.0;
/// How many ticks an enemy flashes white when hit.
const FLASH_TICKS: u32 = 3;
/// How far a bat's zigzag swerves from its way, against 1 for its way.
const WOBBLE_STRENGTH: f32 = 0.9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Zombie,
    Bat,
    Abomination,
}

/// What a kind of enemy is like.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub hp: u32,
    /// The XP of the gem it may leave when it falls.
    pub xp: u32,
    /// The chance it leaves a gem when it falls, in percent.
    pub drop_percent: u32,
    /// How big a stain it leaves when it falls.
    pub stain: u8,
    /// In tiles per second.
    pub speed: f32,
    /// What touching the player costs them, in HP.
    pub damage: u32,
    /// Half the width of its body, in tiles.
    pub half_width: f32,
    /// Whether it flies over obstacles.
    pub flies: bool,
    /// When it starts to appear, in seconds into the run.
    pub arrives_at: u32,
    /// How often it appears, against the other kinds that have arrived.
    pub weight: u32,
}

impl EnemyKind {
    pub const ALL: [EnemyKind; 3] = [EnemyKind::Zombie, EnemyKind::Bat, EnemyKind::Abomination];

    pub fn stats(self) -> Stats {
        match self {
            EnemyKind::Zombie => Stats {
                hp: 10,
                xp: 1,
                drop_percent: 75,
                stain: 3,
                speed: 2.5,
                damage: 5,
                half_width: 0.45,
                flies: false,
                arrives_at: 0,
                weight: 6,
            },
            EnemyKind::Bat => Stats {
                hp: 5,
                xp: 1,
                drop_percent: 75,
                stain: 2,
                speed: 5.0,
                damage: 3,
                half_width: 0.35,
                flies: true,
                arrives_at: 60,
                weight: 3,
            },
            EnemyKind::Abomination => Stats {
                hp: 40,
                xp: 5,
                drop_percent: 100,
                stain: 6,
                speed: 2.0,
                damage: 15,
                half_width: 0.8,
                flies: false,
                arrives_at: 180,
                weight: 1,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    /// Its own number in the run.
    pub id: u32,
    pub kind: EnemyKind,
    /// The centre of its body, in tiles.
    pub position: Vec2,
    pub facing: Facing,
    pub hp: u32,
    /// Ticks left during which it shows white, after a hit.
    pub flash: u32,
    /// The horde it came with, if it did.
    pub horde: Option<u32>,
    /// Where it is in its zigzag and wingbeat, in radians: a bat's own, drawn as it appears.
    pub phase: f32,
}

impl Enemy {
    pub fn new(id: u32, kind: EnemyKind, position: Vec2, phase: f32) -> Self {
        Self {
            id,
            kind,
            position,
            facing: Facing::default(),
            hp: kind.stats().hp,
            flash: 0,
            horde: None,
            phase,
        }
    }

    /// Takes a hit: loses `damage` HP, and flashes white.
    pub fn hit(&mut self, damage: u32) {
        self.hp = self.hp.saturating_sub(damage);
        self.flash = FLASH_TICKS;
    }

    /// Where it wants to go this tick, towards `target`: straight for most, in a zigzag for a
    /// bat. Length 1, or zero.
    pub fn heading(&mut self, target: Vec2) -> Vec2 {
        let way = (target - self.position).normalized();
        if self.kind != EnemyKind::Bat {
            return way;
        }
        self.phase += WOBBLE_SPEED * tick_seconds();
        (way + way.perpendicular() * (self.phase.sin() * WOBBLE_STRENGTH)).normalized()
    }
}
