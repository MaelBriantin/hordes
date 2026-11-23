use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::announce::{Announcement, News, WeatherNews};
use super::barricade::Barricade;
use super::baton::Thrust;
use super::body::{slide, touch};
use super::build::{
    Action, BASE_HP, Build, Choice, HEAL, Legendary, Passive, RARE_PERCENT, VITALITY_HP, Weapon,
};
use super::burst::{Blast, Swelling};
use super::c4::{self, Charge};
use super::dash::{Dash, Dashes};
use super::defibrillator::Shock;
use super::enemy::Enemy;
use super::fists::Punch;
use super::gem::Gem;
use super::grenade::Thrown;
use super::guns::{Bullet, Magazine};
use super::hammer::{Blow, Quake};
use super::heal::{self, CAMPFIRE_HP, CAMPFIRE_LIGHT, HERB_HP, HERB_REACH};
use super::horde::{self, Tally, Warning};
use super::katana::Cuts;
use super::knife::Stab;
use super::level::xp_to_next;
use super::machete::Swing;
use super::map::Feature;
use super::mode::Mode;
use super::molotov::{Bottle, Puddle};
use super::napalm::Bomb;
use super::progress::{
    Achievement, CARNAGE_KILLS, COLD_BLOOD_PERCENT, COLD_BLOOD_SECONDS, DEMOLISHER_ABOMINATIONS,
    FIRST_AID_HP, HOLD_THE_LINE_ENEMIES, HOLD_THE_LINE_REACH, HOLD_THE_LINE_SECONDS, LEGEND_KILLS,
    Progress, SHADOW_HUNTER_SHADOWS, SHADOW_SECONDS, VETERAN_LEVEL,
};
use super::score;
use super::sniper::{Impact, Shot};
use super::stain::Stain;
use super::weather::{self, Weather};
use super::{Facing, Map, Skin, TICKS_PER_SECOND, Vec2, tick_seconds};

mod barricades;
mod blows;
mod bombs;
mod cards;
mod charges;
mod cuts;
mod debug;
mod enemies;
mod fire_walls;
mod flames;
mod goals;
mod grenades;
mod jabs;
mod kits;
mod player;
mod punches;
mod rams;
mod saws;
mod shocks;
mod sniping;
mod stabs;
mod storm;
mod torch;
mod weapons;

#[cfg(test)]
mod balance;
#[cfg(test)]
mod tests;

/// The player's speed, in tiles per second.
const PLAYER_SPEED: f32 = 6.0;
/// Half the width of the player's body, in tiles: what obstacles stop, smaller than the sprite
/// so that brushing past one is forgiven.
const PLAYER_HALF_WIDTH: f32 = 0.45;
/// In arcade, how much harder the run grows each minute, compounded; and how many enemies appear a
/// second at its start.
const ARCADE_GROWTH: f32 = 1.07;
const ARCADE_RATE: f32 = 1.5;
/// The cards a run can deal again on a level-up, R on the cards.
const REROLLS: u8 = 1;
/// The seconds into the run the night falls at: the torch can be lit from then.
const TORCH_SECONDS: u32 = 26;

/// How far the torch's light fades into the dark beyond its radius, in tiles.
pub const TORCH_FADE: f32 = 6.0;

/// How an enemy of a horde is veiled in red, as Horde Hunter has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veil {
    /// One of the very last of its horde: the veil pulses.
    Pulsing,
    /// One of the others of the share veiled: the veil stays, faint.
    Steady,
}

/// The tick the night falls at, the torch to be lit.
fn torch_lights_up() -> u32 {
    TORCH_SECONDS * u32::from(TICKS_PER_SECOND)
}

