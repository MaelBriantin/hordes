//! The firearms: the pistol and the machine gun. A gun shoots at the nearest enemies in its range
//! while its magazine holds rounds, then reloads it. Its reserve of rounds is endless.

use rand::{Rng, RngExt};

use super::build::Weapon;
use super::enemy::Enemy;
use super::{TICKS_PER_SECOND, Vec2, tick_seconds};

/// Half the width of a bullet, in tiles: a pixel.
pub const BULLET_HALF_WIDTH: f32 = 0.25;
/// How near the enemy it hit the next one has to be for a star to ricochet on it, in tiles; and
/// what each ricochet takes off its damage, in points of percent of what it did first.
pub const RICOCHET_RANGE: f32 = 4.0;
const RICOCHET_LOSS: u32 = 25;

/// What a gun is like at a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gun {
    /// Which gun it is: its bullets show its own way.
    pub weapon: Weapon,
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
    /// The pellets each bullet bursts into, fanned out over `spread` radians: 1 but for the
    /// shotgun.
    pub pellets: u16,
    pub spread: f32,
    /// How many enemies a bullet goes through before stopping at the next.
    pub pierce: u32,
    /// How many times a bullet ricochets on the nearest enemy left once it has hit one, a
    /// quarter of its damage less each time.
    pub ricochets: u32,
    /// In tiles per second.
    pub speed: f32,
    /// How far a bullet flies, and how near an enemy must be to be shot at, in tiles.
    pub range: f32,
}

/// The ticks the machine gun waits between two shots of a burst (its wait, then the shot): a
/// shot every 3 ticks, a tenth of a second.
const MACHINE_GUN_INTERVAL: u32 = 2;

/// How much faster or slower than its gun a pellet may fly, as a share of the gun's speed.
const PELLET_PACE: f32 = 0.2;

/// Tenths of a second, in ticks.
fn tenths(count: u32) -> u32 {
    u32::from(TICKS_PER_SECOND) * count / 10
}

/// A gun at a level of its weapon; none for a weapon that isn't a gun.
pub fn gun(weapon: Weapon, level: u8) -> Option<Gun> {
    match weapon {
        Weapon::Pistol => Some(Gun {
            weapon,
            magazine: match level {
                0 | 1 => 6,
                2..=4 => 9,
                _ => 12,
            },
            interval: tenths(5),
            reload: if level >= 3 { tenths(15) } else { tenths(20) },
            damage: if level >= 5 { 15 } else { 10 },
            bullets: if level >= 4 { 2 } else { 1 },
            pellets: 1,
            spread: 0.0,
            pierce: 0,
            ricochets: 0,
            speed: 20.0,
            range: 12.0,
        }),
        Weapon::MachineGun => Some(Gun {
            weapon,
            // Short bursts, a magazine each: longer with the levels.
            magazine: match level {
                0 | 1 => 8,
                2..=4 => 12,
                _ => 16,
            },
            interval: MACHINE_GUN_INTERVAL,
            reload: if level >= 4 { tenths(18) } else { tenths(25) },
            damage: if level >= 3 { 5 } else { 3 },
            bullets: 1,
            pellets: 1,
            spread: 0.0,
            pierce: u32::from(level >= 5),
            ricochets: 0,
            speed: 22.0,
            range: 10.0,
        }),
        Weapon::Shotgun => Some(Gun {
            weapon,
            // A double shot, then the reload.
            magazine: 2,
            interval: tenths(3),
            reload: if level >= 4 { tenths(15) } else { tenths(25) },
            damage: if level >= 3 { 8 } else { 6 },
            bullets: 1,
            pellets: if level >= 2 { 7 } else { 5 },
            spread: if level >= 5 { 60.0_f32 } else { 40.0 }.to_radians(),
            pierce: u32::from(level >= 5),
            ricochets: 0,
            speed: 18.0,
            range: 6.0,
        }),
        Weapon::Shuriken => Some(Gun {
            weapon,
            // A few stars thrown in a row, then the next ones taken out.
            magazine: if level >= 2 { 4 } else { 3 },
            interval: tenths(3),
            reload: if level >= 3 { tenths(10) } else { tenths(14) },
            damage: 9,
            bullets: if level >= 5 { 2 } else { 1 },
            pellets: 1,
            spread: 0.0,
            pierce: u32::from(level < 4),
            ricochets: if level >= 4 { 3 } else { 0 },
            speed: 14.0,
            range: 8.0,
        }),
        Weapon::Crowbar
        | Weapon::Machete
        | Weapon::Drones
        | Weapon::Molotov
        | Weapon::Flamethrower
        | Weapon::Hammer
        | Weapon::Baton
        | Weapon::Knife
        | Weapon::Katana
        | Weapon::Defibrillator
        | Weapon::Chainsaw
        | Weapon::Grenade
        | Weapon::Sniper
        | Weapon::Napalm
        | Weapon::Fists
        | Weapon::Truncheon => None,
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
    /// What it did to the first enemy it hit, before ricocheting.
    first_damage: u32,
    /// How many more enemies it goes through; how many more times it ricochets, and has.
    pub pierce: u32,
    ricochets: u32,
    bounces: u32,
    /// The enemies it has hit, by id: it hits each only once.
    pub hit: Vec<u32>,
    /// The gun it left: a pistol's slug and a machine gun's tracer don't look alike.
    pub weapon: Weapon,
    /// A shotgun's pellets come in two looks, every other one.
    pub other: bool,
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
            first_damage: damage,
            pierce: gun.pierce,
            ricochets: gun.ricochets,
            bounces: 0,
            hit: Vec::new(),
            weapon: gun.weapon,
            other: false,
        }
    }

    /// The pellets of a shot of `gun` leaving `from` towards `target`, scattered at random over
    /// the gun's spread, each a little faster or slower than the gun's speed: a spray, not a
    /// wave. A single bullet, straight at the target, for a gun that doesn't spread.
    pub fn burst(
        from: Vec2,
        target: Vec2,
        gun: &Gun,
        damage: u32,
        rng: &mut impl Rng,
    ) -> Vec<Self> {
        if gun.pellets <= 1 {
            return vec![Self::toward(from, target, gun, damage)];
        }
        let way = (target - from).normalized();
        let middle = way.y.atan2(way.x);
        let half = gun.spread / 2.0;
        (0..gun.pellets)
            .map(|index| {
                let angle = middle + rng.random_range(-half..=half);
                let aim = from + Vec2::new(angle.cos(), angle.sin());
                let mut pellet = Self::toward(from, aim, gun, damage);
                pellet.velocity =
                    pellet.velocity * rng.random_range(1.0 - PELLET_PACE..=1.0 + PELLET_PACE);
                pellet.other = index % 2 == 1;
                pellet
            })
            .collect()
    }

    /// The way it flies, length 1.
    pub fn way(&self) -> Vec2 {
        self.velocity.normalized()
    }

    /// Whether it ricochets on the next enemy once it has hit one.
    pub fn ricochets(&self) -> bool {
        self.ricochets > 0
    }

    /// Off on a ricochet towards `target`, from where it hit: as far again as the next one may
    /// be, a quarter of what it first did less.
    pub fn ricochet(&mut self, target: Vec2) {
        let speed = self.velocity.length();
        self.velocity = (target - self.position).normalized() * speed;
        self.flown = 0.0;
        self.range = RICOCHET_RANGE + 1.0;
        self.ricochets -= 1;
        self.bounces += 1;
        self.damage =
            self.first_damage * 100_u32.saturating_sub(RICOCHET_LOSS * self.bounces) / 100;
    }

    /// Moves it on by one tick. Whether it is still flying.
    pub fn advance(&mut self) -> bool {
        self.position += self.velocity;
        self.flown += self.velocity.length();
        self.flown < self.range
    }
}

