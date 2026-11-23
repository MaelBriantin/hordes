//! What the player has, on the pause screen beside its menu: their hero, with their perk and
//! their action, then each card taken, its icon, its level and what it brings at it.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};

use super::sprites::{self, Sprites};
use super::{Blank, ViewState, level_up, pixels, skins, text};
use crate::game::{Action, Choice, Facing, MAX_LEVEL, Skin, World};

/// How wide the box is at most, and at least, in cells, borders and room included: narrower,
/// it doesn't show.
pub const WIDTH: u16 = 74;
pub const MIN_WIDTH: u16 = 48;
/// A picture's room, in cells: an icon's, or the hero's, twice as big: 8 cells wide, 4 high;
/// and the room between it and its words.
const PICTURE: (u16, u16) = (8, 4);
const GAP: u16 = 2;
/// The most lines of what a card brings shown.
const TEXT_LINES: usize = 5;
/// The room around what the box holds: its borders, and a cell inside them, each way.
const ROOM: (u16, u16) = (6, 4);
const WHITE: Color = Color::Rgb(235, 235, 238);
const GREY: Color = Color::Rgb(140, 140, 150);
const LEVEL: Color = Color::Rgb(110, 160, 240);
const MAX: Color = Color::Rgb(240, 190, 60);
const READY: Color = Color::Rgb(110, 200, 110);

/// The box, in `area`: as tall as its cards need, or as `area` allows, the cards then a line
/// each, without their icons.
pub fn render(frame: &mut Frame, area: Rect, world: &World, sprites: &Sprites, view: ViewState) {
    let picked = world.player.build.picked();
    let inner = area.width.saturating_sub(ROOM.0);
    // The cards with their icons, in a column, else in two; a line each, without, for want of
    // room.
    let laid = [1, 2].into_iter().find_map(|columns| {
        let width = column_width(inner, columns);
        let heights = row_heights(world, picked, columns, width);
        let wanted = ROOM.1 + PICTURE.1 + heights.iter().map(|height| height + 1).sum::<u16>();
        (wanted <= area.height).then_some((columns, width, heights, wanted))
    });
    let wanted = laid.as_ref().map_or(
        ROOM.1 + PICTURE.1 + 1 + u16::try_from(picked.len()).unwrap_or(0),
        |laid| laid.3,
    );
    let box_area = Rect {
        y: area.y + area.height.saturating_sub(wanted.min(area.height)) / 2,
        height: wanted.min(area.height),
        ..area
    };
    frame.render_widget(Blank, box_area);
    let block = Block::bordered()
        .padding(Padding::new(2, 2, 1, 1))
        .title(Line::from(format!(" {} ", fl!("equipment-title").to_uppercase())).centered());
    let inside = block.inner(box_area);
    frame.render_widget(block, box_area);
    hero(frame, inside, inside.y, world, sprites, view);
    let top = inside.y + PICTURE.1 + 1;
    let bottom = inside.y + inside.height;
    let Some((columns, width, heights, _)) = laid else {
        for (row, &choice) in (0..).zip(picked) {
            if top + row >= bottom {
                break;
            }
            card_line(frame, inside, top + row, world, choice);
        }
        return;
    };
    let mut y = top;
    for (row, height) in heights.iter().enumerate() {
        for (column, &choice) in (0_u16..).zip(
            picked
                .iter()
                .skip(row * usize::from(columns))
                .take(usize::from(columns)),
        ) {
            let room = Rect::new(inside.x + column * (width + GAP), y, width, *height);
            card(frame, room, world, sprites, choice);
        }
        y += height + 1;
    }
}

/// How wide a column of cards is, `inner` cells wide in all, `columns` of them a gap apart.
fn column_width(inner: u16, columns: u16) -> u16 {
    (inner + GAP) / columns - GAP
}

/// How tall each row of cards is, `columns` a row, `width` wide: its tallest card, an icon's
/// height at least.
fn row_heights(world: &World, picked: &[Choice], columns: u16, width: u16) -> Vec<u16> {
    picked
        .chunks(usize::from(columns))
        .map(|row| {
            row.iter()
                .map(|choice| u16::try_from(card_lines(world, *choice, width).len()).unwrap_or(0))
                .max()
                .unwrap_or(0)
                .max(PICTURE.1)
        })
        .collect()
}

/// The hero, from the front, twice as big: their outfit's name, their perk, their action.
fn hero(
    frame: &mut Frame,
    inside: Rect,
    y: u16,
    world: &World,
    sprites: &Sprites,
    view: ViewState,
) {
    let picture = Rect::new(inside.x, y, PICTURE.0, PICTURE.1);
    let look = sprites::hero(Facing::Down, view.outfit());
    pixels::render_look_scaled(frame.buffer_mut(), picture, sprites, look, 2);
    let words = Rect::new(
        inside.x + PICTURE.0 + GAP,
        y,
        inside.width.saturating_sub(PICTURE.0 + GAP),
        PICTURE.1,
    );
    let hero = world.player.build.hero();
    let mut lines = vec![
        Line::styled(
            skins::outfit_name(view.outfit()).to_uppercase(),
            Style::new().fg(WHITE).add_modifier(Modifier::BOLD),
        ),
        Line::styled(skins::perk_line(hero), Style::new().fg(GREY)),
    ];
    if let Some(action) = action_line(world) {
        lines.push(action);
    }
    frame.render_widget(Paragraph::new(lines), words);
}

