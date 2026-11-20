//! The title screen and the pause menu, over the map.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};

use super::clock;
use super::font;
use super::night;
use super::pixels;
use super::sprites::{SpriteId, Sprites};
use super::text::{centred, centred_as_is, centred_spans, shimmer};
use super::{Arrow, Blank, BloodLook, DARK_GREY, GREY, RainDisplay, Skin, ViewState};
use crate::game::{Entry, Leaderboard, Mode, WeatherNews, World};

/// What is selected: an item of the pause menu, the letter being typed in.
const SELECTED: Color = Color::Rgb(230, 55, 55);
/// What is worth points: a score, a run on the leaderboard.
const GOLD: Color = Color::Rgb(240, 200, 60);
/// The width of the leaderboard's box on the title screen.
const BOARD_WIDTH: u16 = 30;
/// The box of the keys: its longest line, and room around it.
const KEYS_WIDTH: u16 = 52;
/// The death screen's big word: blood red on top, darker below, as the title.
const DEATH_COLORS: (Color, Color) = (Color::Rgb(210, 40, 40), Color::Rgb(140, 20, 25));
/// The dawn's word, for a run held out to the end: the rising sun's gold.
pub const DAWN_COLORS: (Color, Color) = (Color::Rgb(255, 210, 90), Color::Rgb(230, 140, 50));

/// What the pause menu offers, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseItem {
    Resume,
    /// The settings.
    Options,
    NewRun,
    /// Back to the title screen, the run left.
    Title,
    Quit,
}

impl PauseItem {
    pub const ALL: [PauseItem; 5] = [
        PauseItem::Resume,
        PauseItem::Options,
        PauseItem::NewRun,
        PauseItem::Title,
        PauseItem::Quit,
    ];

    fn label(self) -> String {
        match self {
            PauseItem::Resume => fl!("pause-resume"),
            PauseItem::Options => fl!("options-title"),
            PauseItem::NewRun => fl!("pause-new-run"),
            PauseItem::Title => fl!("pause-title-screen"),
            PauseItem::Quit => fl!("pause-quit"),
        }
    }
}

/// What the options menu offers, in its order: the settings, kept for the next runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionItem {
    /// Blood shown or not.
    Blood,
    /// How the rain shows.
    Rain,
    /// The arrow over the player, in its colour.
    Arrow,
    /// How the hero looks.
    Skin,
    /// How a new weather is told: in big letters, the run standing still or not; or on a banner.
    Announcements,
    /// Whether runs open on the dusk or on the night.
    Dusk,
    /// Whether the gear shows over the top left of the map.
    Gear,
    /// Whether the player's HP shows in a small bar over their head.
    MiniBars,
    /// Whether the ground is coloured: an experiment.
    Ground,
    /// The language the game speaks.
    Language,
    /// Back where the menu was opened from.
    Back,
    /// Quit the game: from the title screen's options.
    Quit,
}

impl OptionItem {
    const ALL: [OptionItem; 12] = [
        OptionItem::Blood,
        OptionItem::Rain,
        OptionItem::Arrow,
        OptionItem::Skin,
        OptionItem::Announcements,
        OptionItem::Dusk,
        OptionItem::Gear,
        OptionItem::MiniBars,
        OptionItem::Ground,
        OptionItem::Language,
        OptionItem::Back,
        OptionItem::Quit,
    ];

