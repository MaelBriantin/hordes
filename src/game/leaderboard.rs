//! The best runs: their scores, each with three letters for who played it, best first. Kept
//! between runs by the app; here, only what makes a score one of the best.

/// How many runs the leaderboard keeps.
pub const PLACES: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Three letters, arcade style.
    pub name: String,
    pub score: u64,
    /// The seconds the run lasted.
    pub seconds: u32,
    pub level: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Leaderboard {
    /// Best first.
    entries: Vec<Entry>,
}

impl Leaderboard {
    /// A leaderboard of these runs, sorted, the best kept.
    pub fn new(mut entries: Vec<Entry>) -> Self {
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.score));
        entries.truncate(PLACES);
        Self { entries }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Whether a run that scored `score` makes it onto the leaderboard.
    pub fn qualifies(&self, score: u64) -> bool {
        score > 0
            && (self.entries.len() < PLACES || self.entries.iter().any(|entry| score > entry.score))
    }

    /// Puts a run on the leaderboard, below those that scored as much or more. Its place, from 0;
    /// none if it didn't make it.
    pub fn insert(&mut self, entry: Entry) -> Option<usize> {
        if !self.qualifies(entry.score) {
            return None;
        }
        let place = self
            .entries
            .iter()
            .position(|other| entry.score > other.score)
            .unwrap_or(self.entries.len());
        self.entries.insert(place, entry);
        self.entries.truncate(PLACES);
        Some(place)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, score: u64) -> Entry {
        Entry {
            name: name.to_owned(),
            score,
            seconds: 60,
            level: 3,
        }
    }

    #[test]
    fn a_run_takes_its_place_below_those_that_scored_as_much() {
        let mut board = Leaderboard::new(vec![entry("AAA", 100), entry("BBB", 300)]);
        assert_eq!(board.insert(entry("CCC", 300)), Some(1));
        let names: Vec<&str> = board.entries().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["BBB", "CCC", "AAA"]);
    }

    #[test]
    fn a_full_leaderboard_keeps_only_the_best() {
        let mut board = Leaderboard::new((1..=10).map(|n| entry("AAA", n * 10)).collect());
        assert!(!board.qualifies(10));
        assert_eq!(board.insert(entry("ZZZ", 5)), None);
        assert_eq!(board.insert(entry("ZZZ", 55)), Some(5));
        assert_eq!(board.entries().len(), PLACES);
        assert_eq!(board.entries().last().unwrap().score, 20);
    }

    #[test]
    fn a_run_without_points_is_not_kept() {
        assert!(!Leaderboard::default().qualifies(0));
    }
}
