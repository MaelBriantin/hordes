//! What the terminal shows: the player's HP and XP bars and the time on top, the map around the
//! player in a frame, the keys below it; over the map, the title screen, the pause menu, the
//! level-up's cards or the game over.

mod achievement;
mod announce;
mod collection;
mod debug;
mod equipment;
mod font;
mod horde;
mod icons;
mod level_up;
mod menu;
mod night;
mod pixels;
mod skins;
mod sprites;
mod text;

use std::time::Duration;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Widget};

use crate::game::{Achievement, Leaderboard, Progress, Vec2, World, xp_to_next};
use crate::game::{Mode, Weather, WeatherNews};
pub use achievement::TOAST_FRAMES;
pub use debug::DebugAction;
pub use menu::{OptionItem, PauseItem};
pub use night::{BloodLook, RainDisplay};
pub use pixels::Arrow;
pub use skins::{SKINS, unlocked as skin_unlocked};
pub use sprites::{Outfit, Skin, Sprites, colours_of};

/// Where the player is, outside the run itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The game waits for the player to start.
    Title,
    Playing,
    /// The run stands still, the pause menu open on its item `selected`.
    Paused {
        selected: usize,
    },
    /// The run is over with one of the best scores: three letters to type in, arcade style, the
    /// one at `cursor` being changed.
    NameEntry {
        letters: [u8; 3],
        cursor: usize,
    },
    /// The run stands still, the debug console open on its action `selected`: debug builds only.
    Debug {
        selected: usize,
    },
    /// The run stands still, every key told.
    Keys,
    /// The options menu open on its item `selected`: over the run, opened from the pause menu, or
    /// over the title screen.
    Options {
        selected: usize,
        from_pause: bool,
    },
}
/// How bright a bar's empty part is, against its full part.
const BAR_EMPTY: f32 = 0.22;
/// Every weather, in their order, for the row of their icons.
const WEATHERS: [Weather; 6] = [
    Weather::Dusk,
    Weather::Clear,
    Weather::Drizzle,
    Weather::Rain,
    Weather::Storm,
    Weather::Tempest,
];
/// The icons of the row: those gone, put out, in grey; those to come, and the next one by the
/// end of the weather now, at a share of their colour.
const WEATHER_GONE: Color = Color::Rgb(50, 50, 58);
const WEATHER_AHEAD: f32 = 0.18;
const WEATHER_NEXT: f32 = 0.8;
/// The dawn's icon, at the end of the row: the rising sun's gold.
const DAWN_COLOR: Color = Color::Rgb(255, 210, 90);
/// Below this share of their HP, the player's HP bar blinks, and their heart beats.
const LOW_HP_PERCENT: u32 = 30;
/// The frames the low HP bar stays lit, then dimmed, as it blinks.
const LOW_HP_BLINK: u32 = 8;
/// The width of the HP bar's inside, in cells.
const BAR_CELLS: usize = 12;
/// The HP bar's red, the herbs': and dimmed, as it blinks.
const HP_COLOR: Color = Color::Rgb(220, 45, 65);
const HP_DIMMED: Color = Color::Rgb(90, 20, 28);
const XP_COLOR: Color = Color::Rgb(115, 138, 178);
const SCORE_COLOR: Color = Color::Rgb(240, 200, 60);
/// The text on the score's gold badge.
const BADGE_TEXT: Color = Color::Rgb(30, 22, 8);
/// The torch's bulb, on and out.
const TORCH_COLOR: Color = Color::Rgb(80, 215, 110);
const TORCH_OUT_COLOR: Color = Color::Rgb(120, 140, 170);
/// How long the edges of the map flush red after a blow, in ticks; how deep, in cells; how red
/// at most, and in which red.
const HURT_TICKS: u32 = 15;
const HURT_EDGE: u16 = 4;
const HURT_STRENGTH: f32 = 0.6;
const HURT_RED: (u8, u8, u8) = (200, 20, 30);
/// Low on HP (under `LOW_HP_PERCENT`), the player's heart beats on the screen's edges, red: a beat
/// every so many ticks, from at that share to near 0 HP; how strong a beat is, and how much of
/// the time between two it takes to fade.
const HEARTBEAT_TICKS: (u32, u32) = (36, 15);
const HEARTBEAT_STRENGTH: f32 = 0.35;
const HEARTBEAT_FADE: f32 = 0.4;
/// The game's own background and text, whatever the terminal's theme: a light one would wash
/// the night out.
const SCREEN: Style = Style::new()
    .fg(Color::Rgb(225, 225, 230))
    .bg(Color::Rgb(12, 12, 18));
