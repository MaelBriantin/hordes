use super::{Facing, Vec2};

/// How long an enemy burns once on fire, in seconds; the ticks between two of its losses, and
/// what each takes from it, in HP: 2 HP every half second, 12 in all.
const BURN_SECONDS: u32 = 3;
const BURN_EVERY: u32 = 15;
const BURN_DAMAGE: u32 = 1;

/// The ticks an enemy burns once on fire.
fn burn_ticks() -> u32 {
    BURN_SECONDS * u32::from(super::TICKS_PER_SECOND)
}

/// How many ticks an enemy flashes white when hit.
const FLASH_TICKS: u32 = 3;

/// A Shadow hunting in the dark goes faster the longer it hunts: up to this much faster, in
/// percent, after so many ticks (5 seconds).
const HUNT_MOST_PERCENT: u16 = 50;
const HUNT_RAMP_TICKS: u32 = 150;
/// Caught in the torch's light, a Shadow bolts out of it: so many ticks, so many times its pace.
pub const BOLT_TICKS: u32 = 10;
pub const BOLT_PACE: f32 = 2.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Zombie,
    Shadow,
    Abomination,
    /// Out of a felled abomination: small, fast, frail; it always finds the player.
    Parasite,
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
    /// The kinds that appear of themselves: parasites come out of abominations alone.
    pub const ALL: [EnemyKind; 3] = [EnemyKind::Zombie, EnemyKind::Shadow, EnemyKind::Abomination];

    pub fn stats(self) -> Stats {
        match self {
            EnemyKind::Zombie => Stats {
                hp: 15,
                xp: 1,
                drop_percent: 75,
                stain: 3,
                speed: 2.5,
                damage: 5,
                half_width: 0.45,
                flies: false,
            },
            EnemyKind::Shadow => Stats {
                hp: 40,
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
            EnemyKind::Parasite => Stats {
                hp: 12,
                xp: 0,
                drop_percent: 0,
                stain: 1,
                speed: 4.5,
                damage: 3,
                half_width: 0.3,
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
    /// Ticks left it burns, losing HP as it does; 0 when it doesn't.
    pub burning: u32,
    /// Ticks left it stands stunned, not moving; 0 when it isn't.
    pub stunned: u32,
    /// Ticks it has hunted the player in the dark, without a break: a Shadow's.
    pub hunting: u32,
    /// Ticks left it bolts out of the torch's light: a Shadow's.
    pub bolting: u32,
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
            burning: 0,
            stunned: 0,
            hunting: 0,
            bolting: 0,
        }
    }

    /// Its pace going for the player: faster the longer it has hunted in the dark.
    pub fn hunt_pace(&self) -> f32 {
        let ramped = u16::try_from(self.hunting.min(HUNT_RAMP_TICKS)).unwrap_or(0);
        let ramp = u16::try_from(HUNT_RAMP_TICKS).unwrap_or(u16::MAX);
        1.0 + f32::from(ramped) / f32::from(ramp) * f32::from(HUNT_MOST_PERCENT) / 100.0
    }

    /// Catches fire, or burns anew: as long again as a fire burns, never longer.
    pub fn ignite(&mut self) {
        self.burning = burn_ticks();
    }

    /// Burns on by a tick, if it burns: a little HP lost every so often.
    pub fn scorch(&mut self) {
        if self.burning == 0 {
            return;
        }
        self.burning -= 1;
        if self.burning.is_multiple_of(BURN_EVERY) {
            self.hp = self.hp.saturating_sub(BURN_DAMAGE);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_enemy_on_fire_burns_six_hp_over_three_seconds_and_anew_if_set_on_fire_again() {
        let mut enemy = Enemy::new(0, EnemyKind::Abomination, Vec2::ZERO, 0.0);
        enemy.hp = 100;
        enemy.ignite();
        for _ in 0..burn_ticks() {
            enemy.scorch();
        }
        assert_eq!((enemy.hp, enemy.burning), (94, 0));
        enemy.ignite();
        for _ in 0..30 {
            enemy.scorch();
        }
        enemy.ignite();
        assert_eq!(enemy.burning, burn_ticks());
    }
}