    /// The items of the menu, opened from the pause menu (which has its own way to quit) or
    /// from the title screen.
    pub fn items(from_pause: bool) -> &'static [OptionItem] {
        if from_pause {
            &Self::ALL[..Self::ALL.len() - 1]
        } else {
            &Self::ALL
        }
    }

    fn label(self, view: ViewState) -> String {
        match self {
            OptionItem::Blood => fl!("pause-blood", value = blood_name(view.blood)),
            OptionItem::Rain => fl!("pause-rain", value = rain_name(view.rain)),
            OptionItem::Arrow => fl!("option-arrow", value = arrow_name(view.arrow)),
            OptionItem::Skin => fl!(
                "option-skin",
                value = match view.skin {
                    Skin::Survivor => fl!("skin-survivor"),
                    Skin::Classic => fl!("skin-classic"),
                }
            ),
            OptionItem::Announcements => fl!(
                "pause-announcements",
                value = match view.weather_news {
                    WeatherNews::Paused => fl!("announcements-paused"),
                    WeatherNews::Big => fl!("announcements-big"),
                    WeatherNews::Banner => fl!("announcements-banner"),
                }
            ),
            OptionItem::Dusk => fl!(
                "option-dusk",
                value = if view.skip_dusk {
                    fl!("dusk-skipped")
                } else {
                    fl!("dusk-played")
                }
            ),
            OptionItem::Gear => fl!(
                "option-gear",
                value = if view.gear {
                    fl!("gear-shown")
                } else {
                    fl!("gear-hidden")
                }
            ),
            OptionItem::MiniBars => fl!(
                "option-mini-bars",
                value = if view.mini_bars {
                    fl!("mini-bars-shown")
                } else {
                    fl!("mini-bars-hidden")
                }
            ),
            OptionItem::Ground => fl!(
                "option-ground",
                value = if view.ground {
                    fl!("ground-coloured")
                } else {
                    fl!("ground-plain")
                }
            ),
            OptionItem::Language => fl!("pause-language", value = view.language),
            OptionItem::Back => fl!("option-back"),
            OptionItem::Quit => fl!("pause-quit"),
        }
    }
}

fn blood_name(blood: BloodLook) -> String {
    match blood {
        BloodLook::Red => fl!("blood-red"),
        BloodLook::Ketchup => fl!("blood-ketchup"),
        BloodLook::Mayo => fl!("blood-mayo"),
        BloodLook::Pesto => fl!("blood-pesto"),
        BloodLook::Off => fl!("setting-off"),
    }
}

fn arrow_name(arrow: Arrow) -> String {
    match arrow {
        Arrow::Off => fl!("arrow-off"),
        Arrow::White => fl!("arrow-white"),
        Arrow::Yellow => fl!("arrow-yellow"),
        Arrow::Red => fl!("arrow-red"),
    }
}

fn rain_name(rain: RainDisplay) -> String {
    match rain {
        RainDisplay::Full => fl!("rain-full"),
        RainDisplay::Reduced => fl!("rain-reduced"),
        RainDisplay::Off => fl!("rain-off"),
    }
}

/// A box of `width` × `height` in the middle of `map`, cleared, framed, `footer` dim on its
/// bottom border, on the right.
fn boxed(frame: &mut Frame, map: Rect, width: u16, height: u16, title: &str, footer: &str) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(map);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    frame.render_widget(Blank, area);
    let block = Block::bordered().padding(Padding::new(2, 2, 1, 1));
    let block = if title.is_empty() {
        block
    } else {
        block.title(Line::from(format!(" {title} ")).centered())
    };
    let block = block.title_bottom(
        Line::styled(format!(" {footer} "), Style::new().fg(DARK_GREY)).right_aligned(),
    );
    let inside = block.inner(area);
    frame.render_widget(block, area);
    inside
}

