//! Text laid out by hand: centred the same way everywhere.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

/// Text centred in `area`, wrapped at its width. A line that can't sit exactly in the middle
/// leaves its spare column on the right, as ratatui does for a block's title: all the text of a
/// box leans the same way.
pub fn centred(frame: &mut Frame, text: &str, style: Style, area: Rect) {
    let width = usize::from(area.width);
    let lines: Vec<Line> = wrap(text, width)
        .into_iter()
        .map(|line| {
            let left = width.saturating_sub(line.chars().count()) / 2;
            Line::styled(format!("{}{line}", " ".repeat(left)), style)
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

/// One line centred in `area` as it is, its spaces kept: for columns, as the leaderboard's.
pub fn centred_as_is(frame: &mut Frame, text: &str, style: Style, area: Rect) {
    let left = usize::from(area.width).saturating_sub(text.chars().count()) / 2;
    frame.render_widget(
        Paragraph::new(Line::styled(format!("{}{text}", " ".repeat(left)), style)),
        area,
    );
}

/// A line centred in `area` as it is, made of spans: for a line not all in one style.
pub fn centred_spans(frame: &mut Frame, spans: Vec<Span<'static>>, area: Rect) {
    let width: usize = spans.iter().map(|span| span.content.chars().count()).sum();
    let left = usize::from(area.width).saturating_sub(width) / 2;
    let mut line = vec![Span::raw(" ".repeat(left))];
    line.extend(spans);
    frame.render_widget(Paragraph::new(Line::from(line)), area);
}

/// Text in gold with a gleam sweeping across it, left to right, `frame` frames in: a letter
/// per span, each its own shade, brightest where the gleam is.
pub fn shimmer(text: &str, frame: u32) -> Vec<Span<'static>> {
    let length = i32::try_from(text.chars().count()).unwrap_or(0);
    // The gleam crosses the text and past each end, then rests a while before it starts over.
    let course = length + 2 * GLEAM_WIDTH + GLEAM_REST;
    let step = i32::try_from(frame % 10_000).unwrap_or(0) / GLEAM_FRAMES_PER_LETTER;
    let gleam = step.rem_euclid(course.max(1)) - GLEAM_WIDTH;
    (0..)
        .zip(text.chars())
        .map(|(index, letter)| {
            let near = GLEAM_WIDTH - (index - gleam).abs();
            let light = f32::from(u8::try_from(near.max(0)).unwrap_or(0))
                / f32::from(u8::try_from(GLEAM_WIDTH).unwrap_or(1));
            let style = Style::new()
                .fg(blend(GOLD, GLEAM, light))
                .add_modifier(Modifier::BOLD);
            Span::styled(letter.to_string(), style)
        })
        .collect()
}

/// How many letters the gleam spans, each side of its brightest, how fast it moves, and how
/// long it rests between two sweeps, in letters' time.
const GLEAM_WIDTH: i32 = 2;
const GLEAM_FRAMES_PER_LETTER: i32 = 3;
const GLEAM_REST: i32 = 20;
const GOLD: (u8, u8, u8) = (240, 200, 60);
/// A paler gold, not white: the gleam stays discreet.
const GLEAM: (u8, u8, u8) = (255, 232, 150);

/// A colour `share` of the way from `from` to `to`.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "between two channels, so between 0 and 255"
)]
fn blend(from: (u8, u8, u8), to: (u8, u8, u8), share: f32) -> Color {
    let channel =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * share).round() as u8;
    Color::Rgb(
        channel(from.0, to.0),
        channel(from.1, to.1),
        channel(from.2, to.2),
    )
}

/// Words gathered into lines no wider than `width`; a word wider alone gets a line of its own.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_wrap_at_the_width() {
        assert_eq!(
            wrap("Gems fly in from 1 tile further", 18),
            ["Gems fly in from 1", "tile further"]
        );
        assert_eq!(wrap("Heal 20 HP", 18), ["Heal 20 HP"]);
    }

    #[test]
    fn the_gleam_brightens_the_letters_it_passes() {
        // Three frames per letter: at frame 6, the gleam is on the first letter.
        let spans = shimmer("RECORD", 6);
        let colour = |index: usize| spans[index].style.fg;
        assert_eq!(colour(0), Some(Color::Rgb(255, 232, 150)));
        assert_eq!(colour(5), Some(Color::Rgb(240, 200, 60)));
    }
}