/// The most ticks the torch flickers as it lights up, before it holds: a second and a half.
/// The run's level at which the hero's own action comes, told on a card of its own.
const ACTION_LEVEL: u32 = 3;
/// The ticks the infiltrator's goggles stay blank once struck by lightning: 4 s.
const GOGGLES_OUT_TICKS: u32 = 120;
const TORCH_FLICKER_MOST: u32 = 45;
/// The seconds a horde wiped out waits before its level comes, for the news to be read first;
/// and the chance, in percent, of a rare card on that level.
const HORDE_LEVEL_DELAY: u32 = 2;
const HORDE_RARE_PERCENT: u32 = 60;
/// The chance, in percent, that a level-up offers a legendary, at first and after one.
const LEGENDARY_PERCENT: u32 = 5;
/// A melee weapon's swing as it goes: the ticks before it can swing again; the slashes of the
/// swing done, still to come, and the ticks before the next.
#[derive(Debug, Clone, Copy, Default)]
struct Swinging {
    wait: u32,
    done: u32,
    left: u32,
    gap: u32,
}

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
#[allow(
    clippy::struct_excessive_bools,
    reason = "states of the run, each on its own"
)]
pub struct World {
    pub map: Map,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    /// The number the next enemy to appear gets.
    next_enemy_id: u32,
    pub bullets: Vec<Bullet>,
    /// Where bullets lately hit a tree or a rock, and when: a spark shows there a moment.
    pub sparks: Vec<(Vec2, u32)>,
    pub gems: Vec<Gem>,
    /// Where blood was spilt lately, oldest first.
    pub stains: Vec<Stain>,
    /// The enemies the player has felled, and the hordes wiped out to the last.
    pub kills: u32,
    hordes_wiped: u32,
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
    /// Ticks left before each drone shoots again.
    drone_waits: Vec<u32>,
    /// The Molotov's bottles in the air, its puddles of fire on the ground, and the ticks before
    /// its next throw.
    pub bottles: Vec<Bottle>,
    pub puddles: Vec<Puddle>,
    molotov_wait: u32,
    /// The C4's charges set down, its blasts while they show, and the ticks before its next
    /// charge.
    pub charges: Vec<Charge>,
    pub c4_blasts: Vec<c4::Blast>,
    pub c4_wait: u32,
    /// The ticks before the first-aid kit can be opened again.
    pub kit_wait: u32,
    /// The ticks the wall of fire still follows the player, and before it can be lit again.
    pub fire_wall_left: u32,
    pub fire_wall_wait: u32,
    /// The sniper rifle: the enemy it aims at and the ticks left before the shot, and the ticks
    /// before it can aim again.
    aim: Option<(u32, u32)>,
    sniper_wait: u32,
    /// The sniper's bullet in the air, and its impacts while they show.
    pub shot: Option<Shot>,
    pub impacts: Vec<Impact>,
    /// The grenades in the air, their blasts while they show, and the ticks before the next
    /// throw.
    pub grenades: Vec<Thrown>,
    pub grenade_blasts: Vec<c4::Blast>,
    grenade_wait: u32,
    /// The napalm's bombs falling, its blasts while they show, and the ticks before its next
    /// strike.
    pub bombs: Vec<Bomb>,
    pub napalm_blasts: Vec<c4::Blast>,
    napalm_wait: u32,
    /// The hammer's last blow, while it shows, and the ticks before the next.
    pub blow: Option<Blow>,
    hammer_wait: u32,
    /// The ticks the hammer stays raised over the player's head before it falls; 0 when it isn't.
    pub hammer_raised: u32,
    /// The shocks to come after the last blow, the waves of those come while they show, and the
    /// enemies the blow and its shocks have hit: each once.
    quakes: Vec<Quake>,
    pub quake_waves: Vec<Blow>,
    blow_hit: Vec<u32>,
    /// The stun baton's last thrust, while it shows, and the ticks before the next.
    pub thrust: Option<Thrust>,
    baton_wait: u32,
    /// The stun baton's strikes so far this run: now and then, one is a whirlwind.
    baton_strikes: u32,
    /// The combat knife's last stabs, while they show, and the ticks before the next.
    pub stabs: Vec<Stab>,
    knife_wait: u32,
    /// The katana's last chain of cuts, while it shows, and the ticks before the next.
    pub cuts: Option<Cuts>,
    katana_wait: u32,
    /// The defibrillator's last shock, while it shows, and the ticks before the next.
    pub shock: Option<Shock>,
    defibrillator_wait: u32,
    /// Whether the chainsaw grinds an enemy this tick, and the ticks before its next bite.
    pub sawing: bool,
    chainsaw_wait: u32,
    /// The HP the player has got back this run, for First aid.
    healed: u32,
    /// The Shadows felled this run, for Shadow hunter.
    shadows_down: u32,
    /// The ticks the player has stood low on HP this run, for Cold blood.
    low_hp_ticks: u32,
    /// The ticks the torch has been out this run, the night fallen, for Shadow; and since it was
    /// put out last, for the infiltrator.
    dark_ticks: u32,
    pub(super) dark_streak: u32,
    /// The ticks left the infiltrator's goggles stay blank, struck by lightning.
    goggles_out: u32,
    /// The fists' last punch, while it shows; the ticks before the next; the punches so far this
    /// run: every third is a heavier one.
    pub punch: Option<Punch>,
    pub(super) fists_wait: u32,
    punches: u32,
    /// The ram: the ticks before it can be played again; whether the player is charging, and the
    /// enemies the charge has hit, each once.
    pub ram_wait: u32,
    /// The hero's own action, just come, its card shown: the run stands still until the player
    /// has seen it.
    pub unlocked: Option<Action>,
    /// Braced before a charge: the way it will go, and the ticks left.
    ram_windup: Option<(Vec2, u32)>,
    ramming: bool,
    rammed: Vec<u32>,
    /// The ticks the player has stood surrounded this run, for Hold the line.
    surrounded_ticks: u32,
    /// The abominations felled by a melee blow this run: the crowbar's, the machete's, the
    /// hammer's.
    abominations_smashed: u32,
    /// Whether the flamethrower burns this tick; the way its fire goes (length 1); the ticks it
    /// has burnt, for its bursts of damage; and where the wall of fire left its last stretch.
    pub flaming: bool,
    flame_way: Vec2,
    flame_ticks: u32,
    trail_from: Vec2,
    /// The barricades standing, oldest first, and the ticks before the next can be set down.
    pub barricades: Vec<Barricade>,
    pub barricade_wait: u32,
    /// The last barricade that couldn't be set down, the ground taken, and when: it shows a
    /// moment where it would have stood.
    pub barricade_refused: Option<(Barricade, u32)>,
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
    /// The tick the player last dodged a blow at, if they have: the ninja's.
    pub dodged_at: Option<u32>,
    /// Whether the player wants their torch on; the tick it holds from, lit, flickering until
    /// then, as long as luck has it; and that luck, for the flicker.
    torch_switch: bool,
    /// Whether the player has lit their torch this run: until they do, F is shown to them.
    torch_lit_once: bool,
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
    /// The weapon the run started with, the hero's own: its levels dealt whatever the progress.
    start: Weapon,
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
        // The night falling, the torch waits for the player to light it; the dusk skipped, it
        // is lit already.
        world.torch_switch = skip_dusk;
        world.torch_lit_once = skip_dusk;
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
    #[allow(
        clippy::too_many_lines,
        reason = "every field of the world, one a line"
    )]
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
            sparks: Vec::new(),
            gems: Vec::new(),
            stains: Vec::new(),
            kills: 0,
            hordes_wiped: 0,
            score: 0,
            combo: 0,
            last_kill: 0,
            melee: HashMap::new(),
            swing: None,
            magazines: HashMap::new(),
            drone_angle: 0.0,
            drone_waits: Vec::new(),
            lightning: None,
            strikes_coming: Vec::new(),
            bottles: Vec::new(),
            puddles: Vec::new(),
            swellings: Vec::new(),
            blasts: Vec::new(),
            molotov_wait: 0,
            charges: Vec::new(),
            c4_blasts: Vec::new(),
            c4_wait: 0,
            kit_wait: 0,
            fire_wall_left: 0,
            fire_wall_wait: 0,
            aim: None,
            sniper_wait: 0,
            shot: None,
            impacts: Vec::new(),
            grenades: Vec::new(),
            grenade_blasts: Vec::new(),
            grenade_wait: 0,
            bombs: Vec::new(),
            napalm_blasts: Vec::new(),
            napalm_wait: 0,
            blow: None,
            hammer_wait: 0,
            hammer_raised: 0,
            quakes: Vec::new(),
            quake_waves: Vec::new(),
            blow_hit: Vec::new(),
            thrust: None,
            baton_wait: 0,
            baton_strikes: 0,
            stabs: Vec::new(),
            knife_wait: 0,
            cuts: None,
            katana_wait: 0,
            shock: None,
            defibrillator_wait: 0,
            sawing: false,
            chainsaw_wait: 0,
            healed: 0,
            shadows_down: 0,
            low_hp_ticks: 0,
            dark_ticks: 0,
            dark_streak: 0,
            goggles_out: 0,
            punch: None,
            fists_wait: 0,
            punches: 0,
            ram_wait: 0,
            unlocked: None,
            ram_windup: None,
            ramming: false,
            rammed: Vec::new(),
            surrounded_ticks: 0,
            abominations_smashed: 0,
            flaming: false,
            flame_way: Vec2::new(1.0, 0.0),
            flame_ticks: 0,
            trail_from: start,
            barricades: Vec::new(),
            barricade_wait: 0,
            barricade_refused: None,
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
            dodged_at: None,
            torch_switch: true,
            torch_lit_once: true,
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
            start: Weapon::Crowbar,
            progress: Progress::everything(),
            dealt: Progress::everything(),
            ticks: 0,
            rng,
            spawn_debt: 0.0,
            view: DEFAULT_VIEW,
        };
        // Lit as the night falls, flickering as luck has it: as a run that skips the dusk has it.
        world.light_up_torch();
        world
    }

    /// The run played by a hero: their HP, their speed, their perk, and their own weapon rather
    /// than the crowbar, dealt again to them alone.
    pub fn play_as(&mut self, hero: Skin) {
        self.start = hero.weapon();
        self.player.build = Build::playing(hero);
        self.player.hp = self.player.build.max_hp();
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

    /// How an enemy of a horde shows veiled in red, as Horde Hunter has it: pulsing for the very
    /// last ones of its horde, steady for the others of its share veiled; none otherwise.
    pub fn horde_veil(&self, enemy: &Enemy) -> Option<Veil> {
        let percent = self.player.build.horde_veil_percent();
        let tally = self.tallies.get(&enemy.horde?)?;
        if percent > 0 && tally.last(horde::LAST_PERCENT.min(percent)) {
            Some(Veil::Pulsing)
        } else if percent > 0 && tally.last(percent) {
            Some(Veil::Steady)
        } else {
            None
        }
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
        if self.is_over() || self.offer.is_some() || self.unlocked.is_some() {
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
            self.progress.dawns.insert(self.player.build.hero());
            return;
        }
        if self.weather() != weather {
            self.tell(News::Weather(self.weather()));
        }
        self.move_player(input);
        self.charge();
        self.spawn();
        self.hordes();
        self.move_enemies();
        self.batter_barricades();
        for enemy in &mut self.enemies {
            enemy.scorch();
        }
        self.burn_enemies();
        self.relocate_stragglers();
        self.storm();
        self.swing_melee();
        self.swing_hammer();
        self.thrust_baton();
        self.stab_knife();
        self.throw_punches();
        self.cut_katana();
        self.shock_defibrillator();
        self.grind_chainsaw();
        self.set_charges();
        self.wait_kit();
        self.follow_fire_wall();
        self.throw_grenades();
        self.drop_napalm();
        self.throw_molotovs();
        self.throw_flames();
        self.fire_guns();
        self.snipe();
        self.move_bullets();
        self.turn_drones();
        self.burst_abominations();
        self.fell_enemies();
        self.gather_gems();
        self.give_horde_levels();
        self.unlock_action();
        self.illuminate();
        self.regenerate();
        self.heal_player();
        self.hurt_player();
        self.reach_goals();
        if self.pending_levels > 0 && !self.is_over() {
            self.offer = Some(self.deal());
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

    /// Illumination gives XP of itself, a point a second.
    fn illuminate(&mut self) {
        if self.player.build.has(Legendary::Illumination)
            && self.ticks.is_multiple_of(u32::from(TICKS_PER_SECOND))
        {
            self.give_xp(100);
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
}
