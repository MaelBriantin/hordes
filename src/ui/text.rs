//! Text laid out by hand: centred the same way everywhere.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
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

/// Words gathered into lines no wider than `width`; a word wider alone gets a line of its own.
fn wrap(text: &str, width: usize) -> Vec<String> {
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
}
