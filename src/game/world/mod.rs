use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::announce::{Announcement, News, WeatherNews};
use super::body::{fits, slide, touch};
use super::build::{
    BASE_HP, Build, Choice, HEAL, Legendary, Passive, RARE_PERCENT, VITALITY_HP, Weapon,
};
use super::burst::{self, Blast, Swelling};
use super::dash::{Dash, Dashes};
use super::drones;
use super::enemy::{BOLT_PACE, BOLT_TICKS, Enemy, EnemyKind};
use super::gem::Gem;
use super::guns::{self, Bullet, Magazine};
use super::heal::{self, CAMPFIRE_HP, CAMPFIRE_LIGHT, HERB_HP, HERB_REACH};
use super::horde::{self, Tally, Warning};
use super::level::xp_to_next;
use super::machete::{self, Melee, Swing};
use super::map::Feature;
use super::mode::Mode;
use super::molotov::{self, Bottle, Puddle};
use super::progress::{Achievement, Progress};
use super::score;
use super::spawn::{self, MARGIN, MAX_ENEMIES};
use super::stain::{self, MAX_STAINS, Stain};
use super::weather::{self, Weather};
use super::{Facing, Map, TICKS_PER_SECOND, Vec2, tick_seconds};

#[cfg(test)]
mod balance;
#[cfg(test)]
mod tests;

/// The player's speed, in tiles per second.
const PLAYER_SPEED: f32 = 6.0;
/// Half the width of the player's body, in tiles: what obstacles stop, smaller than the sprite
/// so that brushing past one is forgiven.
const PLAYER_HALF_WIDTH: f32 = 0.45;
/// How hard enemies that overlap push each other apart, as a share of the overlap per tick.
const SEPARATION: f32 = 0.25;
/// The longest a body moves in a tick, in tiles: less than a tile, so that it never jumps
/// over one.
const MAX_STEP: f32 = 0.9;
/// How many tries to find a free spot for an enemy to appear on.
const SPAWN_TRIES: u32 = 10;
/// The most enemies at once while a horde is about: a horde comes beyond the usual most.
const MAX_ENEMIES_WITH_HORDE: usize = 600;
/// The size of a cell of the grid enemies are sorted in to push each other apart, in tiles:
/// more than the widest two bodies, so that only neighbouring cells can overlap.
const SEPARATION_CELL: f32 = 2.0;
/// In arcade, how much harder the run grows each minute, compounded; and how many enemies appear a
/// second at its start.
const ARCADE_GROWTH: f32 = 1.07;
const ARCADE_RATE: f32 = 1.5;
/// The cards a run can deal again on a level-up, R on the cards.
const REROLLS: u8 = 1;
/// The seconds into the run the torch lights up of itself at, the night falling.
const TORCH_SECONDS: u32 = 26;

/// How far the torch's light fades into the dark beyond its radius, in tiles.
pub const TORCH_FADE: f32 = 6.0;

/// Where a body ends up pushed `by` from `from`, step by step, sliding along what stands in
/// its way.
fn shove(map: &Map, from: Vec2, by: Vec2, half_width: f32, flies: bool) -> Vec2 {
    let step = by * (1.0 / f32::from(PUSH_STEPS));
    (0..PUSH_STEPS).fold(from, |at, _| slide(map, at, step, half_width, flies))
}

/// The tick the torch lights up of itself at.
fn torch_lights_up() -> u32 {
    TORCH_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// How near the player, in the dark, an enemy finds them, and how far it loses them again; how
/// far a campfire lures those that haven't, and how fast they wander, against their speed, in
/// tiles.
const NOTICE_REACH: f32 = 6.0;
const LOSE_REACH: f32 = 8.0;
const LURE_REACH: f32 = 20.0;
const WANDER_PACE: f32 = 0.5;
/// The parasites a felled abomination lets out.
const PARASITES: u16 = 4;
/// The steps a burst pushes in, each shorter than the longest a body moves in a tick.
const PUSH_STEPS: u16 = 3;
/// The most ticks the torch flickers as it lights up, before it holds: a second and a half.
const TORCH_FLICKER_MOST: u32 = 45;
/// The seconds a horde wiped out waits before its level comes, for the news to be read first;
/// and the chance, in percent, of a rare card on that level.
const HORDE_LEVEL_DELAY: u32 = 2;
const HORDE_RARE_PERCENT: u32 = 60;
/// The chance, in percent, that a level-up offers a legendary, at first and after one.
const LEGENDARY_PERCENT: u32 = 5;
/// How near the player the debug console sets enemies on fire, in tiles.
const DEBUG_FIRE_REACH: f32 = 8.0;
/// The ticks an enemy flashes white before it burns at daybreak.
const BURN_FLASH: u32 = 6;
/// The weapons that are guns, in the order they show.
const GUNS: [Weapon; 3] = [Weapon::Pistol, Weapon::MachineGun, Weapon::Shotgun];
/// A melee weapon's swing as it goes: the ticks before it can swing again; the slashes of the
/// swing done, still to come, and the ticks before the next.
#[derive(Debug, Clone, Copy, Default)]
struct Swinging {
    wait: u32,
    done: u32,
    left: u32,
    gap: u32,
}

/// The most gems lying around at once: past it, the oldest go.
const MAX_GEMS: usize = 500;
/// What the player sees until the app tells, half of it each way, in tiles.
const DEFAULT_VIEW: Vec2 = Vec2::new(40.0, 12.0);

/// The last strike of lightning: where it fell, and when.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lightning {
    pub tick: u32,
    pub at: Vec2,
}

/// What the player asks for during a tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Input {
    /// Where to go: -1, 0 or 1 on each axis, `y` growing downwards.
    pub direction: Vec2,
}

#[derive(Debug)]
pub struct Player {
    /// The centre of the player's body, in tiles: the tile `(0, 0)` spans from 0.0 to 1.0.
    pub position: Vec2,
    pub facing: Facing,
    pub hp: u32,
    /// Ticks left during which nothing hurts the player, after a hit.
    pub invincible: u32,
    /// The dashes ready, and the next coming back.
    pub dashes: Dashes,
    /// A dash under way: nothing hurts the player meanwhile.
    pub dash: Option<Dash>,
    pub level: u32,
    /// The XP gathered towards the next level.
    pub xp: u32,
    pub build: Build,
}

impl Player {
    pub fn max_hp(&self) -> u32 {
        self.build.max_hp()
    }

    /// Gathers XP, going up as many levels as it makes. How many.
    fn gain(&mut self, xp: u32) -> u32 {
        self.xp += xp;
        let mut levels = 0;
        while self.xp >= xp_to_next(self.level) {
            self.xp -= xp_to_next(self.level);
            self.level += 1;
            levels += 1;
        }
        levels
    }
}

