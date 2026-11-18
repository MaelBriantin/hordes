# Changelog

What each version brought, for the player. The rules themselves, with their numbers, live in [docs/RULEBOOK.md](docs/RULEBOOK.md).

Each version's lines go under New (what didn't exist), Changed (what works otherwise, balance included) and Fixed (bugs), in that order, only those that have lines.

Numbers: a minor version (0.2.0) for new things to play with; a patch (0.1.1) for a fix or a balance change alone.

## Unreleased
Written as the work goes, under the same New, Changed and Fixed; it becomes the next version's section when it is released.

## v0.1.0 · November 18, 2026
Survive the night.

### New
- An endless map drawn from a seed, in pixel art: groves, rocks, grass, all in half blocks of the terminal.
- A hero with a red headband, moving in 4 directions, the last key pressed winning.
- Zombies, bats flying over obstacles, and abominations, coming in hordes that grow by the minute.
- Weapons that strike by themselves: a machete to start with, then a pistol and a machine gun with their magazines and reloads, and drones circling the player.
- XP gems, levels, and a level-up of three cards to pick from: new weapons, their next levels, and passives (Vitality, Swiftness, Magnet, Might, Haste, Flashlight).
- The night: only the player's torch lights the map; rain, and lightning that shows the whole horde at once.
- Weather that worsens every 2 minutes, from a clear night to a tempest: more rain, more lightning, which strikes enemies from the storm on, tougher enemies, a dimmer light.
- An arcade score: points for every kill, times the weather and a combo; a leaderboard of the 10 best runs, three letters typed in on the death screen.
- A title screen and a death screen under the storm, a pause menu.
- Settings kept between runs: blood shown or not, the rain's look, and the language, English or French.
- An installer for macOS, Linux and Windows; `hordes update` installs the latest version, `hordes --version` shows yours.
