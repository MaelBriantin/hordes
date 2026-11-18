use super::{Facing, Vec2};

/// How many ticks an enemy flashes white when hit.
const FLASH_TICKS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Zombie,
    Shadow,
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
}

impl EnemyKind {
    pub const ALL: [EnemyKind; 3] = [EnemyKind::Zombie, EnemyKind::Shadow, EnemyKind::Abomination];

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
            },
            EnemyKind::Shadow => Stats {
                hp: 25,
                xp: 2,
                drop_percent: 75,
                stain: 2,
                speed: 5.7,
                damage: 8,
                half_width: 0.35,
                flies: true,
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
    /// A number of its own, drawn as it appears, in radians: where it starts to wander.
    pub phase: f32,
    /// Whether it has found the player, and goes for them: in the dark, it has to see them first.
    pub noticed: bool,
    /// Where it wanders when it hasn't, in radians.
    pub wander: f32,
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
            noticed: false,
            wander: phase,
        }
    }

    /// Takes a hit: loses `damage` HP, and flashes white.
    pub fn hit(&mut self, damage: u32) {
        self.hp = self.hp.saturating_sub(damage);
        self.flash = FLASH_TICKS;
    }

    /// Where it wants to go this tick, towards `target`, straight. Length 1, or zero.
    pub fn heading(&self, target: Vec2) -> Vec2 {
        (target - self.position).normalized()
    }
}