/// A key drawn as a cap: dark on grey.
const KEY_CAP: Style = Style::new()
    .fg(Color::Rgb(12, 12, 18))
    .bg(Color::Rgb(140, 145, 160));
/// Greys for what is said aside, and for frames.
pub const GREY: Color = Color::Rgb(165, 165, 172);
pub const DARK_GREY: Color = Color::Rgb(105, 105, 115);

/// Empties an area in the game's own colours: under a box or a banner, or the whole screen.
pub struct Blank;

impl Widget for Blank {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Clear.render(area, buffer);
        buffer.set_style(area, SCREEN);
    }
}

/// The screen's parts: the line on top, the map in its frame, the keys below.
struct Areas {
    top: Rect,
    field: Rect,
    help: Rect,
}

fn areas(screen: Rect) -> Areas {
    // A row left empty at the bottom: the keys or the XP bar don't sit on the screen's edge.
    let [top, field, help, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(screen);
    Areas { top, field, help }
}

/// How many tiles a row of the collection holds on a screen of that size.
pub fn collection_row(screen: Rect) -> usize {
    collection::in_a_row(Block::bordered().inner(areas(screen).field).width)
}

/// The collection's tile under or over the one at `at`, rows of `per_row` tiles: the one standing
/// there on the screen.
pub fn collection_beside(at: usize, per_row: usize, down: bool) -> usize {
    collection::beside(at, per_row, down)
}

/// Whether the collection shows on the title screen, on a screen of that size.
pub fn collection_shows(screen: Rect) -> bool {
    menu::collection_shows(Block::bordered().inner(areas(screen).field))
}

/// Half of the map a screen of that size shows each way, in tiles.
pub fn view(screen: Rect) -> Vec2 {
    pixels::view(Block::bordered().inner(areas(screen).field))
}

/// What the screen needs beyond the run: the card selected on a level-up and how long the
/// cards have been shown, and the frames drawn so far, for what moves on its own.
#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "settings and states of the screen, each on its own"
)]
pub struct ViewState {
    pub selected: usize,
    pub shown: Duration,
    pub frame: u32,
    /// How blood shows, and how the rain shows.
    pub blood: BloodLook,
    pub rain: RainDisplay,
    /// The arrow over the player, always, in its colour; or a moment, once pinged.
    pub arrow: Arrow,
    pub pinged: bool,
    /// Whether the player's HP shows in a small bar over their head.
    pub mini_bars: bool,
    /// Whether the ground is coloured: grass speckled with earth, rather than the screen's dark.
    pub ground: bool,
    /// How the hero looks; in which colours each hero shows among the heroes, by their place,
    /// unlocked or not; and the colours the hero worn wears in a run, theirs unlocked.
    pub skin: Skin,
    pub colours: [u8; SKINS.len()],
    pub worn_colours: u8,
    /// The mode the next run is played in; whether the arcade can be chosen, its achievement
    /// reached.
    pub mode: Mode,
    pub arcade: bool,
    /// Whether the gear shows over the top left of the map; as a list rather than icons.
    pub gear: bool,
    pub gear_list: bool,
    /// How a new weather is told.
    pub weather_news: WeatherNews,
    /// The language the game speaks, by its own name: `Français`.
    pub language: &'static str,
    /// The leaderboard's place of the run just over, if it made it.
    pub highlight: Option<usize>,
    /// The achievement being told, and the frames since it started to show.
    pub toast: Option<(Achievement, u32)>,
    /// The collection's tile the arrows are on, on the title screen; none without a collection
    /// showing.
    pub picked: Option<usize>,
    /// The skin's tile the arrows are on, on the title screen, and the frames since it was
    /// picked; none while the skins don't show.
    pub skins: Option<(usize, u32)>,
    /// On the title screen, what was under the title before (the mode, and whether the
    /// collection or the skins showed) and the frames since it changed: the title slides from
    /// where it stood.
    pub slide_from: (Mode, bool),
    pub panel_since: u32,
    /// Whether the options ask to confirm erasing the progress.
    pub confirm_reset: bool,
}

