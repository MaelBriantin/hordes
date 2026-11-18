# hordes — Rulebook

What every rule of a run does, with its numbers. Rules not in the game yet are marked *(planned)*; numbers still to be set are marked *TBD*.

## Controls

- **Move**: the arrows, WASD or ZQSD, in **4 directions**, no diagonal: of the keys held down, the last one pressed wins (D then W goes up; letting go of W goes right again).
- **Start**: the game opens on its title screen, the run standing still; Enter starts it.
- **Pause**: P or Esc, during the run or a level-up: the run stands still, and a menu offers to resume (P or Esc again), show blood or not, set the rain, change the language, start a new run or quit. ↑↓ choose, Enter confirms, ←→ change a setting.
- **Blood**: red, ketchup, mayo, pesto or off, with B on the title screen or from the pause menu.
- **Rain**: how it shows (full, reduced to half its drops, or off), with R on the title screen or from the pause menu. The weather itself is the same whatever the choice.
- **Language**: English or French, with L on the title screen or from the pause menu. At first, the system's language if the game speaks it (`LANG`), English otherwise.
- These choices are kept for the next runs (`settings.txt` in the game's data folder).
- The title screen and the pause menu show the game's version.
- **Quit**: from the pause menu; Esc on the title screen or the game over; Ctrl+C at any time.
- **The camera** keeps the player in the middle of the screen.
- **Held keys**: with kitty's keyboard protocol (Ghostty, kitty, WezTerm, iTerm2…), a key holds exactly as long as it is held down. Elsewhere, the terminal only repeats presses: a key counts as held 0.5 s after a press and 0.15 s after each repeat, and holding two keys at once may not work.

## The run

- The world moves on **30 times per second**.
- **Announcements**: the run opens on the dusk, told in big letters over the map, with the goal under it (hold out until dawn); each new weather (see Weather) and each horde (see Hordes) is told the same way, a line under it: the night has fallen, the weather worsens, dawn in two minutes. The world stands still for the **2 s** it shows: the clock, the weather and the enemies wait.
- **Victory**: the **dawn**, at **11:00**, after the tempest's two minutes. Over its last minute, the sun rises little by little, the rain thins out to nothing, and lightning no longer strikes. At 11:00, the dawn's word shows over the run, which goes on for **4 s**: the enemies flee the player, the risen sun at their back, and burn one by one, flashing white, worth nothing; none comes, nothing hurts the player. Then the run stops: the screen shows the player standing in the light of the risen sun, a line under the word (congratulations, you survived; this time), the rain over, the score with a **dawn bonus of 50%** of it, and the leaderboard, where a sun marks the runs that saw the dawn.
- **Death**: at 0 HP, the screen shows where the player fell, in the dark but for their torch, a pool of blood (if shown) where they lay, under the storm: the score, the time survived, the level and the kills, and the leaderboard. A run that makes the leaderboard asks for its three letters right there. R starts a new run on a new map.

## The map

- **Endless**, in every direction. A tile shows as 2 × 2 pixels of pixel art.
- Each run draws a new map from a **seed**, chunk by chunk (32 × 32 tiles) as the player nears them: the same place always looks the same.
- **Obstacles** block the way: trees and rocks, each covering **2 × 2 tiles**. They come in clumps of up to **8**: groves (70% of clumps) and heaps of rocks (30%). Together they cover **8%** of each chunk.
- The player starts with no obstacle within **8 tiles**.
- Healing is **rare**: about one chunk in 33 holds a **herb** on its free ground, one in 100 a **campfire** in a clearing (no obstacle of the chunk within 2 tiles). The same place always holds the same.

## Healing

- **Herbs**: a hurt player walking over a herb (within 0.9 tile of its middle) picks it up and gets back **20 HP**. In full health, the herb is left for later. Herbs don't glow: only the torch or a strike of lightning shows them.
- **Campfires**: a hurt player within **2 tiles** of a burning campfire gets back **3 HP a second**. A campfire gives **24 HP** in all (8 seconds of healing), then dies out, leaving its logs. It lights the night around it, **6 tiles** at its brightest, less and less as it gives (never under a tile and a half while it burns).

## Hordes

- A **horde** comes in the middle of each stage of weather, once the night has fallen: at **2:00**, then every **2 minutes** (4:00, 6:00…). It brings **30** enemies, and **15** more for each horde before it.
- It is **announced 3 s before**: told in big letters (see The run), then three red arrows on the edge it comes from, and the map shaking twice.
- Until the storm, it comes as a dense **pack** out of one side (a band 16 tiles wide, 5 deep, just beyond the screen); from the storm on, as a **ring** out of every side, announced on every edge.
- Its enemies: zombies; a bat in ten from the rain on; an abomination in ten in the tempest. Their HP follow the weather, as every enemy's.
- **Wiped out** to the last, a horde **doubles** the points its kills were worth: the bonus shows in gold on top of the map.
- Its zombies show as **ghouls**, pale, in darker rags: a horde stands out from the rest.

## Score

Every run is played for its score, arcade style.

- Every enemy felled is worth **points**: zombie **10**, bat **15**, abomination **100**.
- Times the **weather's multiplier**: ×1 at dusk and on a clear night, ×1.5 in the drizzle, ×2 in the rain, ×3 in the storm, ×5 in the tempest.
- Times the **combo**: every kill within **2 s** of the last adds ×0.1, up to ×2; 2 s without a kill and it is gone. Rounded down.
- The status line shows the score and the multiplier of the moment (the weather's times the combo's). While the run is ahead of the best one kept, its score gleams.
- **Leaderboard**: the 10 best runs, kept in `scores.txt` in the game's data folder. A run that makes it asks for three letters on the death screen (a **new high score** only when it beats the best; otherwise its place), where it shows in its place as they are typed: ↑↓ turn a letter or type it, ←→ move, Enter keeps the run, Esc leaves it off. The leaderboard shows on the title screen too, in a box under its tagline, the best run in gold; the settings are on its last line.

## The night

- Runs start at **dusk**, for a minute: for its first 30 s, the setting sun lights the whole map, warm; over the next 30 s, the night falls, dimmer and bluer, until only the lights below show anything. Before the dawn, the sun rises the other way round, over the tempest's last minute.
- Then runs take place at night. Everything is dimmer and bluer.
- The player carries a **light**: everything within **9 tiles** (2 more per level of Flashlight) is lit, fading into the dark over 6 more tiles. Beyond, the map and the enemies are barely seen (14% of their brightness).
- **Gems, bullets and the machete's swings glow**: they show in the dark.
- **Lightning** (see Weather) lights the whole map at once as it strikes, fading back to the dark in 0.8 s.
- **Blood** (red, ketchup, mayo or pesto, as the settings say; or none): a stain where an enemy falls (bigger for an abomination) and where the player is hit, fading away in **30 s**.

## Weather

The weather worsens as the run goes on, a stage every **2 minutes**, as the difficulty does, after the minute of dusk (see The night). The status line shows it, with a gauge to the next stage. The rain thickens over the last quarter of a stage, into the next one's.

| From | Weather | Rain | Lightning | Light | Enemies appearing |
|---|---|---|---|---|---|
| 0:00 | Dusk | none | none | — | — |
| 1:00 | Clear night | none | none | — | — |
| 3:00 | Drizzle | 4 drops per 1000 pixels | every 8 s | — | — |
| 5:00 | Rain | 8 | every 5 s | −1 tile | +10% HP |
| 7:00 | Storm | 14 | every 3 s, **hurting** | −2 tiles | +25% HP |
| 9:00 | Tempest | 20 | every 2 s, **hurting** | −3 tiles | +50% HP, +10% speed |
| 11:00 | Dawn | — | — | — | the run is won (see The run) |

- A strike falls somewhere the player sees, at a moment of its own: between half and one and a half times the weather's wait.
- From the storm on, a strike takes **30 HP** from every enemy within **2.5 tiles** of where it falls (a ring shows it). It never hurts the player.
- The light never falls under **4 tiles**.

## The player

| Stat | Value |
|---|---|
| Speed | 6 tiles/s (+8% per level of Swiftness) |
| Body | 0.9 × 0.9 tile, smaller than the sprite (2 × 2 tiles): brushing past an obstacle is forgiven |
| HP | 100 (+20 per level of Vitality); the HP bar blinks under 30% |
| Invincibility after a hit | 0.5 s (the player blinks) |

## Enemies

| Enemy | HP | XP | Speed | Damage | Body | Moves |
|---|---|---|---|---|---|---|
| Zombie | 10 | 1 (3 times in 4) | 2.5 tiles/s | 5 | 0.9 tile | Straight at the player |
| Bat | 5 | 1 (3 times in 4) | 5 tiles/s | 3 | 0.7 tile | In a zigzag, **flying over obstacles** |
| Abomination | 40 | 5 (always) | 2 tiles/s | 15 | 1.6 tile | Straight at the player |

- Every enemy is slower than the player: they can always be outrun.
- **Obstacles** stop zombies and abominations, which slide along them; bats fly over.
- **Touching** the player costs them the damage of the hardest-hitting enemy touching them; then nothing hurts them for 0.5 s.
- Enemies that overlap **push each other apart**; one touching the player stops there.
- A hit enemy **flashes white**. At 0 HP it falls, and may leave an **XP gem** worth its XP: zombies and bats 3 times in 4, abominations always.

## Waves

- Enemies appear just beyond what the player sees (3 tiles past the edge of the screen), never on an obstacle (bats excepted).
- How many, and which kinds, follow the weather: each stage starts at its own rate, and over the stage the rate grows little by little to the next one's.

| Weather | Enemies a second | Zombie / bat / abomination, by weight |
|---|---|---|
| Dusk | 1 | 10 / 0 / 0 |
| Clear night | 2 | 9 / 1 / 0 |
| Drizzle | 4 | 8 / 1 / 1 |
| Rain | 6 | 7 / 2 / 1 |
| Storm | 8 | 6 / 2 / 2 |
| Tempest | 10 | 5 / 2 / 3 |

- At most **300** enemies at once (600 while a horde is about).
- An enemy left far behind (twice as far as the edge of the screen) appears again just beyond the side the player faces.

## Weapons

They strike by themselves. Up to **4 weapons**, each up to **level 5**. The run starts with the **machete**; the others come from level-ups. Might raises their damage and Haste shortens their cooldowns: the machete's swings, the guns' reloads, the drones' rest (see Passives). Damage is rounded down.

### Machete

A swing in an arc before the player, the way they face, hitting **every enemy in it** for **12 damage**: within reach of the player (plus the enemy's half width) and within the arc. It swings every **1.2 s**, once an enemy is within its reach, whichever way. A swing may hold several **slashes**, 0.2 s apart, each hitting anew: ahead, then **behind**, then ahead again.

| Level | Swing |
|---|---|
| 1 | a slash in a 90° arc, reaching 2.5 tiles |
| 2 | reaching 3.5 tiles |
| 3 | 2 slashes: ahead, then behind |
| 4 | a 180° arc |
| 5 | 3 slashes: ahead, behind, ahead |

### Guns

A gun shoots at the **nearest enemies within its range**, a bullet per enemy, while its **magazine** holds rounds; empty, it **reloads**, then shoots again at once. The reserve of rounds is endless. A bullet flies over obstacles and stops at the first enemy it hits, unless it **pierces**: then it goes through that many enemies first, hitting each once. Over the top left of the map, under each gun's icon, a bar shows its rounds left, filling back in grey as it reloads.

| Level | Pistol | Machine gun |
|---|---|---|
| 1 | 6 rounds, a shot every 0.5 s, 10 damage, a 2-second reload, range 12, 20 tiles/s | 20 rounds, a shot every 0.1 s, 4 damage, a 3-second reload, range 10, 22 tiles/s |
| 2 | 9 rounds | 30 rounds |
| 3 | a 1.5-second reload | 6 damage |
| 4 | 2 bullets a shot, at the 2 nearest | a 2-second reload |
| 5 | 12 rounds, 15 damage | bullets pierce 1 enemy |

### Drones

Drones circling the player **2.5 tiles** away, a turn every **2 s**. A drone hits the first enemy it runs into: **8 damage**, and a push of **half a tile** away from the player. The blow **breaks** it: it keeps circling, grey and harmless, and repairs in **2 s** (less with Haste).

**Levels**: 2: +1 drone · 3: 12 damage · 4: +1 drone (3) · 5: 3.5 tiles away, a turn every 1.5 s. The drones are evenly spaced on their circle.

### To come

| Weapon | What it does |
|---|---|
| Gas *(planned)* | A toxic cloud around the player. |
| Molotov *(planned)* | A puddle of fire under a group of enemies, hurting all that stay in it for a few seconds. |
| Airstrike *(planned)* | Strikes falling at random around the player, each marked on the ground just before it blows. |
| Explosive rounds *(planned)* | Felled enemies blow up, hurting their neighbours: chain reactions. |

## Passives

Up to **4 passives**, each up to **level 5**.

| Passive | Per level |
|---|---|
| Vitality | +20 max HP, and heals as much |
| Swiftness | +8% speed |
| Magnet | Gems fly in from 1 tile further (2 tiles at first) |
| Might | +10% damage of every weapon |
| Haste | −8% to every weapon's cooldown, the drones' repair included |
| Flashlight | +2 tiles of light around the player (9 at first) |
| Compass | A red arrow on the edge of the screen, white on a pulsing red cell, points to the nearest healing (a herb, a burning campfire), while it is off screen: within 150 tiles, 50 more per level |

## Experience

- Fallen enemies may leave an XP gem (see Enemies). A gem covers one tile. A gem within **2 tiles** (more with Magnet) flies to the player (12 tiles/s) and keeps on until picked up.
- At most **500 gems** lie around: past that, the oldest go.
- The next level takes **5 + 3 × (level − 1)** XP: 5 from level 1, 8 from level 2, 11 from level 3. XP beyond it carries over.
- **Level-up**: the game pauses and offers **3 different choices** at random among what can still be picked: a new weapon or passive (while there is room), or the next level of one owned. At dusk, Flashlight is never offered: there is still light enough. With nothing left to pick, the one choice is **Heal 20 HP**. Several levels at once offer their choices one after the other.
- **Choosing**: ←/→ (or Q/A and D) then Enter (or Space), or 1, 2, 3 straight away.
