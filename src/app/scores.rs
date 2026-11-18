//! The leaderboard, kept between runs in `scores.txt` in the game's data folder: one run per
//! line, its name, score, seconds and level: `MAE 12345 342 7`.

use std::fs;
use std::io;

use super::settings::data_file;
use crate::game::{Entry, Leaderboard};

const FILE: &str = "scores.txt";

/// The leaderboard kept, or an empty one if there is none or it can't be read.
pub fn load() -> Leaderboard {
    data_file(FILE)
        .and_then(|path| fs::read_to_string(path).ok())
        .map_or_else(Leaderboard::default, |text| parse(&text))
}

/// Keeps the leaderboard for the next runs.
pub fn save(board: &Leaderboard) -> io::Result<()> {
    let path = data_file(FILE).ok_or_else(|| io::Error::other("no data folder"))?;
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    fs::write(path, text(board))
}

/// The runs of a leaderboard's text; a broken line is left out.
fn parse(text: &str) -> Leaderboard {
    let entries = text
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            Some(Entry {
                name: words.next()?.to_owned(),
                score: words.next()?.parse().ok()?,
                seconds: words.next()?.parse().ok()?,
                level: words.next()?.parse().ok()?,
            })
        })
        .collect();
    Leaderboard::new(entries)
}

fn text(board: &Leaderboard) -> String {
    let mut text = String::new();
    for entry in board.entries() {
        let line = [
            entry.name.clone(),
            entry.score.to_string(),
            entry.seconds.to_string(),
            entry.level.to_string(),
        ];
        text.push_str(&line.join(" "));
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leaderboard_reads_back_what_it_wrote() {
        let board = Leaderboard::new(vec![
            Entry {
                name: "MAE".to_owned(),
                score: 12_345,
                seconds: 342,
                level: 7,
            },
            Entry {
                name: "KID".to_owned(),
                score: 999,
                seconds: 60,
                level: 2,
            },
        ]);
        assert_eq!(parse(&text(&board)), board);
    }

    #[test]
    fn a_broken_line_is_left_out() {
        assert_eq!(parse("MAE lots 3 4\nKID 10 60 2\n").entries().len(), 1);
    }
}