impl ViewState {
    /// The hero worn, in their colours.
    pub fn outfit(&self) -> Outfit {
        Outfit {
            skin: self.skin,
            colours: self.worn_colours,
        }
    }
}

pub fn render(
    frame: &mut Frame,
    world: &World,
    sprites: &Sprites,
    leaderboard: &Leaderboard,
    progress: &Progress,
    screen: Screen,
    view: ViewState,
) {
    frame.render_widget(Blank, frame.area());
    let areas = areas(frame.area());
    let block = Block::bordered();
    let map = block.inner(areas.field);
    frame.render_widget(block, areas.field);
    // The run over, the death screen, but for the pause menu over the run.
    let dead = world.is_over()
        && !matches!(
            screen,
            Screen::Paused { .. } | Screen::Debug { .. } | Screen::Keys | Screen::Options { .. }
        );
    let on_title = matches!(
        screen,
        Screen::Title
            | Screen::Options {
                from_pause: false,
                ..
            }
    );
    if on_title {
        menu::render_title(frame, map, sprites, view, leaderboard, progress);
    } else if dead {
        let name = match screen {
            Screen::NameEntry { letters, cursor } => Some((letters, cursor)),
            _ => None,
        };
        menu::render_death(frame, map, sprites, view, world, leaderboard, name);
    } else {
        render_status(frame, areas.top, world, leaderboard, view.frame);
        // Under a word in big letters, the player isn't picked out: the arrow and the bar
        // would cross it.
        let view = if announce::showing(world) {
            ViewState {
                arrow: Arrow::Off,
                pinged: false,
                mini_bars: false,
                ..view
            }
        } else {
            view
        };
        pixels::render_map(frame.buffer_mut(), map, world, sprites, view);
        if !announce::showing(world) {
            render_torch_hint(frame, map, world, view.frame);
        }
        render_hurt(frame.buffer_mut(), map, world);
        render_heartbeat(frame.buffer_mut(), map, world);
    }
    // The keys below, but for the run going on: its XP bar, all along the screen.
    let help = help_line(screen, world, view);
    match help {
        Some(help) => frame.render_widget(Line::from(help).fg(DARK_GREY), areas.help),
        None => frame.render_widget(experience(world, areas.help.width), areas.help),
    }
    match screen {
        Screen::Title | Screen::NameEntry { .. } => return,
        // A menu open: what lies under it dimmed, the menu standing out.
        Screen::Paused { selected } => {
            announce::dim(frame.buffer_mut(), map);
            menu::render_pause(frame, map, selected, (world, sprites, view));
            return;
        }
        Screen::Options {
            selected,
            from_pause,
        } => {
            announce::dim(frame.buffer_mut(), map);
            menu::render_options(frame, map, selected, from_pause, view);
            return;
        }
        Screen::Debug { selected } => {
            announce::dim(frame.buffer_mut(), map);
            debug::render(frame, map, selected, world.unhurt);
            return;
        }
        Screen::Keys => {
            announce::dim(frame.buffer_mut(), map);
            menu::render_keys(frame, map);
            return;
        }
        Screen::Playing => {}
    }
    if !dead {
        horde::render(frame, map, world, view.frame);
        announce::render(frame, map, world, sprites);
        // The hero's action just come shows alone; the cards of the level-up after it.
        if world.unlocked.is_some() {
            level_up::render_unlocked(frame, map, world, sprites);
        } else {
            level_up::render(frame, map, world, sprites, view.selected, view.shown);
        }
    }
    if let Some((achievement, since)) = view.toast {
        achievement::render(frame.buffer_mut(), map, sprites, achievement, since);
    }
}