/// The title screen, `frames` frames in: the game's name, big, over a stormy night; the game
/// waits for the player.
pub fn render_title(
    frame: &mut Frame,
    area: Rect,
    sprites: &Sprites,
    view: ViewState,
    leaderboard: &Leaderboard,
) {
    let frames = view.frame;
    night::render_storm(frame.buffer_mut(), area, sprites, frames);
    // Twice as big where it fits: 70 columns, 7 rows.
    let scale = if area.width >= 74 && area.height >= 14 {
        2
    } else {
        1
    };
    let name_rows = u16::try_from(4 * scale).unwrap_or(4);
    // The leaderboard's box under the title, as many of its runs as the screen has room for;
    // the settings on the last line.
    // The leaderboard, the arcade's: its room kept in either mode, the title staying in place.
    let runs = leaderboard.entries().len();
    let room = usize::from(area.height.saturating_sub(name_rows + 15));
    let shown = runs.min(room);
    let board_rows = if shown == 0 {
        0
    } else {
        u16::try_from(shown).unwrap_or(0) + 2
    };
    let [_, name, _, line, _, mode, _, start, _, board, _, _, last] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(name_rows),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(board_rows),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    if shown > 0 && view.mode == Mode::Arcade {
        render_board(frame, board, &leaderboard.entries()[..shown], frames);
    }
    pixels::render_sprite_scaled(frame.buffer_mut(), name, sprites, SpriteId::Title, scale);
    centred(frame, &fl!("title-tagline"), Style::new().fg(GREY), line);
    // The mode the run will be played in, changed with ←→.
    let mode_words = match view.mode {
        Mode::Standard => fl!("mode-standard"),
        Mode::Arcade => fl!("mode-arcade"),
    };
    centred(
        frame,
        &format!("◀  {mode_words}  ▶"),
        Style::new().fg(GOLD).add_modifier(Modifier::BOLD),
        mode,
    );
    // Blinking, as on an arcade cabinet: shown two thirds of each second.
    if frames % 30 < 20 {
        centred(
            frame,
            &fl!("title-start").to_uppercase(),
            Style::new().add_modifier(Modifier::BOLD),
            start,
        );
    }
    frame.render_widget(
        Line::styled(format!("{} ", version()), Style::new().fg(DARK_GREY)).right_aligned(),
        last,
    );
}

/// The game's version, as the title screen and the pause menu show it: `v0.3.0`.
fn version() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

/// Every key of a run, in a box over the map, the run standing still.
pub fn render_keys(frame: &mut Frame, map: Rect) {
    let lines = [
        fl!("keys-move"),
        fl!("keys-torch"),
        fl!("keys-dash"),
        fl!("keys-cards"),
        fl!("keys-gear"),
        fl!("keys-ping"),
        fl!("keys-pause"),
        fl!("keys-keys"),
    ];
    let height = u16::try_from(lines.len()).unwrap_or(6) + 4;
    let inside = boxed(
        frame,
        map,
        KEYS_WIDTH,
        height,
        &fl!("keys-title").to_uppercase(),
        "",
    );
    let lines: Vec<Line> = lines.into_iter().map(Line::from).collect();
    frame.render_widget(Paragraph::new(lines), inside);
}

/// The pause menu, `selected` its highlighted item.
pub fn render_pause(frame: &mut Frame, map: Rect, selected: usize) {
    let labels: Vec<String> = PauseItem::ALL.iter().map(|item| item.label()).collect();
    render_list(frame, map, &fl!("pause-title"), &labels, selected, 28);
}

/// The options menu, `selected` its highlighted item, the settings as `view` holds them.
pub fn render_options(
    frame: &mut Frame,
    map: Rect,
    selected: usize,
    from_pause: bool,
    view: ViewState,
) {
    let labels: Vec<String> = OptionItem::items(from_pause)
        .iter()
        .map(|item| item.label(view))
        .collect();
    render_list(frame, map, &fl!("options-title"), &labels, selected, 44);
}

