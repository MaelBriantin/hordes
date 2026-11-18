# Changelog

What each version brought, for the player. The rules themselves, with their numbers, live in [docs/RULEBOOK.md](docs/RULEBOOK.md).

Each version's lines go under New (what didn't exist), Changed (what works otherwise, balance included) and Fixed (bugs), in that order, only those that have lines.

Numbers: a minor version (0.2.0) for new things to play with; a patch (0.1.1) for a fix or a balance change alone.

## Unreleased
Written as the work goes, under the same New, Changed and Fixed; it becomes the next version's section when it is released.

### New
- Runs start at dusk: the setting sun lights the whole map for a minute, then the night falls over the next one. The weather worsens from there, 2 minutes later than before.
- Hordes: every 90 seconds, a wave of enemies comes all at once, announced a moment before; from the storm on, it closes in from every side. Its zombies are pale ghouls. Wiped out, it doubles its points.
- Red herbs on the ground, which give back 20 HP to a hurt player walking over them.
- Campfires, which heal 3 HP a second to a player staying by them: their light shrinks as they give, until they die out.
- Herbs and campfires are rare: the map holds about one herb for 33 chunks, one campfire for 100. Only the torch, a strike of lightning or a campfire's own light shows them.
- A new passive, Compass: a red arrow on the edge of the screen points to the nearest healing.
- `hordes reset` erases your settings and your leaderboard; `hordes uninstall` removes the game, and asks whether to erase them too.

### Changed
- The status line wears icons (Nerd Font): HP, weather, time, score; the time and the score stand out in bold. 
- Over the top left of the map, on a dark veil, the weapons and passives picked show as the icons of their cards, each with its level in pixels under it; the guns show their rounds there too, filling back as they reload.
- The XP bar runs along the bottom of the screen during a run, the level at its start.
- The game draws its own dark background and text, whatever the terminal's theme: a light one no longer washes it out.
- The night is a little less dark beyond the torch: 14% of the light, rather than 10%.
- A drone breaks on its blow, and repairs in 2 seconds: drones were too strong.
- Blood comes red, as ketchup, mayo or pesto, or not at all: B on the title screen, or the pause menu.
- The HP bar is red, and blinks when HP run low.
- The score gleams in the status line while the run is ahead of the best one.

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
