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
/// A card's icon in its frame, two pixels more each way, and a pixel more each way again: the
/// icon starting on an odd pixel row, as in the gear in play, its level on the same rows. The
/// hero's picture, twice as big, in as much room. And the room between a picture and its words.
const CARD_PICTURE: (u16, u16) = (12, 7);
/// Where a card's level ends in its picture, in cells: at the icon's last column, on the row of
/// its last pixel row and the room under it, inside the frame: as in the gear in play.
const LEVEL_RIGHT: u16 = 10;
const LEVEL_ROW: u16 = 5;
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
/// The rule between two groups of cards.
const RULE: Color = Color::Rgb(70, 72, 82);

/// The box, in `area`: as tall as its cards need, or as `area` allows, the cards then a line
/// each, without their icons. The cards in groups, a row apart: the hero's own weapon and
/// action, the other weapons, the passives, the legendaries.
pub fn render(frame: &mut Frame, area: Rect, world: &World, sprites: &Sprites, view: ViewState) {
    let groups = world.player.build.groups();
    // Between two groups, a rule: a row.
    let gaps = u16::try_from(groups.len().saturating_sub(1)).unwrap_or(0);
    let inner = area.width.saturating_sub(ROOM.0);
    // The cards with their icons, in a column, else in two; a line each, without, for want of
    // room.
    let laid = [1, 2].into_iter().find_map(|columns| {
        let width = column_width(inner, columns);
        let heights: Vec<Vec<u16>> = groups
            .iter()
            .map(|(_, cards)| row_heights(world, cards, columns, width))
            .collect();
        // The pictures' empty pixel rows, top and bottom, keep the rows apart: no row between.
        let rows: u16 = heights.iter().flatten().sum();
        let wanted = ROOM.1 + CARD_PICTURE.1 + gaps + rows;
        (wanted <= area.height).then_some((columns, width, heights, wanted))
    });
    let count = u16::try_from(world.player.build.picked().len()).unwrap_or(0);
    let wanted = laid
        .as_ref()
        .map_or(ROOM.1 + CARD_PICTURE.1 + gaps + count, |laid| laid.3);
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
    let bottom = inside.y + inside.height;
    let mut y = inside.y + CARD_PICTURE.1;
    let rule = |frame: &mut Frame, y: u16| {
        frame.render_widget(
            Line::styled("─".repeat(usize::from(inside.width)), Style::new().fg(RULE)),
            Rect::new(inside.x, y, inside.width, 1),
        );
    };
    let Some((columns, width, heights, _)) = laid else {
        for (index, (_, cards)) in groups.iter().enumerate() {
            if index > 0 && y < bottom {
                rule(frame, y);
                y += 1;
            }
            for &choice in cards {
                if y >= bottom {
                    return;
                }
                card_line(frame, inside, y, world, choice);
                y += 1;
            }
        }
        return;
    };
    for (index, ((_, cards), heights)) in groups.iter().zip(&heights).enumerate() {
        // The groups apart: a rule.
        if index > 0 {
            rule(frame, y);
            y += 1;
        }
        for (row, height) in heights.iter().enumerate() {
            for (column, &choice) in (0_u16..).zip(
                cards
                    .iter()
                    .skip(row * usize::from(columns))
                    .take(usize::from(columns)),
            ) {
                let room = Rect::new(inside.x + column * (width + GAP), y, width, *height);
                card(frame, room, world, sprites, choice);
            }
            y += height;
        }
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
            // Its words beside its framed icon.
            let words = width.saturating_sub(CARD_PICTURE.0 + GAP);
            row.iter()
                .map(|choice| u16::try_from(card_lines(world, *choice, words).len()).unwrap_or(0))
                .max()
                .unwrap_or(0)
                .max(CARD_PICTURE.1)
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
    // In the middle of as much room as a framed icon: their words in line with the cards'.
    let picture = Rect::new(inside.x, y, CARD_PICTURE.0, CARD_PICTURE.1);
    let look = sprites::hero(Facing::Down, view.outfit());
    pixels::render_look_scaled(frame.buffer_mut(), picture, sprites, look, 2);
    let words = Rect::new(
        inside.x + CARD_PICTURE.0 + GAP,
        y + 1,
        inside.width.saturating_sub(CARD_PICTURE.0 + GAP),
        CARD_PICTURE.1 - 1,
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
        Action::FireWall => world.fire_wall_wait == 0,
        Action::Smoke => world.smoke_wait == 0,
        Action::Gatling => world.gatling_wait == 0,
        Action::Vault => world.vault_wait == 0,
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
    let picture = Rect::new(room.x, room.y, CARD_PICTURE.0, CARD_PICTURE.1);
    pixels::render_card_icon(
        frame.buffer_mut(),
        picture,
        sprites,
        choice,
        world.spent(choice),
    );
    // Its level in its frame's bottom right corner, inside it, as in the gear in play; none for
    // a legendary.
    if !matches!(choice, Choice::Legendary(_) | Choice::Heal) {
        pixels::render_level_badge(
            frame.buffer_mut(),
            (picture.x + LEVEL_RIGHT, picture.y + LEVEL_ROW),
            level_of(world, choice),
        );
    }
    let width = room.width.saturating_sub(CARD_PICTURE.0 + GAP);
    let words = Rect::new(room.x + CARD_PICTURE.0 + GAP, room.y, width, room.height);
    frame.render_widget(Paragraph::new(card_lines(world, choice, width)), words);
}

/// A card's words, `width` wide: its name and level (the level on a line of its own if they
/// don't fit together), then what it brings, wrapped, a few lines at most.
fn card_lines(world: &World, choice: Choice, width: u16) -> Vec<Line<'static>> {
    // Its name alone: its level is on its icon, as in the gear in play.
    let level = level_of(world, choice);
    let mut lines = vec![Line::styled(
        level_up::name(choice),
        Style::new().fg(WHITE).add_modifier(Modifier::BOLD),
    )];
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
    fn a_card_is_as_tall_as_its_framed_icon_its_name_and_words_beside_it() {
        // Its name, and its words at most, beside its icon.
        assert!(u16::try_from(TEXT_LINES).unwrap() < CARD_PICTURE.1);
        let mut world = World::new(1);
        for _ in 0..3 {
            world.player.build.take(Choice::Weapon(Weapon::Pistol));
        }
        let picked = world.player.build.picked().to_vec();
        for (columns, width) in [(1, 66), (2, 24)] {
            let heights = row_heights(&world, &picked, columns, width);
            assert!(
                heights.iter().all(|height| *height == CARD_PICTURE.1),
                "{heights:?}"
            );
        }
    }

    #[test]
    fn a_card_s_words_start_with_its_name_alone_its_level_on_its_icon() {
        let world = World::new(1);
        let lines = card_lines(&world, Choice::Passive(Passive::HordeHunter), 30);
        assert_eq!(
            lines[0].to_string(),
            level_up::name(Choice::Passive(Passive::HordeHunter))
        );
    }
}