/// A menu in its box: its `title` on the frame, the game's version under it, its `labels` one a
/// line, the `selected` one marked.
fn render_list(
    frame: &mut Frame,
    map: Rect,
    title: &str,
    labels: &[String],
    selected: usize,
    width: u16,
) {
    let height = u16::try_from(labels.len()).unwrap_or(3) + 4;
    let inside = boxed(frame, map, width, height, &title.to_uppercase(), &version());
    let lines: Vec<Line> = labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            if index == selected {
                Line::styled(
                    format!("▶ {}", label.to_uppercase()),
                    Style::new().fg(SELECTED).add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(format!("  {}", label.to_uppercase()))
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inside);
}

/// The death screen, where the player fell, under the storm: the run's end in big red letters,
/// its summary, and the leaderboard. When the run made it, `name` holds the letters being typed in and the
/// one being changed: the run shows in its place among the best, as typed so far, in gold.
pub fn render_death(
    frame: &mut Frame,
    area: Rect,
    sprites: &Sprites,
    view: ViewState,
    world: &World,
    leaderboard: &Leaderboard,
    name: Option<([u8; 3], usize)>,
) {
    pixels::render_death_scene(frame.buffer_mut(), area, world, sprites, view);
    let dawn = world.survived();
    let (title, colors) = if dawn {
        (fl!("dawn-title"), DAWN_COLORS)
    } else {
        (fl!("death-title"), DEATH_COLORS)
    };
    // Twice as big where it fits.
    let wide = font::width(&title) * 2 + 4 <= i32::from(area.width);
    let scale = if wide && area.height >= 26 { 2 } else { 1 };
    let word_rows = u16::try_from(4 * scale).unwrap_or(4);

    // While the name is typed in, the run in its place, with the letters so far. A standard run
    // keeps no score, nor leaderboard.
    let arcade = world.mode == Mode::Arcade;
    let mut board = if arcade {
        leaderboard.clone()
    } else {
        Leaderboard::default()
    };
    let highlight = match name {
        Some((letters, _)) => board.insert(Entry {
            name: String::from_utf8_lossy(&letters).into_owned(),
            score: world.score,
            seconds: world.seconds(),
            level: world.player.level,
        }),
        None => view.highlight,
    };
    let runs = board.entries();
    let board_rows = if runs.is_empty() {
        0
    } else {
        u16::try_from(runs.len()).unwrap_or(0) + 3
    };
    let [_, big, under, _, score, summary, _, prompt, board_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(word_rows),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(board_rows),
        Constraint::Fill(2),
    ])
    .areas(area);
    font::render(frame.buffer_mut(), big, sprites, &title, colors, scale);
    if dawn {
        label(
            frame,
            &fl!("dawn-line"),
            Style::new().fg(DAWN_COLORS.0).add_modifier(Modifier::BOLD),
            under,
        );
    }
    if arcade {
        label(
            frame,
            &fl!("over-points", score = world.score),
            Style::new().fg(GOLD).add_modifier(Modifier::BOLD),
            score,
        );
    }
    let line = [
        fl!("over-survived", time = clock(world.seconds())),
        fl!(
            "over-score",
            level = world.player.level,
            kills = world.kills
        ),
    ]
    .join(" · ");
    label(frame, &line, Style::new().fg(GREY), summary);
    match name {
        Some((letters, cursor)) => {
            let place = highlight.unwrap_or(0);
            render_name_prompt(frame, prompt, letters, cursor, place, view.frame);
        }
        // Blinking, as on the title screen.
        None if view.frame % 30 < 20 => label(
            frame,
            &fl!("death-again"),
            Style::new().add_modifier(Modifier::BOLD),
            prompt,
        ),
        None => {}
    }
    if runs.is_empty() {
        return;
    }
    render_death_board(frame, board_area, runs, highlight, view.frame);
}

/// The death screen's leaderboard, in its box under the words: the run at `highlight` in gold,
/// gleaming if it is the best.
fn render_death_board(
    frame: &mut Frame,
    board_area: Rect,
    runs: &[Entry],
    highlight: Option<usize>,
    frames: u32,
) {
    let [_, boxed_board] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(board_area);
    let [boxed_board] = Layout::horizontal([Constraint::Length(BOARD_WIDTH)])
        .flex(Flex::Center)
        .areas(boxed_board);
    frame.render_widget(Blank, boxed_board);
    let block = Block::bordered()
        .border_style(Style::new().fg(DARK_GREY))
        .title(Line::from(format!(" {} ", fl!("leaderboard-title"))).centered());
    let rows = block.inner(boxed_board);
    frame.render_widget(block, boxed_board);
    for (place, (entry, row)) in runs.iter().zip(rows.rows()).enumerate() {
        let line = board_line(place, entry);
        match highlight {
            // The run, a new high score: gleaming.
            Some(0) if place == 0 => centred_spans(frame, shimmer(&line, frames), row),
            Some(run) if run == place => centred_as_is(
                frame,
                &line,
                Style::new().fg(GOLD).add_modifier(Modifier::BOLD),
                row,
            ),
            _ => centred_as_is(frame, &line, Style::new(), row),
        }
    }
}