/// Where the `count` enemies nearest to `from` are, those within `range` that `sight` says can
/// be seen, nearest first.
pub fn targets(
    enemies: &[Enemy],
    from: Vec2,
    count: usize,
    range: f32,
    sight: impl Fn(Vec2) -> bool,
) -> Vec<Vec2> {
    let mut in_range: Vec<Vec2> = enemies
        .iter()
        .map(|enemy| enemy.position)
        .filter(|position| (*position - from).length() <= range && sight(*position))
        .collect();
    in_range.sort_by(|a, b| (*a - from).length().total_cmp(&(*b - from).length()));
    in_range.truncate(count);
    in_range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_level_4_a_star_ricochets_three_times_a_quarter_of_its_damage_less_each() {
        let gun = gun(Weapon::Shuriken, 4).unwrap();
        assert_eq!((gun.pierce, gun.ricochets), (0, 3));
        assert_eq!(gun_ricochets(3), 0, "not before");
        let mut star = Bullet::toward(Vec2::ZERO, Vec2::new(1.0, 0.0), &gun, 12);
        let mut damages = vec![star.damage];
        while star.ricochets() {
            star.ricochet(Vec2::new(0.0, 1.0));
            damages.push(star.damage);
        }
        assert_eq!(damages, [12, 9, 6, 3]);
        assert!(star.way().y > 0.99, "off towards the next");
    }

    fn gun_ricochets(level: u8) -> u32 {
        gun(Weapon::Shuriken, level).unwrap().ricochets
    }

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
    fn the_machine_gun_shoots_short_bursts_longer_with_the_levels() {
        let first = gun(Weapon::MachineGun, 1).unwrap();
        assert_eq!(
            (first.interval, first.magazine, first.reload, first.pierce),
            (2, 8, 75, 0)
        );
        let last = gun(Weapon::MachineGun, 5).unwrap();
        assert_eq!((last.magazine, last.reload, last.pierce), (16, 54, 1));
    }

    #[test]
    fn the_machete_is_no_gun() {
        assert_eq!(gun(Weapon::Machete, 1), None);
    }

    #[test]
    fn a_shotgun_sprays_its_pellets_within_its_fan() {
        use rand::SeedableRng;
        let gun = gun(Weapon::Shotgun, 1).unwrap();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let pellets = Bullet::burst(Vec2::ZERO, Vec2::new(5.0, 0.0), &gun, 6, &mut rng);
        assert_eq!(pellets.len(), 5);
        // Each within 20° of the target's way, each at a pace of its own.
        let most = 20.0_f32.to_radians().sin() + 1e-4;
        assert!(pellets.iter().all(|pellet| pellet.way().y.abs() <= most));
        let paces: Vec<f32> = pellets
            .iter()
            .map(|pellet| pellet.velocity.length())
            .collect();
        assert!(
            paces
                .windows(2)
                .any(|pair| (pair[0] - pair[1]).abs() > 1e-4)
        );
    }
}
