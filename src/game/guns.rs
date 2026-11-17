//! The firearms: the pistol and the machine gun. A gun shoots at the nearest enemies in its range
//! while its magazine holds rounds, then reloads it. Its reserve of rounds is endless.

use super::build::Weapon;
use super::enemy::Enemy;
use super::{TICKS_PER_SECOND, Vec2, tick_seconds};

/// Half the width of a bullet, in tiles: a pixel.
pub const BULLET_HALF_WIDTH: f32 = 0.25;

/// What a gun is like at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gun {
    /// The rounds a full magazine holds: one per shot.
    pub magazine: u32,
    /// The ticks between two shots.
    pub interval: u32,
    /// The ticks a reload takes, before Haste.
    pub reload: u32,
    /// What a bullet takes from the enemy it hits, in HP, before Might.
    pub damage: u32,
    /// The bullets of a shot, each at a different enemy.
    pub bullets: usize,
    /// How many enemies a bullet goes through before stopping at the next.
    pub pierce: u32,
    /// In tiles per second.
    pub speed: f32,
    /// How far a bullet flies, and how near an enemy must be to be shot at, in tiles.
    pub range: f32,
}

/// Tenths of a second, in ticks.
fn tenths(count: u32) -> u32 {
    u32::from(TICKS_PER_SECOND) * count / 10
}

/// A gun at a level of its weapon; none for a weapon that isn't a gun.
pub fn gun(weapon: Weapon, level: u8) -> Option<Gun> {
    match weapon {
        Weapon::Pistol => Some(Gun {
            magazine: match level {
                0 | 1 => 6,
                2..=4 => 9,
                _ => 12,
            },
            interval: tenths(5),
            reload: if level >= 3 { tenths(15) } else { tenths(20) },
            damage: if level >= 5 { 15 } else { 10 },
            bullets: if level >= 4 { 2 } else { 1 },
            pierce: 0,
            speed: 20.0,
            range: 12.0,
        }),
        Weapon::MachineGun => Some(Gun {
            magazine: if level >= 2 { 30 } else { 20 },
            interval: tenths(1),
            reload: if level >= 4 { tenths(20) } else { tenths(30) },
            damage: if level >= 3 { 6 } else { 4 },
            bullets: 1,
            pierce: u32::from(level >= 5),
            speed: 22.0,
            range: 10.0,
        }),
        Weapon::Machete | Weapon::Drones => None,
    }
}

/// A gun's magazine: the rounds left in it, and what the gun waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Magazine {
    pub rounds: u32,
    /// Ticks before the next shot.
    pub wait: u32,
    /// Ticks left before the reload is done; 0 when not reloading.
    pub reloading: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Vec2,
    /// Its move each tick.
    velocity: Vec2,
    /// How far it has flown, and may fly, in tiles.
    flown: f32,
    range: f32,
    pub damage: u32,
    /// How many more enemies it goes through.
    pub pierce: u32,
    /// The enemies it has hit, by id: it hits each only once.
    pub hit: Vec<u32>,
}

impl Bullet {
    /// A bullet of `gun` leaving `from` towards `target`.
    pub fn toward(from: Vec2, target: Vec2, gun: &Gun, damage: u32) -> Self {
        Self {
            position: from,
            velocity: (target - from).normalized() * (gun.speed * tick_seconds()),
            flown: 0.0,
            range: gun.range,
            damage,
            pierce: gun.pierce,
            hit: Vec::new(),
        }
    }

    /// Moves it on by one tick. Whether it is still flying.
    pub fn advance(&mut self) -> bool {
        self.position += self.velocity;
        self.flown += self.velocity.length();
        self.flown < self.range
    }
}

/// Where the `count` enemies nearest to `from` are, those within `range`, nearest first.
pub fn targets(enemies: &[Enemy], from: Vec2, count: usize, range: f32) -> Vec<Vec2> {
    let mut in_range: Vec<Vec2> = enemies
        .iter()
        .map(|enemy| enemy.position)
        .filter(|position| (*position - from).length() <= range)
        .collect();
    in_range.sort_by(|a, b| (*a - from).length().total_cmp(&(*b - from).length()));
    in_range.truncate(count);
    in_range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pistol_grows_with_its_levels() {
        let first = gun(Weapon::Pistol, 1).unwrap();
        let last = gun(Weapon::Pistol, 5).unwrap();
        assert_eq!((first.magazine, first.reload, first.bullets), (6, 60, 1));
        assert_eq!(
            (last.magazine, last.reload, last.bullets, last.damage),
            (12, 45, 2, 15)
        );
    }

    #[test]
    fn the_machine_gun_shoots_fast_and_pierces_at_the_top() {
        let first = gun(Weapon::MachineGun, 1).unwrap();
        assert_eq!((first.interval, first.magazine, first.pierce), (3, 20, 0));
        assert_eq!(gun(Weapon::MachineGun, 5).unwrap().pierce, 1);
    }

    #[test]
    fn the_machete_is_no_gun() {
        assert_eq!(gun(Weapon::Machete, 1), None);
    }
}
