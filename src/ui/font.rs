//! A pixel font, 5 × 7, for the big words of the title and death screens: capitals only, an
//! accented letter drawn as its plain one.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::pixels::Canvas;
use super::sprites::Sprites;

/// A letter's height, in pixels; its width is 5, then a pixel of space.
const HEIGHT: i32 = 7;
const WIDTH: i32 = 5;
/// Where a word's lighter top gives way to its darker bottom, in rows of a letter.
const SHADE_FROM: i32 = 4;

/// A letter's pixels, row by row; none for a character the font lacks.
fn glyph(letter: char) -> Option<[&'static str; 7]> {
    let rows = match plain(letter) {
        'A' => [
            ".XXX.", "X...X", "X...X", "XXXXX", "X...X", "X...X", "X...X",
        ],
        'B' => [
            "XXXX.", "X...X", "X...X", "XXXX.", "X...X", "X...X", "XXXX.",
        ],
        'C' => [
            ".XXXX", "X....", "X....", "X....", "X....", "X....", ".XXXX",
        ],
        'D' => [
            "XXXX.", "X...X", "X...X", "X...X", "X...X", "X...X", "XXXX.",
        ],
        'E' => [
            "XXXXX", "X....", "X....", "XXXX.", "X....", "X....", "XXXXX",
        ],
        'F' => [
            "XXXXX", "X....", "X....", "XXXX.", "X....", "X....", "X....",
        ],
        'G' => [
            ".XXXX", "X....", "X....", "X..XX", "X...X", "X...X", ".XXXX",
        ],
        'H' => [
            "X...X", "X...X", "X...X", "XXXXX", "X...X", "X...X", "X...X",
        ],
        'I' => [
            "XXXXX", "..X..", "..X..", "..X..", "..X..", "..X..", "XXXXX",
        ],
        'J' => [
            "..XXX", "...X.", "...X.", "...X.", "...X.", "X..X.", ".XX..",
        ],
        'K' => [
            "X...X", "X..X.", "X.X..", "XX...", "X.X..", "X..X.", "X...X",
        ],
        'L' => [
            "X....", "X....", "X....", "X....", "X....", "X....", "XXXXX",
        ],
        'M' => [
            "X...X", "XX.XX", "X.X.X", "X.X.X", "X...X", "X...X", "X...X",
        ],
        'N' => [
            "X...X", "XX..X", "X.X.X", "X..XX", "X...X", "X...X", "X...X",
        ],
        'O' => [
            ".XXX.", "X...X", "X...X", "X...X", "X...X", "X...X", ".XXX.",
        ],
        'P' => [
            "XXXX.", "X...X", "X...X", "XXXX.", "X....", "X....", "X....",
        ],
        'Q' => [
            ".XXX.", "X...X", "X...X", "X...X", "X.X.X", "X..X.", ".XX.X",
        ],
        'R' => [
            "XXXX.", "X...X", "X...X", "XXXX.", "X.X..", "X..X.", "X...X",
        ],
        'S' => [
            ".XXXX", "X....", "X....", ".XXX.", "....X", "....X", "XXXX.",
        ],
        'T' => [
            "XXXXX", "..X..", "..X..", "..X..", "..X..", "..X..", "..X..",
        ],
        'U' => [
            "X...X", "X...X", "X...X", "X...X", "X...X", "X...X", ".XXX.",
        ],
        'V' => [
            "X...X", "X...X", "X...X", "X...X", "X...X", ".X.X.", "..X..",
        ],
        'W' => [
            "X...X", "X...X", "X...X", "X.X.X", "X.X.X", "XX.XX", "X...X",
        ],
        'X' => [
            "X...X", "X...X", ".X.X.", "..X..", ".X.X.", "X...X", "X...X",
        ],
        'Y' => [
            "X...X", "X...X", ".X.X.", "..X..", "..X..", "..X..", "..X..",
        ],
        'Z' => [
            "XXXXX", "....X", "...X.", "..X..", ".X...", "X....", "XXXXX",
        ],
        '!' => [
            "..X..", "..X..", "..X..", "..X..", "..X..", ".....", "..X..",
        ],
        ' ' => [
            ".....", ".....", ".....", ".....", ".....", ".....", ".....",
        ],
        _ => return None,
    };
    Some(rows)
}

/// A letter as the font has it: a capital, without its accent.
fn plain(letter: char) -> char {
    match letter.to_uppercase().next().unwrap_or(letter) {
        'À' | 'Â' | 'Ä' => 'A',
        'Ç' => 'C',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'Î' | 'Ï' => 'I',
        'Ô' | 'Ö' => 'O',
        'Ù' | 'Û' | 'Ü' => 'U',
        other => other,
    }
}

/// How wide a word is in the font, in pixels, at scale 1.
pub fn width(word: &str) -> i32 {
    let letters = i32::try_from(word.chars().filter(|c| glyph(*c).is_some()).count()).unwrap_or(0);
    (letters * (WIDTH + 1) - 1).max(0)
}

/// A word in big letters, `scale` times as big, in the middle of `area`: lighter on top, darker
/// below, as the title.
pub fn render(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    word: &str,
    colors: (Color, Color),
    scale: i32,
) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let (left, top) = (-width(word) * scale / 2, -HEIGHT * scale / 2);
    let mut x = left;
    for rows in word.chars().filter_map(glyph) {
        for (y, row) in (0..).zip(rows) {
            let color = if y < SHADE_FROM { colors.0 } else { colors.1 };
            for (dx, pixel) in (0..).zip(row.chars()) {
                if pixel != 'X' {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        canvas.set((x + dx * scale + sx, top + y * scale + sy), color);
                    }
                }
            }
        }
        x += (WIDTH + 1) * scale;
    }
    canvas.draw(buffer, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_accented_letter_is_drawn_as_its_plain_one() {
        assert_eq!(glyph('é'), glyph('E'));
    }

    #[test]
    fn a_word_is_as_wide_as_its_letters_and_the_gaps_between_them() {
        assert_eq!(width("DEAD"), 4 * 6 - 1);
        assert_eq!(width(""), 0);
    }
}