/// The keys to tell under the map, for `screen`; none while a run goes on, its XP bar there.
fn help_line(screen: Screen, world: &World, view: ViewState) -> Option<String> {
    match screen {
        Screen::Title if view.picked.is_some() => Some(fl!("help-title-browsing")),
        Screen::Title if !view.arcade => Some(fl!("help-title-standard")),
        Screen::Title if view.mode == Mode::Standard => Some(fl!("help-title")),
        Screen::Title => Some(fl!("help-title-arcade")),
        Screen::Paused { .. } => Some(fl!("help-paused")),
        Screen::Options { .. } => Some(fl!("help-options")),
        Screen::NameEntry { .. } => Some(fl!("help-name")),
        Screen::Playing if world.is_over() => Some(fl!("help-over")),
        // The hero's action just come: its own box tells how to go on.
        Screen::Playing if world.unlocked.is_some() => None,
        Screen::Playing if world.offer.is_some() => Some(fl!("help-level-up")),
        Screen::Playing | Screen::Debug { .. } | Screen::Keys => None,
    }
}

/// The rows under the middle of the map, where the player stands, that F shows on.
const TORCH_HINT_ROWS: u16 = 3;

/// The night fallen, the torch never lit yet this run: F under the player, blinking as the
/// title's prompt, with what it does.
fn render_torch_hint(frame: &mut Frame, map: Rect, world: &World, frames: u32) {
    if !world.torch_hint() || frames % 30 >= 20 {
        return;
    }
    let y = map.y + map.height / 2 + TORCH_HINT_ROWS;
    if y >= map.bottom() {
        return;
    }
    let line = Line::from(vec![
        Span::styled(format!(" {} ", fl!("key-f")), KEY_CAP),
        Span::styled(
            format!(" {}", fl!("torch-hint").to_uppercase()),
            Style::new()
                .fg(Color::Rgb(225, 225, 230))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(line.centered(), Rect::new(map.x, y, map.width, 1));
}

/// The edges of the map flushing red a moment after the player is hurt, fading: the blow is
/// felt.
fn render_hurt(buffer: &mut Buffer, map: Rect, world: &World) {
    let Some(at) = world.hurt_at else {
        return;
    };
    let since = world.ticks().saturating_sub(at);
    if since >= HURT_TICKS {
        return;
    }
    let share = |part: u32, whole: u32| {
        f32::from(u16::try_from(part).unwrap_or(0)) / f32::from(u16::try_from(whole).unwrap_or(1))
    };
    redden_edges(
        buffer,
        map,
        HURT_STRENGTH * (1.0 - share(since, HURT_TICKS)),
    );
}

/// How low the player's HP is, under `LOW_HP_PERCENT` of it: from 0, just under, to 1, near
/// none; none above, or fallen.
pub(super) fn low_hp(world: &World) -> Option<f32> {
    let (hp, most) = (world.player.hp, world.player.max_hp().max(1));
    if hp == 0 || hp * 100 >= most * LOW_HP_PERCENT {
        return None;
    }
    let share = |part: u32, whole: u32| {
        f32::from(u16::try_from(part).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(whole).unwrap_or(u16::MAX).max(1))
    };
    Some(1.0 - share(hp * 100, most * LOW_HP_PERCENT))
}

/// Low on HP, the player's heart beats on the edges of the map, red, faster the lower.
fn render_heartbeat(buffer: &mut Buffer, map: Rect, world: &World) {
    let Some(low) = low_hp(world) else {
        return;
    };
    let (slowest, fastest) = (
        f32::from(u16::try_from(HEARTBEAT_TICKS.0).unwrap_or(1)),
        f32::from(u16::try_from(HEARTBEAT_TICKS.1).unwrap_or(1)),
    );
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few dozen ticks"
    )]
    let every = (slowest + (fastest - slowest) * low).round().max(1.0) as u32;
    let since = f32::from(u16::try_from(world.ticks() % every).unwrap_or(0));
    let fade = f32::from(u16::try_from(every).unwrap_or(1)) * HEARTBEAT_FADE;
    let beat = (1.0 - since / fade).max(0.0);
    if beat > 0.0 {
        redden_edges(buffer, map, HEARTBEAT_STRENGTH * beat);
    }
}

/// The edges of the map reddened, `strength` at the very edge, fading inwards.
fn redden_edges(buffer: &mut Buffer, map: Rect, strength: f32) {
    let share = |part: u32, whole: u32| {
        f32::from(u16::try_from(part).unwrap_or(0)) / f32::from(u16::try_from(whole).unwrap_or(1))
    };
    for y in map.top()..map.bottom() {
        for x in map.left()..map.right() {
            // In cells across, and in half cells down: a cell is twice as tall as wide.
            let edge = (x - map.left())
                .min(map.right() - 1 - x)
                .min((y - map.top()) * 2)
                .min((map.bottom() - 1 - y) * 2);
            if edge >= HURT_EDGE {
                continue;
            }
            let by = strength * (1.0 - share(u32::from(edge), u32::from(HURT_EDGE)));
            let cell = &mut buffer[(x, y)];
            cell.fg = redden(cell.fg, HURT_RED, by);
            cell.bg = redden(cell.bg, HURT_RED, by);
        }
    }
}

/// A colour drawn towards a `red`, as much as `by`, from 0 (none) to 1 (all red).
fn redden(color: Color, red: (u8, u8, u8), by: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "between two channels, 0 to 255"
            )]
            let mix = |from: u8, to: u8| {
                let from = f32::from(from);
                (from + (f32::from(to) - from) * by).round() as u8
            };
            Color::Rgb(mix(r, red.0), mix(g, red.1), mix(b, red.2))
        }
        other => other,
    }
}

