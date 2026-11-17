# hordes — Rulebook

What every rule of a run does, with its numbers. Rules not in the game yet are marked *(planned)*; numbers still to be set are marked *TBD*.

## Controls

- **Move**: the arrows, WASD or ZQSD, in **4 directions**, no diagonal: of the keys held down, the last one pressed wins (D then W goes up; letting go of W goes right again).
- **Quit**: Esc (or Ctrl+C).
- **The camera** keeps the player in the middle of the screen.
- **Held keys**: with kitty's keyboard protocol (Ghostty, kitty, WezTerm, iTerm2…), a key holds exactly as long as it is held down. Elsewhere, the terminal only repeats presses: a key counts as held 0.5 s after a press and 0.15 s after each repeat, and holding two keys at once may not work.

## The run

- The world moves on **30 times per second**.
- **Victory**: survive **10 minutes**. *(planned)*
- **Game over**: at 0 HP. Shows the time survived, the level and the kills; R starts a new run on a new map.
- **Pause**: P. *(planned)*

## The map

- **Endless**, in every direction. A tile shows as 2 × 2 pixels of pixel art.
- Each run draws a new map from a **seed**, chunk by chunk (32 × 32 tiles) as the player nears them: the same place always looks the same.
- **Obstacles** block the way: trees and rocks, each covering **2 × 2 tiles**. They come in clumps of up to **8**: groves (70% of clumps) and heaps of rocks (30%). Together they cover **8%** of each chunk.
- The player starts with no obstacle within **8 tiles**.

## The player

| Stat | Value |
|---|---|
| Speed | 6 tiles/s (+8% per level of Swiftness) |
| Body | 0.9 × 0.9 tile, smaller than the sprite (2 × 2 tiles): brushing past an obstacle is forgiven |
| HP | 100 (+20 per level of Vitality) |
| Invincibility after a hit | 0.5 s (the player blinks) |

## Enemies

| Enemy | HP | XP | Speed | Damage | Body | Moves | From |
|---|---|---|---|---|---|---|---|
| Zombie | 10 | 1 (3 times in 4) | 2.5 tiles/s | 5 | 0.9 tile | Straight at the player | 0:00 |
| Bat | 5 | 1 (3 times in 4) | 5 tiles/s | 3 | 0.7 tile | In a zigzag, **flying over obstacles** | 1:00 |
| Abomination | 40 | 5 (always) | 2 tiles/s | 15 | 1.6 tile | Straight at the player | 3:00 |

- Every enemy is slower than the player: they can always be outrun.
- **Obstacles** stop zombies and abominations, which slide along them; bats fly over.
- **Touching** the player costs them the damage of the hardest-hitting enemy touching them; then nothing hurts them for 0.5 s.
- Enemies that overlap **push each other apart**; one touching the player stops there.
- A hit enemy **flashes white**. At 0 HP it falls, and may leave an **XP gem** worth its XP: zombies and bats 3 times in 4, abominations always.

## Waves

- Enemies appear just beyond what the player sees (3 tiles past the edge of the screen), never on an obstacle (bats excepted).
- **1 per second**, and 1 more per second each minute: 2 from 1:00, 10 from 9:00.
- Which kind, among those arrived, by weight: zombie **6**, bat **3**, abomination **1**.
- At most **300** enemies at once.
- An enemy left far behind (twice as far as the edge of the screen) appears again just beyond the side the player faces.

## Weapons

They fire by themselves. Up to **4 weapons**, each up to **level 5**. The run starts with Bolt. Might raises their damage and Haste shortens their cooldowns (see Passives); damage is rounded down.

| Weapon | What it does |
|---|---|
| Bolt | Every **2 s**, a spark at the nearest enemy within **12 tiles** (none in range: it holds fire). The spark flies **15 tiles/s** over obstacles, up to 12 tiles, and stops at the first enemy it hits: **10 damage**. Each spark of a shot goes for a different enemy, the nearest first. |

**Bolt's levels**: 2: +1 spark per shot · 3: every 1.5 s · 4: +1 spark per shot (3) · 5: 20 damage.

**Orb's levels**: 2: +1 orb · 3: 12 damage · 4: +1 orb (3) · 5: 3.5 tiles away, a turn every 1.5 s. The orbs are evenly spaced on their circle.

| Weapon | What it does |
|---|---|
| Blade *(planned)* | An arc in front of the player. |
| Orb | Orbs circling the player **2.5 tiles** away, a turn every **2 s**. An orb hits every enemy it passes through: **8 damage**, and a push of **half a tile** away from the player. An enemy an orb has hit can't be hit by an orb again for **0.5 s** (shorter with Haste). |
| Nova *(planned)* | An aura that hurts around the player. |
| Zone *(planned)* | A puddle under a group of enemies, hurting all that stay in it for a few seconds. |
| Bombardment *(planned)* | Strikes falling at random around the player, each marked on the ground just before it blows. |
| Volatile *(planned)* | Felled enemies blow up, hurting their neighbours: chain reactions. |

## Passives

Up to **4 passives**, each up to **level 5**.

| Passive | Per level |
|---|---|
| Vitality | +20 max HP, and heals as much |
| Swiftness | +8% speed |
| Magnet | Gems fly in from 1 tile further (2 tiles at first) |
| Might | +10% damage of every weapon |
| Haste | −8% to every weapon's cooldown |

## Experience

- Fallen enemies may leave an XP gem (see Enemies). A gem covers one tile. A gem within **2 tiles** (more with Magnet) flies to the player (12 tiles/s) and keeps on until picked up.
- At most **500 gems** lie around: past that, the oldest go.
- The next level takes **5 + 3 × (level − 1)** XP: 5 from level 1, 8 from level 2, 11 from level 3. XP beyond it carries over.
- **Level-up**: the game pauses and offers **3 different choices** at random among what can still be picked: a new weapon or passive (while there is room), or the next level of one owned. With nothing left to pick, the one choice is **Heal 20 HP**. Several levels at once offer their choices one after the other.
- **Choosing**: ←/→ (or Q/A and D) then Enter (or Space), or 1, 2, 3 straight away.