#[derive(Debug)]
pub struct World {
    pub map: Map,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    /// The number the next enemy to appear gets.
    next_enemy_id: u32,
    pub bullets: Vec<Bullet>,
    pub gems: Vec<Gem>,
    /// Where blood was spilt lately, oldest first.
    pub stains: Vec<Stain>,
    /// The enemies the player has felled.
    pub kills: u32,
    pub score: u64,
    /// The kills in a row, each within the combo's window of the one before; 0 once it closes.
    pub combo: u32,
    /// The tick of the last kill.
    last_kill: u32,
    /// Each melee weapon's swing as it goes, once the player has the weapon.
    melee: HashMap<Weapon, Swinging>,
    /// The last slash of a melee weapon, while it shows.
    pub swing: Option<Swing>,
    /// Each gun's magazine, once the player has the gun.
    pub magazines: HashMap<Weapon, Magazine>,
    /// Where the first drone is on its circle, in radians.
    drone_angle: f32,
    /// Ticks left before each drone, broken by a blow, works again; 0 for one that works.
    drone_repairs: Vec<u32>,
    /// The Molotov's bottles in the air, its puddles of fire on the ground, and the ticks before
    /// its next throw.
    pub bottles: Vec<Bottle>,
    pub puddles: Vec<Puddle>,
    molotov_wait: u32,
    /// The abominations felled, swelling before they burst; and their bursts, as they show.
    pub swellings: Vec<Swelling>,
    pub blasts: Vec<Blast>,
    /// The last strike of lightning, if any yet.
    pub lightning: Option<Lightning>,
    /// The strikes coming, their ring showing where they will fall: where, and since when.
    pub strikes_coming: Vec<Lightning>,
    /// The horde on its way, once announced.
    pub horde_warning: Option<Warning>,
    /// The tick the next horde comes at, and its number.
    next_horde: u32,
    hordes_come: u32,
    /// The hordes come and not yet wiped out, by number.
    tallies: HashMap<u32, Tally>,
    /// The last horde wiped out: when, the points it was worth (none in a standard run), and
    /// its number if another horde still stood then, to tell them apart.
    pub horde_cleared: Option<(u32, u64, Option<u32>)>,
    /// The news the world stands still for, while it shows.
    pub announcement: Option<Announcement>,
    /// What reaching the dawn added to the score, once the run has: the run is won, and over.
    pub dawn_bonus: Option<u64>,
    /// The way the run is played: from dusk to dawn, or endless, for the score.
    pub mode: Mode,
    /// How a new weather or a horde is told; and the last news told the run going on, with the
    /// tick it was told at.
    pub weather_news: WeatherNews,
    pub told: Option<(News, u32)>,
    /// The tick the player was last hurt at, and last healed at, if they have been.
    pub hurt_at: Option<u32>,
    pub healed_at: Option<u32>,
    /// Whether the player wants their torch on; the tick it holds from, lit, flickering until
    /// then, as long as luck has it; and that luck, for the flicker.
    torch_switch: bool,
    pub(super) torch_steady_at: u32,
    torch_luck: u32,
    /// Nothing hurts the player: a debug tool.
    pub unhurt: bool,
    /// The herbs picked, by tile.
    picked: HashSet<(i32, i32)>,
    /// The HP each campfire has given, by tile.
    burnt: HashMap<(i32, i32), u32>,
    /// Ticks spent by a campfire towards its next HP.
    by_the_fire: u32,
    /// Ticks before the next strike, once the weather brings lightning.
    next_strike_in: Option<u32>,
    /// What the player is offered to pick, a level gained: the world waits meanwhile.
    pub offer: Option<Vec<Choice>>,
    /// Levels gained whose choice is still to come, the one on offer included.
    pending_levels: u32,
    /// XP gathered short of a whole point, in hundredths: Wisdom gives tenths.
    xp_hundredths: u32,
    /// HP regenerated short of a whole point, in tenths.
    hp_tenths: u32,
    /// The level-ups dealt since the last legendary offered.
    without_legendary: u32,
    /// The ticks the levels of hordes wiped out are due at, and the levels they brought still to
    /// deal, their cards rarer.
    horde_levels_due: Vec<u32>,
    rare_levels: u32,
    /// How rare the cards on offer were dealt, in percent: dealt again as rare.
    offer_rare: u32,
    /// The times left this run the cards on offer can be dealt again.
    pub rerolls: u8,
    /// What the player has reached, this run counted; and what they had as it started: the
    /// cards it deals, set for the whole run.
    pub progress: Progress,
    dealt: Progress,
    ticks: u32,
    rng: StdRng,
    /// Enemies owed: grows with the rate they appear at, one appears per whole unit.
    spawn_debt: f32,
    /// Half the size of what the player sees, in tiles: enemies appear just beyond it.
    view: Vec2,
}

impl World {
    /// A new run, all its luck from `seed`: the map, and the enemies that appear.
    pub fn new(seed: u64) -> Self {
        Self::with(seed, Mode::Standard, false, Progress::everything())
    }

    /// A new run in a mode on a map drawn from `seed`, from the dusk, or from the night if
    /// `skip_dusk`, the torch lit.
    pub fn with(seed: u64, mode: Mode, skip_dusk: bool, progress: Progress) -> Self {
        let start = Vec2::new(0.5, 0.5);
        let map = Map::new(seed, start.tile());
        let mut world = Self::on(map, start, StdRng::seed_from_u64(seed));
        world.mode = mode;
        // One more run played.
        world.progress = progress;
        world.progress.runs += 1;
        world.progress.count();
        world.rerolls = world.progress.rerolls();
        world.dealt = world.progress.clone();
        world.next_horde = horde::first(mode);
        // A run opens on the dusk, told; or on the night, the dusk skipped.
        let opening = if skip_dusk {
            world.ticks = weather::DUSK_SECONDS * u32::from(TICKS_PER_SECOND);
            Weather::Clear
        } else {
            Weather::Dusk
        };
        world.announcement = Some(Announcement::new(News::Weather(opening)));
        world
    }

    /// A run on a given map, the player at `start`.
    fn on(map: Map, start: Vec2, rng: StdRng) -> Self {
        let mut world = Self {
            map,
            player: Player {
                position: start,
                facing: Facing::default(),
                hp: BASE_HP,
                invincible: 0,
                dashes: Dashes::new(),
                dash: None,
                level: 1,
                xp: 0,
                build: Build::new(),
            },
            enemies: Vec::new(),
            next_enemy_id: 0,
            bullets: Vec::new(),
            gems: Vec::new(),
            stains: Vec::new(),
            kills: 0,
            score: 0,
            combo: 0,
            last_kill: 0,
            melee: HashMap::new(),
            swing: None,
            magazines: HashMap::new(),
            drone_angle: 0.0,
            drone_repairs: Vec::new(),
            lightning: None,
            strikes_coming: Vec::new(),
            bottles: Vec::new(),
            puddles: Vec::new(),
            swellings: Vec::new(),
            blasts: Vec::new(),
            molotov_wait: 0,
            horde_warning: None,
            next_horde: horde::first(Mode::Standard),
            hordes_come: 0,
            tallies: HashMap::new(),
            horde_cleared: None,
            announcement: None,
            dawn_bonus: None,
            mode: Mode::Standard,
            weather_news: WeatherNews::Paused,
            told: None,
            hurt_at: None,
            healed_at: None,
            torch_switch: true,
            torch_steady_at: 0,
            torch_luck: 0,
            unhurt: false,
            picked: HashSet::new(),
            burnt: HashMap::new(),
            by_the_fire: 0,
            next_strike_in: None,
            offer: None,
            pending_levels: 0,
            xp_hundredths: 0,
            hp_tenths: 0,
            without_legendary: 0,
            horde_levels_due: Vec::new(),
            rare_levels: 0,
            offer_rare: RARE_PERCENT,
            rerolls: REROLLS,
            progress: Progress::everything(),
            dealt: Progress::everything(),
            ticks: 0,
            rng,
            spawn_debt: 0.0,
            view: DEFAULT_VIEW,
        };
        // It lights up of itself as the night falls, flickering as luck has it.
        world.light_up_torch();
        world
    }

    /// Tells the world how much of it the player sees: half of it each way, in tiles.
    pub fn set_view(&mut self, half: Vec2) {
        self.view = half;
    }

    /// The ticks the run has lasted.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    /// The whole seconds the run has lasted.
    pub fn seconds(&self) -> u32 {
        self.ticks / u32::from(TICKS_PER_SECOND)
    }

    /// The weather now: it worsens as the run goes on.
    pub fn weather(&self) -> Weather {
        Weather::at(self.seconds(), self.mode)
    }

    /// The weather to come after this one: none after the tempest of a standard run.
    pub fn next_weather(&self) -> Option<Weather> {
        weather::next_stage_at(self.seconds(), self.mode).map(|at| Weather::at(at, self.mode))
    }

    /// How far the run is through its stage of weather, from 0 to 1.
    pub fn weather_progress(&self) -> f32 {
        weather::progress(self.seconds(), self.mode)
    }

    /// How much daylight there is now, from 1 (the setting or rising sun) to 0 (the night).
    pub fn daylight(&self) -> f32 {
        weather::daylight(self.ticks, self.mode)
    }

    /// In arcade, how much harder the run has grown: 7% more a minute, compounded (×2 at 10
    /// minutes, ×3.9 at 20); 1 in a standard run.
    pub fn pressure(&self) -> f32 {
        if self.mode == Mode::Standard {
            return 1.0;
        }
        let minutes =
            f32::from(u16::try_from(self.ticks / u32::from(TICKS_PER_SECOND)).unwrap_or(u16::MAX))
                / 60.0;
        ARCADE_GROWTH.powf(minutes)
    }

