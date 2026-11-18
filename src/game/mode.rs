//! The two ways to play a run: the standard one, from dusk to dawn, with no score; and the arcade
//! one, endless, the weather going round and round, harder and harder, for the score.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Standard,
    Arcade,
}

impl Mode {
    /// The other mode.
    pub fn other(self) -> Self {
        match self {
            Mode::Standard => Mode::Arcade,
            Mode::Arcade => Mode::Standard,
        }
    }

    /// Its name in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Mode::Standard => "standard",
            Mode::Arcade => "arcade",
        }
    }
}