/// One line centred in `area` on its own dark ground, a column wider each side: readable over
/// the death screen's scene.
fn label(frame: &mut Frame, text: &str, style: Style, area: Rect) {
    let width = u16::try_from(text.chars().count() + 2).unwrap_or(area.width);
    let [ground] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    frame.render_widget(Blank, ground);
    centred_as_is(frame, text, style, ground);
}

/// A run on the leaderboard at `place` (from 0): the prompt for the three letters, the one at
/// `cursor` underlined, on its own dark ground. A new high score only for the first place.
fn render_name_prompt(
    frame: &mut Frame,
    area: Rect,
    letters: [u8; 3],
    cursor: usize,
    place: usize,
    frames: u32,
) {
    let gold = Style::new().fg(GOLD).add_modifier(Modifier::BOLD);
    // A new high score gleams; a lower place is plain gold.
    let (lead, mut spans) = if place == 0 {
        let news = fl!("name-title");
        (
            format!("{news} {} ", fl!("name-prompt")),
            shimmer(&news, frames),
        )
    } else {
        let shown = place + 1;
        let news = fl!("name-place", place = shown);
        (
            format!("{news} {} ", fl!("name-prompt")),
            vec![Span::styled(news, gold)],
        )
    };
    spans.push(Span::styled(format!(" {} ", fl!("name-prompt")), gold));
    for (index, letter) in letters.iter().enumerate() {
        let style = if index == cursor {
            Style::new()
                .fg(SELECTED)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::new().add_modifier(Modifier::BOLD)
        };
        spans.push(Span::styled(char::from(*letter).to_string(), style));
        if index < 2 {
            spans.push(Span::raw(" "));
        }
    }
    // Centred as one line: the lead, then three letters and the spaces between them.
    let width = u16::try_from(lead.chars().count() + 5).unwrap_or(area.width);
    let [ground] = Layout::horizontal([Constraint::Length(width + 2)])
        .flex(Flex::Center)
        .areas(area);
    frame.render_widget(Blank, ground);
    spans.insert(0, Span::raw(" "));
    frame.render_widget(Paragraph::new(Line::from(spans)), ground);
}

/// A run on the leaderboard: its place, its name and its score.
fn board_line(place: usize, entry: &Entry) -> String {
    format!("{:>2}. {}  {:>7}", place + 1, entry.name, entry.score)
}

/// The best runs in a box, its title on its frame, the runs in columns.
fn render_board(frame: &mut Frame, area: Rect, runs: &[Entry], frames: u32) {
    let [area] = Layout::horizontal([Constraint::Length(BOARD_WIDTH)])
        .flex(Flex::Center)
        .areas(area);
    frame.render_widget(Blank, area);
    let block = Block::bordered()
        .border_style(Style::new().fg(DARK_GREY))
        .title(Line::from(format!(" {} ", fl!("leaderboard-title"))).centered());
    let inside = block.inner(area);
    frame.render_widget(block, area);
    for (place, (entry, row)) in runs.iter().zip(inside.rows()).enumerate() {
        let line = board_line(place, entry);
        // The best run gleams.
        if place == 0 {
            centred_spans(frame, shimmer(&line, frames), row);
        } else {
            centred_as_is(frame, &line, Style::new(), row);
        }
    }
}
