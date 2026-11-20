//! A pixel font, 5 × 7, for the big words of the title and death screens: capitals only, an
//! accented letter drawn as its plain one.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::pixels::Canvas;
use super::sprites::Sprites;

/// A letter's width, in pixels, then a pixel of space; its height is 7.
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
    let shade = |_, y| if y < SHADE_FROM { colors.0 } else { colors.1 };
    render_with(buffer, area, sprites, word, shade, scale);
}

/// A word in big letters, `scale` times as big, in the middle of `area`, each pixel in the
/// colour `color_at` gives it, by its column across the word and its row in the letter.
pub fn render_with(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    word: &str,
    color_at: impl Fn(i32, i32) -> Color,
    scale: i32,
) {
    paint(buffer, area, sprites, (word, glyph, WIDTH), color_at, scale);
}

/// A word in small letters, 3 × 5, in the middle of `area`, each pixel in the colour
/// `color_at` gives it, by its column across the word and its row in the letter.
pub fn render_small_with(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    word: &str,
    color_at: impl Fn(i32, i32) -> Color,
) {
    paint(
        buffer,
        area,
        sprites,
        (word, small_glyph, SMALL_WIDTH),
        color_at,
        1,
    );
}

/// A font's letters: each one's rows of pixels, if it has it.
type Glyphs<const ROWS: usize> = fn(char) -> Option<[&'static str; ROWS]>;

/// A word in the letters `glyph` gives, each `width` wide, in the middle of `area`: its top on
/// a cell's top row, for its rows not to straddle the cells' halves.
fn paint<const ROWS: usize>(
    buffer: &mut Buffer,
    area: Rect,
    sprites: &Sprites,
    (word, glyph, width): (&str, Glyphs<ROWS>, i32),
    color_at: impl Fn(i32, i32) -> Color,
    scale: i32,
) {
    let mut canvas = Canvas::new(sprites, area, (0, 0));
    let letters = word.chars().filter_map(glyph).collect::<Vec<_>>();
    let wide = (i32::try_from(letters.len()).unwrap_or(0) * (width + 1) - 1).max(0);
    let tall = i32::try_from(ROWS).unwrap_or(0) * scale;
    let height = i32::from(area.height) * 2;
    // From the canvas's middle, the top row: on an even pixel of the area.
    let top = ((height - tall) / 2).max(0) / 2 * 2 - height / 2;
    let left = -wide * scale / 2;
    let mut x = left;
    for rows in letters {
        for (y, row) in (0..).zip(rows) {
            for (dx, pixel) in (0..).zip(row.chars()) {
                if pixel != 'X' {
                    continue;
                }
                let color = color_at((x - left) / scale + dx, y);
                for sy in 0..scale {
                    for sx in 0..scale {
                        canvas.set((x + dx * scale + sx, top + y * scale + sy), color);
                    }
                }
            }
        }
        x += (width + 1) * scale;
    }
    canvas.draw(buffer, area);
}

/// A small letter's width, in pixels; its height is 5, then a pixel of space after it.
const SMALL_WIDTH: i32 = 3;

/// A small letter's pixels, row by row; none for a character the font lacks.
fn small_glyph(letter: char) -> Option<[&'static str; 5]> {
    let rows = match plain(letter) {
        'A' => [".X.", "X.X", "XXX", "X.X", "X.X"],
        'B' => ["XX.", "X.X", "XX.", "X.X", "XX."],
        'C' => [".XX", "X..", "X..", "X..", ".XX"],
        'D' => ["XX.", "X.X", "X.X", "X.X", "XX."],
        'E' => ["XXX", "X..", "XX.", "X..", "XXX"],
        'F' => ["XXX", "X..", "XX.", "X..", "X.."],
        'G' => [".XX", "X..", "X.X", "X.X", ".XX"],
        'H' => ["X.X", "X.X", "XXX", "X.X", "X.X"],
        'I' => ["XXX", ".X.", ".X.", ".X.", "XXX"],
        'J' => ["..X", "..X", "..X", "X.X", ".X."],
        'K' => ["X.X", "X.X", "XX.", "X.X", "X.X"],
        'L' => ["X..", "X..", "X..", "X..", "XXX"],
        'M' => ["X.X", "XXX", "XXX", "X.X", "X.X"],
        'N' => ["XX.", "X.X", "X.X", "X.X", "X.X"],
        'O' => [".X.", "X.X", "X.X", "X.X", ".X."],
        'P' => ["XX.", "X.X", "XX.", "X..", "X.."],
        'Q' => [".X.", "X.X", "X.X", "XX.", ".XX"],
        'R' => ["XX.", "X.X", "XX.", "X.X", "X.X"],
        'S' => [".XX", "X..", ".X.", "..X", "XX."],
        'T' => ["XXX", ".X.", ".X.", ".X.", ".X."],
        'U' => ["X.X", "X.X", "X.X", "X.X", "XXX"],
        'V' => ["X.X", "X.X", "X.X", "X.X", ".X."],
        'W' => ["X.X", "X.X", "XXX", "XXX", "X.X"],
        'X' => ["X.X", "X.X", ".X.", "X.X", "X.X"],
        'Y' => ["X.X", "X.X", ".X.", ".X.", ".X."],
        'Z' => ["XXX", "..X", ".X.", "X..", "XXX"],
        '0' => ["XXX", "X.X", "X.X", "X.X", "XXX"],
        '1' => [".X.", "XX.", ".X.", ".X.", "XXX"],
        '2' => ["XXX", "..X", "XXX", "X..", "XXX"],
        '3' => ["XXX", "..X", ".XX", "..X", "XXX"],
        '4' => ["X.X", "X.X", "XXX", "..X", "..X"],
        '5' => ["XXX", "X..", "XXX", "..X", "XXX"],
        '6' => ["XXX", "X..", "XXX", "X.X", "XXX"],
        '7' => ["XXX", "..X", "..X", ".X.", ".X."],
        '8' => ["XXX", "X.X", "XXX", "X.X", "XXX"],
        '9' => ["XXX", "X.X", "XXX", "..X", "XXX"],
        '·' => ["...", "...", ".X.", "...", "..."],
        '+' => ["...", ".X.", "XXX", ".X.", "..."],
        '-' => ["...", "...", "XXX", "...", "..."],
        '!' => [".X.", ".X.", ".X.", "...", ".X."],
        '?' => ["XX.", "..X", ".X.", "...", ".X."],
        ':' => ["...", ".X.", "...", ".X.", "..."],
        '.' => ["...", "...", "...", "...", ".X."],
        ',' => ["...", "...", "...", ".X.", "X.."],
        '\'' | '’' => [".X.", ".X.", "...", "...", "..."],
        ' ' => ["...", "...", "...", "...", "..."],
        _ => return None,
    };
    Some(rows)
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
        assert_eq!(small_glyph('À'), small_glyph('A'));
    }
}