/// The status line, in two parts: on the left, what keeps the player alive (their HP and the
/// weather); on the right, how the run goes (the time and the score).
fn render_status(
    frame: &mut Frame,
    area: Rect,
    world: &World,
    leaderboard: &Leaderboard,
    frames: u32,
) {
    frame.render_widget(survival(world, frames), area);
    frame.render_widget(run(world, leaderboard, frames).right_aligned(), area);
}

/// The player's HP and the weather, as gauges.
fn survival(world: &World, frames: u32) -> Line<'static> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let player = &world.player;
    let (hp, max_hp) = (player.hp, player.max_hp());
    let low = hp * 100 < max_hp * LOW_HP_PERCENT;
    let hp_color = if low && (frames / LOW_HP_BLINK).is_multiple_of(2) {
        HP_DIMMED
    } else {
        HP_COLOR
    };
    let weather = world.weather();
    let mut spans = vec![Span::styled(format!(" {} ", icons::HP), bold.fg(HP_COLOR))];
    // The HP in the middle of their bar.
    let mut hp_bar = embossed(&bar(hp, max_hp), hp_color);
    labelled(&mut hp_bar, &hp.to_string(), hp_color);
    spans.extend(hp_bar);
    spans.push(Span::raw("   "));
    // The torch, on or out, from the start (out at dusk, the torch lighting up of itself as the
    // night falls): green while it is on, grey-blue while it is out.
    let (icon, colour) = if world.torch_on().unwrap_or(false) {
        (icons::TORCH_ON, TORCH_COLOR)
    } else {
        (icons::TORCH_OFF, TORCH_OUT_COLOR)
    };
    spans.push(Span::styled(icon.to_string(), bold.fg(colour)));
    spans.push(Span::raw("   "));
    spans.extend(weather_track(world));
    spans.push(Span::styled(
        format!(" {}", weather_name(weather)),
        weather_style(weather),
    ));
    Line::from(spans)
}

/// The player's level and their XP, in a bar as wide as `width` leaves.
fn experience(world: &World, width: u16) -> Line<'static> {
    let player = &world.player;
    let mut spans = vec![Span::raw(" ")];
    let taken: usize = spans.iter().map(Span::width).sum();
    // The bar's two ends take a cell each, and a cell is left after it.
    let cells = usize::from(width).saturating_sub(taken + 3);
    let mut bar = embossed(
        &bar_of(player.xp, xp_to_next(player.level), cells),
        XP_COLOR,
    );
    // The level in the middle of the bar.
    labelled(&mut bar, &player.level.to_string(), XP_COLOR);
    spans.extend(bar);
    Line::from(spans)
}