    /// The HP of an enemy appearing now: by the weather in a standard run, by the pressure in
    /// arcade.
    fn toughened(&self, hp: u32) -> u32 {
        match self.mode {
            Mode::Standard => hp * self.weather().hp_percent() / 100,
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a few hundred HP at most"
            )]
            Mode::Arcade => {
                (f32::from(u16::try_from(hp).unwrap_or(u16::MAX)) * self.pressure()).round() as u32
            }
        }
    }

    /// How far around the player the night is lit, in tiles: their light, less what the
    /// weather takes, never under 4.
    pub fn light_radius(&self) -> f32 {
        (self.player.build.light_radius() - self.weather().light_loss()).max(4.0)
    }

    /// Where Glare slows the enemies, and how much, in percent: around the player, as far as
    /// their torch's light shows, fading included, as a campfire heals; none without Glare, or
    /// the torch not lit.
    fn glare(&self) -> Option<(f32, u32)> {
        let percent = self.player.build.glare_percent();
        (percent > 0 && self.torch_lit()).then(|| (self.light_radius() + TORCH_FADE, percent))
    }

    /// Whether the player's torch is lit, and holds: not at dusk; it lights up of itself as the
    /// night falls, and when the player lights it again, after flickering a while.
    pub fn torch_lit(&self) -> bool {
        let Some(true) = self.torch_on() else {
            return false;
        };
        self.ticks >= self.torch_steady_at
    }

    /// Whether the torch shines now: lit, or flickering as it lights up, on and off as luck has
    /// it. Flickering, it lights nothing for the run: the player isn't seen, the guns hold.
    pub fn torch_shines(&self) -> bool {
        if self.torch_lit() {
            return true;
        }
        let flickering = self.torch_on() == Some(true) && self.ticks < self.torch_steady_at;
        let luck = ((self.ticks / 2) ^ self.torch_luck)
            .wrapping_mul(0x9E37_79B9)
            .rotate_left(7);
        flickering && luck.is_multiple_of(3)
    }

    /// The torch starts to light up: it flickers, as long as luck has it (maybe not at all),
    /// before it holds.
    fn light_up_torch(&mut self) {
        self.torch_steady_at =
            self.ticks.max(torch_lights_up()) + self.rng.random_range(0..=TORCH_FLICKER_MOST);
        self.torch_luck = self.rng.random();
    }

    /// Whether the player has their torch on: none before it lights up of itself, the night
    /// falling.
    pub fn torch_on(&self) -> Option<bool> {
        (self.ticks >= torch_lights_up()).then_some(self.torch_switch)
    }

    /// Puts the torch out, or lights it again, flickering: once it has lit up of itself.
    pub fn switch_torch(&mut self) {
        match self.torch_on() {
            None => {}
            Some(true) => self.torch_switch = false,
            Some(false) => {
                self.torch_switch = true;
                self.light_up_torch();
            }
        }
    }

    /// Whether the enemies see the player from afar: by daylight, or by their torch.
    fn seen_from_afar(&self) -> bool {
        self.torch_on().is_none() || self.torch_lit()
    }

    /// Tells news, as the player wants it: the world standing still while it shows, or going on.
    fn tell(&mut self, news: News) {
        if self.weather_news == WeatherNews::Paused {
            self.announcement = Some(Announcement::new(news));
        } else {
            self.told = Some((news, self.ticks));
        }
    }

    /// How many enemies of the hordes come are still standing.
    pub fn horde_left(&self) -> u32 {
        self.tallies.values().map(|tally| tally.left).sum()
    }

    /// The hordes come and still standing, in the order they came: each one's number, and how
    /// many of it still stand.
    pub fn hordes_left(&self) -> Vec<(u32, u32)> {
        let mut hordes: Vec<(u32, u32)> = self
            .tallies
            .iter()
            .map(|(&number, tally)| (number, tally.left))
            .collect();
        hordes.sort_unstable();
        hordes
    }

    /// Whether the player has fallen: the world no longer moves on.
    pub fn is_over(&self) -> bool {
        self.player.hp == 0 || self.survived()
    }

    /// Whether the player has held out until the dawn.
    pub fn survived(&self) -> bool {
        self.dawn_bonus.is_some()
    }

    /// Whether the sun has risen: for a few seconds, the enemies flee it and burn, nothing
    /// comes, nothing hurts the player; then the run is won.
    pub fn daybreak(&self) -> bool {
        self.mode == Mode::Standard && self.ticks >= weather::dawn()
    }

    /// Moves the world on by one tick; not while the run is over or a choice is on offer, and not
    /// while news is told: the announcement only wears off.
    pub fn update(&mut self, input: Input) {
        if self.is_over() || self.offer.is_some() {
            return;
        }
        if let Some(announcement) = &mut self.announcement {
            announcement.left = announcement.left.saturating_sub(1);
            if announcement.left == 0 {
                self.announcement = None;
            }
            return;
        }
        let weather = self.weather();
        self.ticks += 1;
        if self.mode == Mode::Standard && self.ticks >= weather::dawn() + weather::daybreak() {
            let bonus = self.score * score::DAWN_BONUS_PERCENT / 100;
            self.score += bonus;
            self.dawn_bonus = Some(bonus);
            self.progress.reach(Achievement::Dawn);
            return;
        }
        if self.weather() != weather {
            self.tell(News::Weather(self.weather()));
        }
        self.move_player(input);
        self.spawn();
        self.hordes();
        self.move_enemies();
        for enemy in &mut self.enemies {
            enemy.scorch();
        }
        self.burn_enemies();
        self.relocate_stragglers();
        self.storm();
        self.swing_melee();
        self.throw_molotovs();
        self.fire_guns();
        self.move_bullets();
        self.turn_drones();
        self.burst_abominations();
        self.fell_enemies();
        self.gather_gems();
        self.give_horde_levels();
        self.illuminate();
        self.regenerate();
        self.heal_player();
        self.hurt_player();
        self.reach_goals();
        if self.pending_levels > 0 && !self.is_over() {
            self.offer = Some(self.deal());
        }
    }

    /// The achievements of the run reached so far: the weather held out until, and those
    /// counted over every run. The cards they unlock are dealt at once.
    fn reach_goals(&mut self) {
        let weather = self.weather();
        if matches!(weather, Weather::Rain | Weather::Storm | Weather::Tempest) {
            self.progress.reach(Achievement::Rain);
        }
        if matches!(weather, Weather::Storm | Weather::Tempest) {
            self.progress.reach(Achievement::Storm);
        }
        self.progress.count();
    }

    /// Picks the choice at `index` of the offer: then the next level's offer, or the world
    /// goes on.
    pub fn choose(&mut self, index: usize) {
        let Some(choice) = self
            .offer
            .as_ref()
            .and_then(|offer| offer.get(index))
            .copied()
        else {
            return;
        };
        let player = &mut self.player;
        match choice {
            Choice::Heal => player.hp = (player.hp + HEAL).min(player.max_hp()),
            Choice::Passive(Passive::Vitality) => {
                player.build.take(choice);
                player.hp += VITALITY_HP;
            }
            _ => player.build.take(choice),
        }
        self.pending_levels -= 1;
        self.offer = (self.pending_levels > 0).then(|| self.deal());
    }

    /// The cards on offer dealt again, as rare as they were, if the run has a reroll left.
    pub fn reroll(&mut self) {
        if self.offer.is_none() || self.rerolls == 0 {
            return;
        }
        self.rerolls -= 1;
        self.offer = Some(self.deal_as(self.offer_rare));
    }

    /// The cards of a level-up: rarer for a level a horde brought.
    fn deal(&mut self) -> Vec<Choice> {
        let rare = if self.rare_levels > 0 {
            self.rare_levels -= 1;
            HORDE_RARE_PERCENT
        } else {
            RARE_PERCENT
        };
        self.offer_rare = rare;
        self.deal_as(rare)
    }

    /// Cards, each a rare one `rare` percent of the time. The longer without a legendary, the
    /// likelier one: a point of percent more for each level-up without, from the first chance
    /// on.
    fn deal_as(&mut self, rare: u32) -> Vec<Choice> {
        let dusk = self.weather() == Weather::Dusk;
        let percent = LEGENDARY_PERCENT + self.without_legendary;
        let progress = &self.dealt;
        let offer = self
            .player
            .build
            .offers(&mut self.rng, dusk, rare, percent, |choice| {
                progress.deals(choice)
            });
        if offer
            .iter()
            .any(|choice| matches!(choice, Choice::Legendary(_)))
        {
            self.without_legendary = 0;
        } else {
            self.without_legendary += 1;
        }
        offer
    }

    /// The Molotov: bottles fly on and burst into puddles; puddles burn on, setting every enemy
    /// in them on fire; once it can, and an enemy is near, bottles are thrown again. Not in the
    /// dark: nothing to aim at.
    fn throw_molotovs(&mut self) {
        let mut landed = Vec::new();
        self.bottles.retain_mut(|bottle| {
            bottle.left = bottle.left.saturating_sub(1);
            if bottle.left == 0 {
                landed.push(Puddle {
                    centre: bottle.to,
                    radius: bottle.radius,
                    left: bottle.lasts,
                    lasts: bottle.lasts,
                });
            }
            bottle.left > 0
        });
        self.puddles.extend(landed);
        for puddle in &mut self.puddles {
            puddle.left = puddle.left.saturating_sub(1);
            for enemy in &mut self.enemies {
                if (enemy.position - puddle.centre).length() <= puddle.radius {
                    enemy.ignite();
                }
            }
        }
        self.puddles.retain(|puddle| puddle.left > 0);
        let level = self.player.build.weapon_level(Weapon::Molotov);
        if level == 0 {
            return;
        }
        self.molotov_wait = self.molotov_wait.saturating_sub(1);
        let dark = self.torch_on().is_some() && !self.torch_lit();
        if self.molotov_wait > 0 || dark {
            return;
        }
        let player = self.player.position;
        let mut near: Vec<Vec2> = self
            .enemies
            .iter()
            .map(|enemy| enemy.position)
            .filter(|at| (*at - player).length() <= molotov::RANGE)
            .collect();
        if near.is_empty() {
            return;
        }
        let throw = molotov::at(level);
        for _ in 0..throw.bottles.min(near.len()) {
            let to = near.swap_remove(self.rng.random_range(0..near.len()));
            self.bottles.push(Bottle {
                from: player,
                to,
                left: molotov::FLIGHT_TICKS,
                radius: throw.radius,
                lasts: throw.lasts,
            });
        }
        self.molotov_wait = self.player.build.cooldown(throw.every);
    }

    /// The melee weapons swing once they can and an enemy is in reach; the slashes of a swing
    /// follow each other, ahead then behind in turn.
    fn swing_melee(&mut self) {
        if let Some(swing) = &mut self.swing {
            swing.left -= 1;
            if swing.left == 0 {
                self.swing = None;
            }
        }
        for weapon in [Weapon::Crowbar, Weapon::Machete] {
            let level = self.player.build.weapon_level(weapon);
            let Some(stats) = machete::melee(weapon, level).filter(|_| level > 0) else {
                continue;
            };
            let mut swinging = self.melee.get(&weapon).copied().unwrap_or_default();
            self.swing_once(weapon, stats, &mut swinging);
            self.melee.insert(weapon, swinging);
        }
    }

    /// One melee weapon on by a tick: its next slash, or a new swing.
    fn swing_once(&mut self, weapon: Weapon, stats: Melee, swinging: &mut Swinging) {
        swinging.wait = swinging.wait.saturating_sub(1);
        if swinging.left > 0 {
            if swinging.gap > 0 {
                swinging.gap -= 1;
            } else {
                self.slash(weapon, stats, swinging);
                swinging.left -= 1;
                swinging.gap = machete::SLASH_GAP - 1;
            }
            return;
        }
        if swinging.wait > 0 {
            return;
        }
        let player = self.player.position;
        let near = self.enemies.iter().any(|enemy| {
            machete::in_reach(
                enemy.position - player,
                enemy.kind.stats().half_width,
                stats.reach,
            )
        });
        if !near {
            return;
        }
        swinging.done = 0;
        self.slash(weapon, stats, swinging);
        swinging.left = stats.slashes - 1;
        swinging.gap = machete::SLASH_GAP - 1;
        swinging.wait = self.player.build.cooldown(stats.cooldown);
    }

    /// A slash: it hits every enemy in its arc, the way the player faces for a swing's first and
    /// third slashes, behind them for its second; pushing them back, stunning them, as the
    /// weapon does.
    fn slash(&mut self, weapon: Weapon, stats: Melee, swinging: &mut Swinging) {
        let player = self.player.position;
        let facing = if swinging.done % 2 == 1 {
            self.player.facing.opposite()
        } else {
            self.player.facing
        };
        swinging.done += 1;
        let swing = Swing {
            weapon,
            facing,
            left: machete::SWING_TICKS,
            reach: stats.reach,
            half_arc: stats.half_arc,
        };
        let ahead = swing.facing.way();
        let damage = self.player.build.damage(stats.damage);
        for enemy in &mut self.enemies {
            let offset = enemy.position - player;
            let enemy_stats = enemy.kind.stats();
            let hit = machete::in_arc(
                offset,
                enemy_stats.half_width,
                swing.reach,
                ahead,
                swing.half_arc,
            );
            if !hit {
                continue;
            }
            enemy.hit(damage);
            enemy.stunned = enemy.stunned.max(stats.stun);
            if stats.knockback > 0.0 {
                let away = offset.normalized() * stats.knockback;
                enemy.position = slide(
                    &self.map,
                    enemy.position,
                    away,
                    enemy_stats.half_width,
                    enemy_stats.flies,
                );
            }
        }
        self.swing = Some(swing);
    }

    /// Lightning strikes now and then, as often as the weather has it, somewhere the player
    /// sees; from the storm on, it hurts the enemies it falls among, and the player: its ring
    /// shows where a second before it falls, often where the player stands.
    fn storm(&mut self) {
        let weather = self.weather();
        let due = self.ticks.saturating_sub(weather::STRIKE_WARNING_TICKS);
        let (falling, coming): (Vec<_>, Vec<_>) = self
            .strikes_coming
            .iter()
            .partition(|strike| strike.tick <= due);
        self.strikes_coming = coming;
        for strike in falling {
            self.strike(strike.at, true);
        }
        let rising = self.mode == Mode::Standard && self.ticks >= weather::sunrise();
        let Some(every) = weather.lightning_every().filter(|_| !rising) else {
            self.next_strike_in = None;
            return;
        };
        let wait = self.next_strike_in.get_or_insert(every);
        if *wait > 0 {
            *wait -= 1;
            return;
        }
        // Not like clockwork: between half and one and a half times the weather's wait.
        self.next_strike_in = Some(self.rng.random_range(every / 2..=every * 3 / 2));
        let (view, centre) = (self.view, self.player.position);
        let aimed = self.rng.random_range(0..100) < weather.aimed_strike_percent();
        let at = if aimed {
            centre
        } else {
            centre
                + Vec2::new(
                    self.rng.random_range(-view.x..=view.x),
                    self.rng.random_range(-view.y..=view.y),
                )
        };
        if weather.strikes_hurt() {
            self.strikes_coming.push(Lightning {
                tick: self.ticks,
                at,
            });
        } else {
            self.strike(at, false);
        }
    }

    /// A strike of lightning falls at `at`, hurting the enemies around it, and the player, if
    /// `hurts`.
    fn strike(&mut self, at: Vec2, hurts: bool) {
        self.lightning = Some(Lightning {
            tick: self.ticks,
            at,
        });
        // The whole map lit: every enemy on screen sees the player.
        self.give_away(self.view.x.max(self.view.y));
        if !hurts {
            return;
        }
        for enemy in &mut self.enemies {
            if (enemy.position - at).length() <= weather::STRIKE_RADIUS {
                enemy.hit(weather::STRIKE_DAMAGE);
            }
        }
        if (self.player.position - at).length() <= weather::STRIKE_RADIUS
            && !self.unhurt
            && !self.daybreak()
            && self.player.dash.is_none()
        {
            self.wound_player(weather::STRIKE_PLAYER_DAMAGE);
            if self.player.hp > 0 {
                self.progress.reach(Achievement::Struck);
            }
        }
    }

    /// Each gun the player has shoots at the nearest enemies in its range, while its magazine
    /// holds rounds; empty, it reloads.
    fn fire_guns(&mut self) {
        // In the dark, the torch out, nothing to aim at: the guns hold their fire.
        let dark = self.torch_on().is_some() && !self.torch_lit();
        let Self {
            player,
            enemies,
            bullets,
            magazines,
            rng,
            ..
        } = self;
        let build = &player.build;
        for weapon in GUNS {
            let level = build.weapon_level(weapon);
            let Some(gun) = guns::gun(weapon, level).filter(|_| level > 0) else {
                continue;
            };
            let magazine = magazines.entry(weapon).or_insert(Magazine {
                rounds: gun.magazine,
                ..Magazine::default()
            });
            if magazine.reloading > 0 {
                magazine.reloading -= 1;
                if magazine.reloading == 0 {
                    magazine.rounds = gun.magazine;
                }
                continue;
            }
            if magazine.wait > 0 {
                magazine.wait -= 1;
                continue;
            }
            if dark {
                continue;
            }
            let targets = guns::targets(enemies, player.position, gun.bullets, gun.range);
            if targets.is_empty() {
                continue;
            }
            let damage = build.damage(gun.damage);
            for target in targets {
                bullets.extend(Bullet::burst(player.position, target, &gun, damage, rng));
            }
            magazine.rounds = magazine.rounds.saturating_sub(1);
            magazine.wait = gun.interval;
            if magazine.rounds == 0 {
                // Reloaded, the gun shoots at once: the reload is its wait.
                magazine.wait = 0;
                magazine.reloading = build.cooldown(gun.reload).max(1);
            }
        }
    }

    /// Bullets fly on, hitting each enemy they touch once; gone once they have gone through as
    /// many as they can, or flown their range.
    fn move_bullets(&mut self) {
        let enemies = &mut self.enemies;
        self.bullets.retain_mut(|bullet| {
            if !bullet.advance() {
                return false;
            }
            for enemy in enemies.iter_mut() {
                let touched = touch(
                    bullet.position,
                    guns::BULLET_HALF_WIDTH,
                    enemy.position,
                    enemy.kind.stats().half_width,
                );
                if enemy.hp == 0 || !touched || bullet.hit.contains(&enemy.id) {
                    continue;
                }
                enemy.hit(bullet.damage);
                bullet.hit.push(enemy.id);
                if bullet.pierce == 0 {
                    return false;
                }
                bullet.pierce -= 1;
            }
            true
        });
    }

    /// The guns the player has, each with its magazine, how many rounds it holds full, and the
    /// ticks a whole reload takes.
    pub fn ammo(&self) -> Vec<(Weapon, Magazine, u32, u32)> {
        GUNS.into_iter()
            .filter_map(|weapon| {
                let level = self.player.build.weapon_level(weapon);
                let gun = guns::gun(weapon, level).filter(|_| level > 0)?;
                let magazine = self.magazines.get(&weapon).copied().unwrap_or(Magazine {
                    rounds: gun.magazine,
                    ..Magazine::default()
                });
                let reload = self.player.build.cooldown(gun.reload).max(1);
                Some((weapon, magazine, gun.magazine, reload))
            })
            .collect()
    }

    /// Where the drones are, if the player has them, and whether each is broken.
    pub fn drones(&self) -> Vec<(Vec2, bool)> {
        let level = self.player.build.weapon_level(Weapon::Drones);
        drones::positions(self.player.position, level, self.drone_angle)
            .into_iter()
            .enumerate()
            .map(|(index, position)| {
                let broken = self.drone_repairs.get(index).is_some_and(|left| *left > 0);
                (position, broken)
            })
            .collect()
    }

    /// The drones turn on; each that works hits the first enemy it runs into, pushes it back
    /// from the player, and breaks. A broken drone repairs with time.
    fn turn_drones(&mut self) {
        let build = &self.player.build;
        let level = build.weapon_level(Weapon::Drones);
        if level == 0 {
            return;
        }
        self.drone_angle =
            (self.drone_angle + drones::turn_per_tick(level)) % std::f32::consts::TAU;
        let damage = build.damage(drones::damage(level));
        let repair = build.cooldown(drones::repair());
        let player = self.player.position;
        let drones = self.drones();
        self.drone_repairs.resize(drones.len(), 0);
        for (index, (position, broken)) in drones.into_iter().enumerate() {
            if broken {
                self.drone_repairs[index] -= 1;
                continue;
            }
            let hit = self.enemies.iter_mut().find(|enemy| {
                enemy.hp > 0
                    && touch(
                        position,
                        drones::HALF_WIDTH,
                        enemy.position,
                        enemy.kind.stats().half_width,
                    )
            });
            let Some(enemy) = hit else {
                continue;
            };
            let stats = enemy.kind.stats();
            enemy.hit(damage);
            let away = (enemy.position - player).normalized() * drones::KNOCKBACK;
            enemy.position = slide(
                &self.map,
                enemy.position,
                away,
                stats.half_width,
                stats.flies,
            );
            self.drone_repairs[index] = repair.max(1);
        }
    }

    /// The score's multiplier now, in tenths: the weather's times the combo's.
    pub fn multiplier_tenths(&self) -> u64 {
        // In arcade, the pressure; in a standard run, scored for nothing, the weather.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a multiplier of a few dozen tenths"
        )]
        let base = match self.mode {
            Mode::Standard => score::weather_tenths(self.weather()),
            Mode::Arcade => (self.pressure() * 10.0).round() as u64,
        };
        base * score::combo_tenths(self.combo) / 10
    }

    /// Enemies out of HP fall, each worth its points, some leaving a gem. The combo closes when
    /// no enemy has fallen for a while.
    fn fell_enemies(&mut self) {
        if self.ticks - self.last_kill > score::combo_window() {
            self.combo = 0;
        }
        let mut wiped_out = 0;
        let weather = self.weather();
        for enemy in self.enemies.iter().filter(|enemy| enemy.hp == 0) {
            // Worth the combo as it stood before this kill.
            let worth = score::worth(enemy.kind, weather, self.combo);
            self.progress.kills += 1;
            if enemy.kind == EnemyKind::Shadow {
                self.progress.shadows += 1;
            }
            self.score += worth;
            self.combo += 1;
            self.last_kill = self.ticks;
            let stats = enemy.kind.stats();
            // A horde wiped out to the last brings a level; in arcade, it doubles what its kills
            // were worth too.
            if let Some(number) = enemy.horde
                && let Some(tally) = self.tallies.get_mut(&number)
            {
                tally.points += worth;
                tally.left -= 1;
                if tally.left == 0 {
                    let bonus = if self.mode == Mode::Arcade {
                        tally.points
                    } else {
                        0
                    };
                    self.score += bonus;
                    self.tallies.remove(&number);
                    let others = !self.tallies.is_empty();
                    self.horde_cleared = Some((self.ticks, bonus, others.then_some(number)));
                    wiped_out += 1;
                }
            }
            let drop = (stats.drop_percent + self.player.build.drop_bonus()).min(100);
            if stats.xp > 0 && self.rng.random_range(0..100) < drop {
                self.gems.push(Gem::new(enemy.position, stats.xp));
            }
            self.stains.push(Stain {
                position: enemy.position,
                tick: self.ticks,
                size: stats.stain,
                ash: enemy.burning > 0,
            });
            self.kills += 1;
        }
        let lasts = stain::lasts();
        let now = self.ticks;
        self.stains.retain(|stain| now - stain.tick < lasts);
        if self.stains.len() > MAX_STAINS {
            self.stains.drain(..self.stains.len() - MAX_STAINS);
        }
        // Its level comes once the horde wiped out has been told.
        for _ in 0..wiped_out {
            self.horde_levels_due
                .push(self.ticks + HORDE_LEVEL_DELAY * u32::from(TICKS_PER_SECOND));
        }
        let swelling = self
            .enemies
            .iter()
            .filter(|enemy| enemy.hp == 0 && enemy.kind == EnemyKind::Abomination)
            .map(|enemy| Swelling {
                position: enemy.position,
                facing: enemy.facing,
                left: burst::SWELL_TICKS,
            });
        self.swellings.extend(swelling.collect::<Vec<_>>());
        self.enemies.retain(|enemy| enemy.hp > 0);
        if self.gems.len() > MAX_GEMS {
            self.gems.drain(..self.gems.len() - MAX_GEMS);
        }
    }

    /// The felled abominations swell on, and burst once they have: a disc of bile that hurts
    /// and pushes back everything within its reach, the parasites thrown out of it.
    fn burst_abominations(&mut self) {
        for blast in &mut self.blasts {
            blast.age += 1;
        }
        self.blasts.retain(Blast::shows);
        let mut bursting = Vec::new();
        self.swellings.retain_mut(|swelling| {
            swelling.left = swelling.left.saturating_sub(1);
            if swelling.left == 0 {
                bursting.push(swelling.position);
            }
            swelling.left > 0
        });
        for at in bursting {
            self.burst(at);
        }
    }

    /// An abomination bursts at `at`.
    fn burst(&mut self, at: Vec2) {
        self.blasts.push(Blast { centre: at, age: 0 });
        for enemy in &mut self.enemies {
            let away = enemy.position - at;
            if away.length() > burst::REACH {
                continue;
            }
            enemy.hit(burst::DAMAGE);
            let stats = enemy.kind.stats();
            enemy.position = shove(
                &self.map,
                enemy.position,
                away.normalized() * burst::PUSH,
                stats.half_width,
                stats.flies,
            );
        }
        let away = self.player.position - at;
        let dashing = self.player.dash.is_some();
        if away.length() <= burst::REACH && !dashing {
            if !self.unhurt && !self.daybreak() && self.player.invincible == 0 {
                self.wound_player(burst::DAMAGE);
            }
            self.player.position = shove(
                &self.map,
                self.player.position,
                away.normalized() * burst::PUSH,
                PLAYER_HALF_WIDTH,
                false,
            );
        }
        self.let_out_parasites(at);
    }

    /// A burst abomination's parasites, thrown out around it, each its own way, as tough as the
    /// weather makes them; nearer where an obstacle stands in the way.
    fn let_out_parasites(&mut self, at: Vec2) {
        let stats = EnemyKind::Parasite.stats();
        let first = spawn::phase(&mut self.rng);
        for index in 0..PARASITES {
            let angle = first + std::f32::consts::TAU * f32::from(index) / f32::from(PARASITES);
            let way = Vec2::new(angle.cos(), angle.sin());
            let spot = shove(
                &self.map,
                at,
                way * burst::THROW,
                stats.half_width,
                stats.flies,
            );
            let mut parasite = Enemy::new(self.next_enemy_id, EnemyKind::Parasite, spot, angle);
            parasite.hp = self.toughened(parasite.hp);
            self.enemies.push(parasite);
            self.next_enemy_id += 1;
        }
    }

    /// A debug tool: the run jumps to just before the next weather, which is then told; in the
    /// tempest, to the sunrise.
    pub fn debug_next_weather(&mut self) {
        self.ticks = match weather::next_stage_at(self.seconds(), self.mode) {
            Some(next) => next * u32::from(TICKS_PER_SECOND) - 1,
            None => self.ticks.max(weather::sunrise()),
        };
        self.after_a_jump();
    }

    /// After a debug jump in time: no horde owed from the time skipped, the next one in the
    /// middle of its stage.
    fn after_a_jump(&mut self) {
        self.horde_warning = None;
        self.next_horde = horde::due_after(self.ticks + horde::warning(), self.mode);
    }

    /// A debug tool: the next horde is announced at once.
    pub fn debug_horde(&mut self) {
        if self.horde_warning.is_none() {
            self.next_horde = self.ticks + horde::warning();
        }
    }

    /// A debug tool: the run jumps to 10 seconds before the dawn.
    pub fn debug_dawn(&mut self) {
        let before = weather::dawn() - 10 * u32::from(TICKS_PER_SECOND);
        self.ticks = self.ticks.max(before);
        self.after_a_jump();
    }

    /// A debug tool: the player gets the XP left to their next level, and its cards.
    pub fn debug_level_up(&mut self) {
        let missing = xp_to_next(self.player.level).saturating_sub(self.player.xp);
        self.pending_levels += self.player.gain(missing);
    }

    /// A debug tool: the enemies within a few tiles of the player catch fire.
    pub fn debug_fire(&mut self) {
        let player = self.player.position;
        for enemy in &mut self.enemies {
            if (enemy.position - player).length() <= DEBUG_FIRE_REACH {
                enemy.ignite();
            }
        }
    }

    /// A debug tool: a burning campfire a few tiles to the player's right.
    pub fn debug_campfire(&mut self) {
        let (x, y) = self.player.position.tile();
        let tile = (x + 4, y);
        self.map.put_feature(tile, Feature::Campfire);
        self.burnt.remove(&tile);
    }

    /// A debug tool: every achievement reached, every card unlocked, from the next run on.
    pub fn debug_unlock_all(&mut self) {
        for achievement in Achievement::ALL {
            self.progress.reach(achievement);
        }
    }

    /// A debug tool: the player's HP all back.
    pub fn debug_heal(&mut self) {
        self.player.hp = self.player.max_hp();
    }

    /// Gems near the player fly to them; those they reach give their XP.
    fn gather_gems(&mut self) {
        let player = &self.player;
        let magnet = player.build.magnet_radius();
        let position = player.position;
        let mut gathered = 0;
        self.gems.retain_mut(|gem| {
            let picked = gem.update(position, magnet);
            if picked {
                gathered += gem.value;
            }
            !picked
        });
        self.give_xp(gathered * self.player.build.xp_percent());
    }

    /// The levels hordes wiped out have brought, once due: each its cards, rarer.
    fn give_horde_levels(&mut self) {
        let now = self.ticks;
        let due = self
            .horde_levels_due
            .iter()
            .filter(|at| **at <= now)
            .count();
        self.horde_levels_due.retain(|at| *at > now);
        for _ in 0..due {
            let missing = xp_to_next(self.player.level).saturating_sub(self.player.xp);
            self.pending_levels += self.player.gain(missing);
            self.rare_levels += 1;
        }
    }

    /// Regeneration gives HP back of itself, every second, tenths kept until they make a whole
    /// one.
    fn regenerate(&mut self) {
        if !self.ticks.is_multiple_of(u32::from(TICKS_PER_SECOND)) {
            return;
        }
        self.hp_tenths += self.player.build.regeneration_tenths();
        let whole = self.hp_tenths / 10;
        self.hp_tenths %= 10;
        self.player.hp = (self.player.hp + whole).min(self.player.max_hp());
    }

    /// Illumination gives XP of itself, a point a second.
    fn illuminate(&mut self) {
        if self.player.build.has(Legendary::Illumination)
            && self.ticks.is_multiple_of(u32::from(TICKS_PER_SECOND))
        {
            self.give_xp(100);
        }
    }

    /// XP for the player, in hundredths of a point: the whole points now, the rest kept for the
    /// next.
    fn give_xp(&mut self, hundredths: u32) {
        self.xp_hundredths += hundredths;
        let whole = self.xp_hundredths / 100;
        self.xp_hundredths %= 100;
        self.pending_levels += self.player.gain(whole);
    }

    fn move_player(&mut self, input: Input) {
        let player = &mut self.player;
        let (most, recharge) = (player.build.dashes(), player.build.dash_recharge());
        player.dashes.tick(most, recharge);
        // Dashing, the player goes on the way they dash, whatever the keys say.
        if let Some(dash) = &mut player.dash {
            let step = dash.step();
            dash.left -= 1;
            if dash.left == 0 {
                player.dash = None;
            }
            player.position = slide(&self.map, player.position, step, PLAYER_HALF_WIDTH, false);
            return;
        }
        player.facing = player.facing.after(input.direction);
        // Normalized, so a diagonal is no faster than a straight line.
        let speed = PLAYER_SPEED * f32::from(player.build.speed_percent()) / 100.0;
        let step = input.direction.normalized() * (speed * tick_seconds());
        player.position = slide(&self.map, player.position, step, PLAYER_HALF_WIDTH, false);
    }

    /// The player dashes the way they face, if they have the Dash card and a dash is ready:
    /// nothing hurts them while they do.
    pub fn dash(&mut self) {
        let player = &mut self.player;
        if self.announcement.is_some()
            || player.dash.is_some()
            || player.build.dashes() == 0
            || !player.dashes.take()
        {
            return;
        }
        player.dash = Some(Dash::new(player.facing.way()));
    }

    /// New enemies, as many as the rate owes, up to the most at once.
    fn spawn(&mut self) {
        if self.daybreak() {
            self.spawn_debt = 0.0;
            return;
        }
        if self.enemies.len() >= MAX_ENEMIES {
            self.spawn_debt = 0.0;
            return;
        }
        let weather = self.weather();
        // By the weather in a standard run; by the pressure in arcade.
        let rate = match self.mode {
            Mode::Standard => spawn::rate(weather, self.weather_progress()),
            Mode::Arcade => ARCADE_RATE * self.pressure(),
        };
        self.spawn_debt += rate * tick_seconds();
        while self.spawn_debt >= 1.0 {
            self.spawn_debt -= 1.0;
            if self.enemies.len() >= MAX_ENEMIES {
                break;
            }
            let kind = spawn::pick_kind(&mut self.rng, weather);
            if let Some(position) = self.free_edge_point(kind, None) {
                let phase = spawn::phase(&mut self.rng);
                let mut enemy = Enemy::new(self.next_enemy_id, kind, position, phase);
                enemy.hp = self.toughened(enemy.hp);
                self.enemies.push(enemy);
                self.next_enemy_id += 1;
            }
        }
    }

    /// Announces the next horde a moment before it comes, then brings it: a pack out of one side,
    /// or a ring out of every side from the storm on.
    fn hordes(&mut self) {
        if self.daybreak() {
            self.horde_warning = None;
            return;
        }
        if self.horde_warning.is_none() && self.ticks + horde::warning() >= self.next_horde {
            // Out of the other side from the enemies about: the player caught between.
            let side = if horde::rings(self.weather()) {
                None
            } else {
                Some(horde::pincer(
                    &mut self.rng,
                    self.player.position,
                    self.view,
                    self.enemies.iter().map(|enemy| enemy.position),
                ))
            };
            self.horde_warning = Some(Warning {
                arrives_at: self.next_horde,
                side,
                number: self.hordes_come,
            });
            self.tell(News::Horde {
                everywhere: side.is_none(),
            });
        }
        if let Some(warning) = self.horde_warning
            && self.ticks >= warning.arrives_at
        {
            self.bring_horde(warning);
            self.horde_warning = None;
            self.hordes_come += 1;
            // From when it came: never a horde owed from the past.
            self.next_horde = self.ticks + horde::every(self.mode);
        }
    }

    /// A horde comes: its enemies, as many as fit, just beyond what the player sees.
    fn bring_horde(&mut self, warning: Warning) {
        let weather = self.weather();
        let room = MAX_ENEMIES_WITH_HORDE.saturating_sub(self.enemies.len());
        let count = horde::size(warning.number).min(u32::try_from(room).unwrap_or(0));
        let spots = horde::spots(
            &mut self.rng,
            self.player.position,
            self.view,
            warning.side,
            count,
        );
        let mut tally = Tally::default();
        for spot in spots {
            let kind = horde::kind(&mut self.rng, weather);
            let stats = kind.stats();
            if !fits(&self.map, spot, stats.half_width, stats.flies) {
                continue;
            }
            let phase = spawn::phase(&mut self.rng);
            let mut enemy = Enemy::new(self.next_enemy_id, kind, spot, phase);
            enemy.hp = self.toughened(enemy.hp);
            enemy.horde = Some(warning.number);
            self.enemies.push(enemy);
            self.next_enemy_id += 1;
            tally.left += 1;
        }
        if tally.left > 0 {
            self.tallies.insert(warning.number, tally);
        }
    }

    /// A point just beyond what the player sees where an enemy of that kind fits: on `side`,
    /// or on any. None if the tries all fall on obstacles or off the map.
    fn free_edge_point(&mut self, kind: EnemyKind, side: Option<Facing>) -> Option<Vec2> {
        let stats = kind.stats();
        (0..SPAWN_TRIES)
            .map(|_| spawn::edge_point(&mut self.rng, self.player.position, self.view, side))
            .find(|point| fits(&self.map, *point, stats.half_width, stats.flies))
    }

    /// Each enemy goes for the player, pushed apart from the others it overlaps, and stops on
    /// touching the player.
    fn move_enemies(&mut self) {
        let pushes = self.separations();
        let target = self.player.position;
        let weather = self.weather();
        // At daybreak, they flee the player, the risen sun at their back.
        let flee = self.daybreak();
        // In the dark, those that haven't seen the player go for the campfires, or wander.
        let seen = self.seen_from_afar();
        let fires = if seen { Vec::new() } else { self.fires_near() };
        // At night, whether Shadows hunt: with the torch out; lit, they shun it.
        let dark_hunt = self.torch_on().map(|_| !self.torch_lit());
        // With Glare, those in the torch's light go slower.
        let glare = self.glare();
        let ticks = self.ticks;
        for (enemy, push) in self.enemies.iter_mut().zip(pushes) {
            enemy.flash = enemy.flash.saturating_sub(1);
            let stats = enemy.kind.stats();
            let apart = (target - enemy.position).length();
            if enemy.horde.is_some() || enemy.kind == EnemyKind::Parasite {
                enemy.noticed = true;
            } else if enemy.kind == EnemyKind::Shadow && dark_hunt.is_some() {
                // Shadows hunt in the dark, faster and faster; caught in the torch's light, they
                // bolt out of it.
                let hunts = dark_hunt == Some(true);
                if enemy.noticed && !hunts {
                    enemy.bolting = BOLT_TICKS;
                }
                enemy.hunting = if hunts { enemy.hunting + 1 } else { 0 };
                enemy.noticed = hunts;
            } else if seen || apart <= NOTICE_REACH {
                enemy.noticed = true;
            } else if apart > LOSE_REACH {
                enemy.noticed = false;
            }
            let fire = fires
                .iter()
                .copied()
                .filter(|fire| (*fire - enemy.position).length() <= LURE_REACH)
                .min_by(|a, b| {
                    (*a - enemy.position)
                        .length()
                        .total_cmp(&(*b - enemy.position).length())
                });
            let (heading, pace) = if enemy.stunned > 0 {
                enemy.stunned -= 1;
                (Vec2::ZERO, 0.0)
            } else if flee {
                ((enemy.position - target).normalized(), 1.0)
            } else if !enemy.noticed && enemy.kind == EnemyKind::Shadow && dark_hunt == Some(false)
            {
                // Away from the light, at their speed: bolting, at first.
                let pace = if enemy.bolting > 0 {
                    enemy.bolting -= 1;
                    BOLT_PACE
                } else {
                    1.0
                };
                ((enemy.position - target).normalized(), pace)
            } else if !enemy.noticed {
                match fire {
                    Some(fire) if (fire - enemy.position).length() > 1.0 => {
                        (enemy.heading(fire), WANDER_PACE)
                    }
                    Some(_) => (Vec2::ZERO, WANDER_PACE),
                    None => {
                        // A slow turn of its own, never the same as its neighbours'.
                        let id = f32::from(u16::try_from(enemy.id % 1_000).unwrap_or(0));
                        let now = f32::from(u16::try_from(ticks % 10_000).unwrap_or(0));
                        enemy.wander += (now * 0.02 + id).sin() * 0.04;
                        (
                            Vec2::new(enemy.wander.cos(), enemy.wander.sin()),
                            WANDER_PACE,
                        )
                    }
                }
            } else if touch(enemy.position, stats.half_width, target, PLAYER_HALF_WIDTH) {
                (Vec2::ZERO, 1.0)
            } else {
                (enemy.heading(target), enemy.hunt_pace())
            };
            enemy.facing = enemy.facing.after(heading);
            let slowed = match glare {
                Some((reach, percent)) if apart <= reach => percent,
                _ => 0,
            };
            let glared = f32::from(u16::try_from(100 - slowed.min(100)).unwrap_or(100)) / 100.0;
            let speed = stats.speed * f32::from(weather.speed_percent()) / 100.0 * pace * glared;
            let step = (heading * (speed * tick_seconds()) + push).clamp_length(MAX_STEP);
            enemy.position = slide(
                &self.map,
                enemy.position,
                step,
                stats.half_width,
                stats.flies,
            );
        }
    }

    /// The campfires still burning around what the player sees, as far as they lure enemies: in
    /// the dark, those that haven't found the player go for them.
    fn fires_near(&self) -> Vec<Vec2> {
        let (x, y) = self.player.position.tile();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the view and the lure, a few dozen tiles"
        )]
        let reach = (self.view.x.max(self.view.y) + LURE_REACH).ceil() as i32;
        self.features_in((x - reach, y - reach), (x + reach, y + reach))
            .into_iter()
            .filter(|(tile, feature)| {
                *feature == Feature::Campfire && self.campfire_left(*tile) > 0
            })
            .map(|(tile, _)| Vec2::middle_of(tile))
            .collect()
    }

    /// The enemies within `reach` of the player find them: a strike of lightning gives their
    /// place away.
    fn give_away(&mut self, reach: f32) {
        let player = self.player.position;
        for enemy in &mut self.enemies {
            if (enemy.position - player).length() <= reach {
                enemy.noticed = true;
            }
        }
    }

    /// At daybreak, the enemies burn in the sun one by one, each at a moment of its own over the
    /// daybreak, flashing white just before: they leave nothing, and are worth nothing.
    fn burn_enemies(&mut self) {
        if !self.daybreak() {
            return;
        }
        let since = self.ticks - weather::dawn();
        let last = weather::daybreak().saturating_sub(BURN_FLASH).max(1);
        self.enemies.retain_mut(|enemy| {
            let burns_at = enemy.id.wrapping_mul(2_654_435_761) % last;
            if since + BURN_FLASH >= burns_at {
                enemy.flash = enemy.flash.max(1);
            }
            since < burns_at + BURN_FLASH
        });
    }

    /// How far each enemy is pushed by those it overlaps, in the order of `enemies`. Enemies are
    /// sorted into a grid first, so that each meets only those in its own cell and the cells
    /// around: a horde of hundreds stays cheap.
    fn separations(&self) -> Vec<Vec2> {
        let cell_of = |position: Vec2| (position * (1.0 / SEPARATION_CELL)).tile();
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (index, enemy) in self.enemies.iter().enumerate() {
            grid.entry(cell_of(enemy.position)).or_default().push(index);
        }
        let mut pushes = vec![Vec2::ZERO; self.enemies.len()];
        for (i, a) in self.enemies.iter().enumerate() {
            let (cx, cy) = cell_of(a.position);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(others) = grid.get(&(cx + dx, cy + dy)) else {
                        continue;
                    };
                    // Each pair once: the one with the larger index pushes both.
                    for &j in others.iter().filter(|j| **j > i) {
                        let b = &self.enemies[j];
                        let reach = a.kind.stats().half_width + b.kind.stats().half_width;
                        let apart = a.position - b.position;
                        let distance = apart.length();
                        if distance >= reach {
                            continue;
                        }
                        // Two on the very same spot part sideways.
                        let away = if distance > f32::EPSILON {
                            apart * (1.0 / distance)
                        } else {
                            Vec2::new(1.0, 0.0)
                        };
                        let push = away * ((reach - distance) * SEPARATION);
                        pushes[i] += push;
                        pushes[j] -= push;
                    }
                }
            }
        }
        pushes
    }

    /// Enemies left far behind appear again just beyond the side the player faces.
    fn relocate_stragglers(&mut self) {
        let (reach_x, reach_y) = (2.0 * (self.view.x + MARGIN), 2.0 * (self.view.y + MARGIN));
        let centre = self.player.position;
        let facing = self.player.facing;
        for index in 0..self.enemies.len() {
            let enemy = &self.enemies[index];
            let apart = enemy.position - centre;
            if apart.x.abs() <= reach_x && apart.y.abs() <= reach_y {
                continue;
            }
            if let Some(point) = self.free_edge_point(enemy.kind, Some(facing)) {
                self.enemies[index].position = point;
            }
        }
    }

    /// What lies on the ground between two tiles, by tile, as it is now: the herbs not yet
    /// picked, and every campfire, burning or out.
    pub fn features_in(&self, from: (i32, i32), to: (i32, i32)) -> Vec<((i32, i32), Feature)> {
        let mut features = self.map.features_in(from, to);
        features.retain(|(tile, feature)| *feature != Feature::Herb || !self.picked.contains(tile));
        features
    }

    /// The HP a campfire still has to give: none once it is out.
    pub fn campfire_left(&self, tile: (i32, i32)) -> u32 {
        CAMPFIRE_HP - self.burnt.get(&tile).copied().unwrap_or(0).min(CAMPFIRE_HP)
    }

    /// A hurt player picks up the herbs within their reach, and is healed by a burning campfire
    /// they stay by, which gives of itself as it does.
    fn heal_player(&mut self) {
        let player = self.player.position;
        let (x, y) = player.tile();
        // As far as a campfire's light shows at its brightest, fading included: it heals within.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a campfire's light, a few tiles"
        )]
        let around = (2.0 * CAMPFIRE_LIGHT).ceil() as i32 + 1;
        let mut fire = None;
        for (tile, feature) in self.features_in((x - around, y - around), (x + around, y + around))
        {
            let middle = Vec2::middle_of(tile);
            let apart = (middle - player).length();
            let hurt = self.player.hp < self.player.max_hp();
            match feature {
                Feature::Herb if hurt && apart <= HERB_REACH => {
                    self.player.hp = (self.player.hp + HERB_HP).min(self.player.max_hp());
                    self.picked.insert(tile);
                    self.healed_at = Some(self.ticks);
                }
                Feature::Campfire
                    if hurt
                        && self.campfire_left(tile) > 0
                        && apart <= heal::reach(self.campfire_left(tile)) =>
                {
                    fire = Some(tile);
                }
                _ => {}
            }
        }
        let Some(tile) = fire else {
            self.by_the_fire = 0;
            return;
        };
        self.by_the_fire += 1;
        if self.by_the_fire >= heal::ticks_per_hp() {
            self.by_the_fire = 0;
            self.player.hp += 1;
            *self.burnt.entry(tile).or_insert(0) += 1;
            self.healed_at = Some(self.ticks);
        }
    }

    /// The player loses the HP of the hardest-hitting enemy touching them, then nothing hurts
    /// them for half a second.
    fn hurt_player(&mut self) {
        if self.unhurt || self.daybreak() || self.player.dash.is_some() {
            return;
        }
        let player = &mut self.player;
        if player.invincible > 0 {
            player.invincible -= 1;
            return;
        }
        let hit = self
            .enemies
            .iter()
            .filter(|enemy| {
                touch(
                    enemy.position,
                    enemy.kind.stats().half_width,
                    player.position,
                    PLAYER_HALF_WIDTH,
                )
            })
            .map(|enemy| enemy.kind.stats().damage)
            .max();
        if let Some(damage) = hit {
            self.wound_player(damage);
        }
    }

    /// The player loses `damage` HP, bleeds, and can't be hurt again for half a second.
    fn wound_player(&mut self, damage: u32) {
        let player = &mut self.player;
        player.hp = player.hp.saturating_sub(damage);
        player.invincible = u32::from(TICKS_PER_SECOND) / 2;
        self.hurt_at = Some(self.ticks);
        self.stains.push(Stain {
            position: player.position,
            tick: self.ticks,
            size: 1,
            ash: false,
        });
    }
}
