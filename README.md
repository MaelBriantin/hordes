# hordes

Survive the night. A survivor-like in the terminal, in half-block pixel art: hold out against the hordes of the night as long as you can, while your weapons strike by themselves and the weather worsens. Built in Rust with [Ratatui](https://ratatui.rs).

## Install

On macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MaelBriantin/hordes/releases/latest/download/hordes-installer.sh | sh
```

On Windows (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/MaelBriantin/hordes/releases/latest/download/hordes-installer.ps1 | iex"
```

Then run `hordes`. To update to the latest version, run `hordes update`. `hordes reset` erases your settings and your leaderboard; `hordes uninstall` removes the game, and asks whether to erase them too.

The game wants a terminal with true colour and a [Nerd Font](https://www.nerdfonts.com) (for instance JetBrains Mono Nerd Font) for the icons of its status line: without one, they show as empty boxes. A terminal with kitty's keyboard protocol (Ghostty, kitty, WezTerm, iTerm2…) knows when a key is let go: the hero stops the moment you do.

## Controls

| Key | Action |
|---|---|
| `←` `↑` `↓` `→` / `WASD` / `ZQSD` | Move (the last key pressed wins) |
| `Enter` | Start, take a card, confirm |
| `P` / `Esc` | Pause: resume, settings, new run, quit |
| `B` `R` `O` `L` | On the title screen: blood, rain, the player's outline, language |
| `R` | After a death: a new run |
| `Ctrl` `C` | Quit at once |

Your weapons strike by themselves. Gather the XP gems, pick a card at each level, and hold out as the weather worsens and the score's multiplier grows.

Every rule, with its numbers, is in [docs/RULEBOOK.md](docs/RULEBOOK.md); what each version brought, in [CHANGELOG.md](CHANGELOG.md).

## Development

```sh
cargo run --release
cargo test
cargo clippy   # the project lints with clippy::pedantic
```

Set `HORDES_DATA_DIR` to keep the settings and the leaderboard somewhere else, to try things without touching your own:

```sh
HORDES_DATA_DIR=/tmp/hordes cargo run
```

In a debug build (`cargo run`), Tab opens a debug console during a run: the next weather, a horde, the dawn, a level-up, a full heal, or nothing hurting the player.

## Licence

All rights reserved. You're free to play the game.