/// The time survived and, in arcade, the score, with its multiplier: the score gleams while
/// ahead of the best run kept, as a new high score does.
fn run(world: &World, leaderboard: &Leaderboard, frames: u32) -> Line<'static> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let mut spans = vec![
        Span::styled(format!("{} ", icons::TIME), bold),
        Span::styled(clock(world.seconds()), bold),
    ];
    // A standard run keeps no score: the time alone.
    if world.mode == Mode::Standard {
        spans.push(Span::raw(" "));
        return Line::from(spans);
    }
    // The score as a badge, dark on gold, its thousands apart: seen at a glance. Ahead of the best
    // run kept, it gleams instead, as a new high score does.
    let badge = bold.fg(BADGE_TEXT).bg(SCORE_COLOR);
    spans.push(Span::raw("   "));
    spans.push(Span::styled(format!(" {} ", icons::SCORE), badge));
    let score = thousands(world.score);
    let best = leaderboard.entries().first().map(|entry| entry.score);
    if best.is_some_and(|best| world.score > best) {
        spans.extend(text::shimmer(&score, frames));
    } else {
        spans.push(Span::styled(score, badge));
    }
    spans.extend([
        Span::styled(
            format!(" {} ", multiplier(world.multiplier_tenths())),
            badge,
        ),
        Span::raw(" "),
    ]);
    Line::from(spans)
}

/// A share of a whole as a bar: `▕████░░░░░░░░▏`. Rounded down, so it only shows full when it
/// is.
fn bar(part: u32, whole: u32) -> String {
    bar_of(part, whole, BAR_CELLS)
}

/// The same bar, `cells` wide inside its ends.
fn bar_of(part: u32, whole: u32, cells: usize) -> String {
    let cells_u64 = u64::try_from(cells).unwrap_or(0);
    let filled = (u64::from(part) * cells_u64 / u64::from(whole.max(1))).min(cells_u64);
    let filled = usize::try_from(filled).unwrap_or(0);
    format!("▕{}{}▏", "█".repeat(filled), "░".repeat(cells - filled))
}

pub fn weather_name(weather: Weather) -> String {
    match weather {
        Weather::Dusk => fl!("weather-dusk"),
        Weather::Clear => fl!("weather-clear"),
        Weather::Drizzle => fl!("weather-drizzle"),
        Weather::Rain => fl!("weather-rain"),
        Weather::Storm => fl!("weather-storm"),
        Weather::Tempest => fl!("weather-tempest"),
    }
}

/// The weather's colour, bold.
fn weather_style(weather: Weather) -> Style {
    Style::new()
        .fg(weather_colour(weather))
        .add_modifier(Modifier::BOLD)
}

/// The weather's colour: orange at dusk, grey for a clear night, darkening through blues to red.
pub fn weather_colour(weather: Weather) -> Color {
    match weather {
        Weather::Dusk => Color::Rgb(240, 150, 70),
        Weather::Clear => Color::Rgb(160, 160, 170),
        Weather::Drizzle => Color::Rgb(130, 170, 220),
        Weather::Rain => Color::Rgb(80, 120, 230),
        Weather::Storm => Color::Rgb(170, 90, 230),
        Weather::Tempest => Color::Rgb(230, 50, 50),
    }
}

/// Every weather's icon in a row, from the dusk to the tempest: the weather now in its colour,
/// those gone dimmed, those to come barely there, but the next one, coming out of the dark as
/// the run goes `progress` through the weather now.
fn weather_track(world: &World) -> Vec<Span<'static>> {
    let (now, next, progress) = (
        world.weather(),
        world.next_weather(),
        world.weather_progress(),
    );
    let mut spans = Vec::new();
    for weather in WEATHERS {
        let colour = weather_colour(weather);
        let shown = if weather == now {
            colour
        } else if Some(weather) == next {
            faded(
                colour,
                WEATHER_AHEAD + (WEATHER_NEXT - WEATHER_AHEAD) * progress,
            )
        } else if weather < now && world.mode == Mode::Standard {
            // Gone for good in a standard run; in arcade, every weather comes back.
            WEATHER_GONE
        } else {
            faded(colour, WEATHER_AHEAD)
        };
        let style = Style::new().fg(shown);
        let style = if weather == now {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        };
        if !spans.is_empty() {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(icons::weather(weather), style));
    }
    // The dawn last, coming out of the dark through the tempest: none in arcade.
    if world.mode == Mode::Arcade {
        return spans;
    }
    let dawn = if now == Weather::Tempest {
        WEATHER_AHEAD + (WEATHER_NEXT - WEATHER_AHEAD) * progress
    } else {
        WEATHER_AHEAD
    };
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        icons::DAWN,
        Style::new().fg(faded(DAWN_COLOR, dawn)),
    ));
    spans
}

