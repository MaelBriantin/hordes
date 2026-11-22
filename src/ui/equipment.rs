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
/// The room around what the box holds: its borders, and a cell inside them, each way.
const ROOM: (u16, u16) = (6, 4);
const WHITE: Color = Color::Rgb(235, 235, 238);
const GREY: Color = Color::Rgb(140, 140, 150);
const LEVEL: Color = Color::Rgb(110, 160, 240);
const MAX: Color = Color::Rgb(240, 190, 60);
const READY: Color = Color::Rgb(110, 200, 110);

/// The rows the box takes, all its cards shown with their icons: the hero's picture, then each
/// card's, a row apart.
pub fn height(world: &World) -> u16 {
    let cards = u16::try_from(world.player.build.picked().len()).unwrap_or(0);
    ROOM.1 + PICTURE.1 + cards * (PICTURE.1 + 1)
}

/// The box, in `area`: as tall as its cards need, or as `area` allows, the cards then a line
/// each, without their icons.
pub fn render(frame: &mut Frame, area: Rect, world: &World, sprites: &Sprites, view: ViewState) {
    let full = height(world);
    let compact = full > area.height;
    let picked = world.player.build.picked();
    let wanted = if compact {
        ROOM.1 + PICTURE.1 + 1 + u16::try_from(picked.len()).unwrap_or(0)
    } else {
        full
    };
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
    let mut y = inside.y;
    hero(frame, inside, y, world, sprites, view);
    y += PICTURE.1 + 1;
    for &choice in picked {
        if y + if compact { 1 } else { PICTURE.1 } > inside.y + inside.height {
            break;
        }
        if compact {
            card_line(frame, inside, y, world, choice);
            y += 1;
        } else {
            card(frame, inside, y, world, sprites, choice);
            y += PICTURE.1 + 1;
        }
    }
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

/// A card: its icon, its name and level, then what it brings at that level, wrapped.
fn card(frame: &mut Frame, inside: Rect, y: u16, world: &World, sprites: &Sprites, choice: Choice) {
    let picture = Rect::new(inside.x, y, PICTURE.0, PICTURE.1);
    pixels::render_sprite(frame.buffer_mut(), picture, sprites, level_up::icon(choice));
    let width = inside.width.saturating_sub(PICTURE.0 + GAP);
    let words = Rect::new(inside.x + PICTURE.0 + GAP, y, width, PICTURE.1);
    let level = level_of(world, choice);
    let mut lines = vec![title(choice, level)];
    for line in text::wrap(
        &brings(choice, level, world.player.build.hero()),
        usize::from(width),
    )
    .into_iter()
    .take(usize::from(PICTURE.1 - 1))
    {
        lines.push(Line::styled(line, Style::new().fg(GREY)));
    }
    frame.render_widget(Paragraph::new(lines), words);
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

/// What a card brings at `level`, to `hero`: what it does, then what each level
/// after the first added.
fn brings(choice: Choice, level: u8, hero: Skin) -> String {
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
        assert_eq!(
            brings(Choice::Passive(Passive::Might), 1, Skin::Survivor),
            level_up::gain_of(Choice::Passive(Passive::Might), 1, Skin::Survivor)
        );
    }

    #[test]
    fn the_box_grows_with_the_cards_taken() {
        let mut world = World::new(1);
        let first = height(&world);
        world.player.build.take(Choice::Passive(Passive::Might));
        assert_eq!(height(&world), first + PICTURE.1 + 1);
    }
}