/// The action the player has, if any: its name, its level, and whether it is ready.
fn action_line(world: &World) -> Option<Line<'static>> {
    let build = &world.player.build;
    let action = Action::ALL
        .into_iter()
        .find(|action| build.action_level(*action) > 0)?;
    let level = build.action_level(action);
    let ready = match action {
        Action::Dash => world.player.dashes.ready > 0,
        Action::Barricade => world.barricade_wait == 0,
        Action::Ram => world.ram_wait == 0,
        Action::C4 => world.c4_wait == 0,
        Action::Kit => world.kit_wait == 0,
    };
    let (state, colour) = if ready {
        (fl!("equipment-ready"), READY)
    } else {
        (fl!("equipment-charging"), GREY)
    };
    Some(Line::from(vec![
        Span::styled(
            fl!(
                "equipment-action",
                action = level_up::name(Choice::Action(action)),
                level = level
            ),
            Style::new().fg(WHITE),
        ),
        Span::styled(format!("  {state}"), Style::new().fg(colour)),
    ]))
}

/// A card in `room`: its icon, then its name and level and what it brings, wrapped.
fn card(frame: &mut Frame, room: Rect, world: &World, sprites: &Sprites, choice: Choice) {
    let picture = Rect::new(room.x, room.y, PICTURE.0, PICTURE.1);
    pixels::render_sprite(frame.buffer_mut(), picture, sprites, level_up::icon(choice));
    let width = room.width.saturating_sub(PICTURE.0 + GAP);
    let words = Rect::new(room.x + PICTURE.0 + GAP, room.y, width, room.height);
    frame.render_widget(Paragraph::new(card_lines(world, choice, width)), words);
}

/// A card's words, `width` wide: its name and level (the level on a line of its own if they
/// don't fit together), then what it brings, wrapped, a few lines at most.
fn card_lines(world: &World, choice: Choice, width: u16) -> Vec<Line<'static>> {
    let level = level_of(world, choice);
    let title = title(choice, level);
    let mut lines = if title.width() <= usize::from(width) {
        vec![title]
    } else {
        title
            .spans
            .into_iter()
            .map(|span| Line::from(Span::styled(span.content.trim().to_string(), span.style)))
            .collect()
    };
    lines.extend(
        text::wrap(
            &brings(choice, level, world.player.build.hero()),
            usize::from(width),
        )
        .into_iter()
        .take(TEXT_LINES)
        .map(|line| Line::styled(line, Style::new().fg(GREY))),
    );
    lines
}

/// A card on a line, without its icon, for want of room: its name and level, then what it
/// brings, cut short.
fn card_line(frame: &mut Frame, inside: Rect, y: u16, world: &World, choice: Choice) {
    let level = level_of(world, choice);
    let mut spans = title(choice, level).spans;
    spans.push(Span::styled(
        format!("  {}", brings(choice, level, world.player.build.hero())),
        Style::new().fg(GREY),
    ));
    frame.render_widget(Line::from(spans), Rect::new(inside.x, y, inside.width, 1));
}

/// A card's name, bold, then its level, in gold once at its last.
fn title(choice: Choice, level: u8) -> Line<'static> {
    let (level_text, colour) = if level >= MAX_LEVEL {
        ("MAX".to_string(), MAX)
    } else {
        (fl!("equipment-level", level = level), LEVEL)
    };
    Line::from(vec![
        Span::styled(
            level_up::name(choice),
            Style::new().fg(WHITE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {level_text}"), Style::new().fg(colour)),
    ])
}

/// What a card brings at `level`, to `hero`: a weapon or an action, what it does, then what
/// each level after the first added; a passive, its own level's words, its total in them.
fn brings(choice: Choice, level: u8, hero: Skin) -> String {
    if matches!(choice, Choice::Passive(_)) {
        return level_up::gain_of(choice, level.max(1), hero);
    }
    (1..=level.max(1))
        .map(|level| level_up::gain_of(choice, level, hero))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The level a card is at in the player's build.
fn level_of(world: &World, choice: Choice) -> u8 {
    let build = &world.player.build;
    match choice {
        Choice::Weapon(weapon) => build.weapon_level(weapon),
        Choice::Passive(passive) => build.passive_level(passive),
        Choice::Action(action) => build.action_level(action),
        Choice::Legendary(_) | Choice::Heal => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Passive, Weapon};

    #[test]
    fn a_card_tells_what_it_does_then_what_each_level_after_added() {
        let choice = Choice::Weapon(Weapon::Truncheon);
        let three = brings(choice, 3, Skin::Survivor);
        assert_eq!(three.matches(" · ").count(), 2, "{three}");
        assert!(three.starts_with(&level_up::gain_of(choice, 1, Skin::Survivor)));
        assert!(three.ends_with(&level_up::gain_of(choice, 3, Skin::Survivor)));
        let might = Choice::Passive(Passive::Might);
        assert_eq!(
            brings(might, 3, Skin::Survivor),
            level_up::gain_of(might, 3, Skin::Survivor),
            "a passive, its own level's words alone: its total is in them"
        );
    }

    #[test]
    fn a_card_is_an_icon_tall_at_least_taller_for_its_words_in_a_narrow_column() {
        let mut world = World::new(1);
        world.player.build.take(Choice::Weapon(Weapon::Hammer));
        world.player.build.take(Choice::Weapon(Weapon::Hammer));
        world.player.build.take(Choice::Weapon(Weapon::Hammer));
        let picked = world.player.build.picked().to_vec();
        let wide = row_heights(&world, &picked, 1, 66);
        let narrow = row_heights(&world, &picked, 2, 32);
        assert!(wide.iter().all(|height| *height >= PICTURE.1));
        assert!(
            narrow.iter().max() > wide.iter().max(),
            "{narrow:?} {wide:?}"
        );
    }

    #[test]
    fn a_long_name_puts_its_level_on_a_line_of_its_own() {
        let world = World::new(1);
        let lines = card_lines(&world, Choice::Passive(Passive::HordeHunter), 16);
        assert!(lines[0].width() <= 16, "{:?}", lines[0]);
    }
}