/// A colour at a share of its brightness, towards the screen's dark ground.
fn faded(color: Color, share: f32) -> Color {
    match color {
        Color::Rgb(red, green, blue) => Color::Rgb(
            night::scale_channel(red, share),
            night::scale_channel(green, share),
            night::scale_channel(blue, share),
        ),
        other => other,
    }
}

/// A bar in its colour, in relief: the top half of each full cell lighter, as lit from above;
/// its empty part flat, in a dark shade of the colour, as tall as the full one.
fn embossed(bar: &str, color: Color) -> Vec<Span<'static>> {
    let empty = faded(color, BAR_EMPTY);
    bar.chars()
        .map(|cell| match cell {
            // The lit top as the cell's ground, the bar's colour as a lower half block over it:
            // a terminal that draws its half blocks a little short shows no line of the bar's
            // colour above the light.
            '█' => Span::styled("▄", Style::new().fg(color).bg(lighter(color))),
            '░' => Span::styled(" ", Style::new().bg(empty)),
            _ => Span::styled(cell.to_string(), color),
        })
        .collect()
}

/// A number in the middle of a bar, in white on whatever part of the bar is there, a space each
/// side.
fn labelled(bar: &mut [Span<'static>], text: &str, color: Color) {
    let label = format!(" {text} ");
    let start = bar.len().saturating_sub(label.len()) / 2;
    for (offset, digit) in label.chars().enumerate() {
        if let Some(cell) = bar.get_mut(start + offset) {
            let ground = cell.style.bg.unwrap_or(color);
            *cell = Span::styled(
                digit.to_string(),
                Style::new()
                    .fg(Color::Rgb(245, 245, 250))
                    .bg(ground)
                    .add_modifier(Modifier::BOLD),
            );
        }
    }
}

/// A colour a third of the way to white: the lit top of a bar.
fn lighter(color: Color) -> Color {
    match color {
        Color::Rgb(red, green, blue) => {
            let lit = |value: u8| value + (u8::MAX - value) / 3;
            Color::Rgb(lit(red), lit(green), lit(blue))
        }
        other => other,
    }
}

/// A number with its thousands apart: `12 345`.
fn thousands(number: u64) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(' ');
        }
        grouped.push(digit);
    }
    grouped
}

/// A multiplier given in tenths: `×2.4`.
fn multiplier(tenths: u64) -> String {
    let (whole, tenth) = (tenths / 10, tenths % 10);
    fl!("multiplier", whole = whole, tenth = tenth)
}

/// Seconds as minutes and seconds: `03:42`.
pub fn clock(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heart_beats_under_30_percent_of_hp_harder_the_lower() {
        let mut world = World::new(1);
        assert_eq!(low_hp(&world), None);
        world.player.hp = 30;
        assert_eq!(low_hp(&world), None, "at 30 %, not yet");
        world.player.hp = 29;
        let just_under = low_hp(&world).unwrap();
        world.player.hp = 3;
        assert!(low_hp(&world).unwrap() > just_under);
        world.player.hp = 0;
        assert_eq!(low_hp(&world), None, "fallen");
    }

    #[test]
    fn a_clock_shows_minutes_and_seconds() {
        assert_eq!(clock(0), "00:00");
        assert_eq!(clock(222), "03:42");
        assert_eq!(clock(600), "10:00");
    }

    #[test]
    fn a_number_shows_its_thousands_apart() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(12_345), "12 345");
        assert_eq!(thousands(1_234_567), "1 234 567");
    }

    #[test]
    fn a_bar_fills_with_its_share() {
        assert_eq!(bar(0, 10), "▕░░░░░░░░░░░░▏");
        assert_eq!(bar(5, 10), "▕██████░░░░░░▏");
        assert_eq!(bar(10, 10), "▕████████████▏");
        assert_eq!(bar(99, 100), "▕███████████░▏");
    }
}
