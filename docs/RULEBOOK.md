# hordes — Rulebook

What every rule of a run does, with its numbers. Rules not in the game yet are marked *(planned)*; numbers still to be set are marked *TBD*.

## Controls

- **Move**: the arrows, WASD or ZQSD, in **4 directions**, no diagonal: of the keys held down, the last one pressed wins (D then W goes up; letting go of W goes right again).
- **Start**: the game opens on its title screen; Enter starts a run, in the mode chosen with ←→, A, Q or D (see Modes).
- **Keys**: ? lists every key of a run, the run standing still; ?, Esc or Enter leave the list.
- **Gear**: I shows or hides the gear over the top left of the map; the choice is kept.
- **Torch**: Space or F, once it has lit up of itself, at 0:26 (see The night).
- **Pause**: P or Esc, during the run or a level-up: the run stands still, and a menu offers to resume (P or Esc again), open the options, start a new run, go back to the title screen or quit. ↑↓ choose, Enter confirms.
- **Options**: O or Esc on the title screen (where they hold the way to quit), or from the pause menu. ↑↓ choose, ←→ or Enter change a setting, Esc goes back.
  - **Blood**: red, ketchup, mayo, pesto or off.
  - **Rain**: how it shows (full, reduced to half its drops, or off). The weather itself is the same whatever the choice.
  - **Outline**: a line of pixels around the player, to see them at once in the crowd: off, white, yellow or red.
  - **Announcements**: how a new weather and a horde are told: big, with a pause (in big letters, the world standing still 2 s); big, no pause (in big letters 2 s, the run going on); or a banner (on the top row of the map 3 s, the run going on). The opening is always told big, with a pause.
  - **Gear**: shown or hidden over the top left of the map (I in a run does the same). The opening and the hordes are always told in big letters.
  - **Dusk**: played or skipped: skipped, a run opens on the night, the torch lit.
  - **Language**: English or French. At first, the system's language if the game speaks it (`LANG`), English otherwise.
- These choices are kept for the next runs (`settings.txt` in the game's data folder).
- The title screen and the pause menu show the game's version.
- **Quit**: from the pause menu, or the title screen's options; Esc on the game over; Ctrl+C at any time.
- **The camera** keeps the player in the middle of the screen.
- **Held keys**: with kitty's keyboard protocol (Ghostty, kitty, WezTerm, iTerm2…), a key holds exactly as long as it is held down. Elsewhere, the terminal only repeats presses: a key counts as held 0.5 s after a press and 0.15 s after each repeat, and holding two keys at once may not work.

## Modes

- **Standard**: from dusk to dawn, as below; no score, no leaderboard: the death screen tells the time survived, the level and the kills. *(planned: trophies for goals reached.)*
- **Arcade**: endless, for the score. The stages of weather last **3 minutes** and go up to the tempest and back down to the night, again and again (night, drizzle, rain, storm, tempest, storm, rain, drizzle, night…), with no dawn. The run grows harder with the time, **7% a minute**, compounded (×2 at 10 minutes, ×3.9 at 20): as many more enemies appear (1.5 a second at first), as many more HP they have, and the score's weather multiplier is that pressure. The leaderboard is the arcade's.

## The run

- The world moves on **30 times per second**.
- **Announcements**: the run opens on the dusk, told in big letters over the map, with the goal under it (hold out until dawn); each new weather (see Weather) and each horde (see Hordes) is told the same way, a line under it: the night has fallen, the weather worsens, dawn in two minutes. The world stands still for the **2 s** it shows: the clock, the weather and the enemies wait.
- **Victory**, in a standard run: the **dawn**, at **10:30**, after the tempest's two minutes. Over its last minute, the sun rises little by little, the rain thins out to nothing, and lightning no longer strikes. At 10:30, the dawn's word shows over the run, which goes on for **4 s**: the enemies flee the player, the risen sun at their back, and burn one by one, flashing white, worth nothing; none comes, nothing hurts the player. Then the run stops: the screen shows the player standing in the light of the risen sun, a line under the word (congratulations, you survived; this time), the rain over, the time, the level and the kills.
- **Death**: at 0 HP, the screen shows where the player fell, in the dark but for their torch, a pool of blood (if shown) where they lay, under the storm: the time survived, the level and the kills; in arcade, the score and the leaderboard too, a run that makes it asking for its three letters right there. R starts a new run on a new map.

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

- A **horde** comes in the middle of each stage of weather, once the night has fallen: at **1:30**, then every **2 minutes** (3:30, 5:30…). It brings **30** enemies, and **10** more for each horde before it.
- It is **announced 3 s before**: told in big letters (see The run), then a blinking band of arrows, white on red, on the edge it comes from.
- Until the storm, it comes as a dense **pack** out of one side (a band 16 tiles wide, 5 deep, just beyond the screen); from the storm on, as a **ring** out of every side, announced on every edge.
- Its enemies: zombies; a Shadow in ten from the rain on; an abomination in ten in the tempest. Their HP follow the weather, as every enemy's.
- While some of a horde still stands, a count shows on the top of the map, white on red: how many are left.
- **Wiped out** to the last, a horde **doubles** the points its kills were worth: it is told in big gold letters, the bonus under them, the run going on.
- Its zombies show as **ghouls**, pale, in darker rags: a horde stands out from the rest.

## Score

In arcade, every run is played for its score.

- Every enemy felled is worth **points**: zombie **10**, Shadow **30**, abomination **100**.
- Times the **weather's multiplier**: ×1 at dusk and in the night, ×1.5 in the drizzle, ×2 in the rain, ×3 in the storm, ×5 in the tempest.
- Times the **combo**: every kill within **2 s** of the last adds ×0.1, up to ×2; 2 s without a kill and it is gone. Rounded down.
- The status line shows the score and the multiplier of the moment (the weather's times the combo's) on a gold badge, the score's thousands apart. While the run is ahead of the best one kept, its score gleams.
- **Leaderboard**: the 10 best runs, kept in `scores.txt` in the game's data folder. A run that makes it asks for three letters on the death screen (a **new high score** only when it beats the best; otherwise its place), where it shows in its place as they are typed: ↑↓ turn a letter or type it, ←→ move, Enter keeps the run, Esc leaves it off. The leaderboard shows on the title screen too, in a box under its tagline, the best run in gold; the settings are on its last line.

## The night

- Runs start at **dusk**, for half a minute, just enough to look around: for its first 24 s, the setting sun lights the whole map, warm, less and less, down to half its light; then the night falls all of a sudden, in **6 s**, dimmer and bluer, until only the lights below show anything. Before the dawn, the sun rises the other way round, over the tempest's last minute.
- Then runs take place at night. Everything is dimmer and bluer.
- The player carries a **light**, a torch, out at dusk: it lights up of itself at **0:26**, as the night falls, flickering twice before it holds. **Space** or **F** puts it out, or lights it again, flickering the same.
- **Its battery**: a full one keeps the torch lit **2 minutes**; out, it charges back from empty in **1 minute** (Batteries: both 25% better per level). Nearly empty, the torch flickers, then dies out; it lights again only from **10%**. The status line shows the torch, on or out (a bulb), and its battery as a gauge, its percent in the middle, yellow while the torch is on, grey-blue while it charges, red under 20%. Lit, everything within **9 tiles** (2 more per level of Flashlight) is lit, fading into the dark over 6 more tiles. Beyond, the map and the enemies are barely seen (14% of their brightness).
- **The torch out**, the player is barely seen, as all the rest; the gems, the campfires and the lightning still show. The enemies no longer see them from afar: an enemy finds the player within **6 tiles**, and loses them again past **12**. One that hasn't found them goes for the nearest burning campfire within **20 tiles**, or wanders, at half its speed. In the dark, the **guns hold their fire**: nothing to aim at (the machete and the drones still strike what comes near). A **strike of lightning** gives the player away to every enemy on screen. The hordes always find the player. Lit, or by daylight, every enemy sees the player.
- **Shadows** go the other way round, at night: they hunt the player in the dark, wherever they are, their eyes shining red; the torch lit, they keep away from it.
- **Gems, bullets and the machete's swings glow**: they show in the dark. A gem casts a small blue glow on the ground around it, two tiles or so, and lights what passes by it, an enemy, out of the dark.
- **Lightning** (see Weather) lights the whole map at once as it strikes, fading back to the dark in 0.8 s.
- **Blood** (red, ketchup, mayo or pesto, as the settings say; or none): a stain where an enemy falls (bigger for an abomination) and where the player is hit, fading away in **30 s**. A hit player also spurts a few drops every way, for a third of a second.
- **A blow** to the player flushes the edges of the screen red for half a second, whatever the blood setting.

## Weather

The weather worsens as the run goes on, a stage every **2 minutes**, as the difficulty does, after the half minute of dusk (see The night). The status line shows every weather's icon in a row, and the dawn's last, the weather now in its colour with its name after the row, those gone put out in grey, those to come barely there; the next one comes out of the dark as the stage goes. The rain thickens over the last quarter of a stage, into the next one's.

| From | Weather | Rain | Lightning | Light | Enemies appearing |
|---|---|---|---|---|---|
| 0:00 | Dusk | none | none | — | — |
| 0:30 | Night | none | none | — | — |
| 2:30 | Drizzle | 4 drops per 1000 pixels | none | — | — |
| 4:30 | Rain | 8 | none | −1 tile | +10% HP |
| 6:30 | Storm | 14 | every 3 s, **hurting** | −2 tiles | +25% HP |
| 8:30 | Tempest | 20 | every 2 s, **hurting** | −3 tiles | +50% HP, +10% speed |
| 10:30 | Dawn | — | — | — | the run is won (see The run) |

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
| Shadow | 25 | 2 (3 times in 4) | 5.7 tiles/s | 8 | 0.7 tile | Straight, **through obstacles**; hunched, barely seen in the dark but for its red eyes |
| Abomination | 40 | 5 (always) | 2 tiles/s | 15 | 1.6 tile | Straight at the player |

- Every enemy is slower than the player: they can always be outrun.
- **Obstacles** stop zombies and abominations, which slide along them; Shadows pass through.
- **Touching** the player costs them the damage of the hardest-hitting enemy touching them; then nothing hurts them for 0.5 s.
- Enemies that overlap **push each other apart**; one touching the player stops there.
- A hit enemy **flashes white**. At 0 HP it falls, and may leave an **XP gem** worth its XP: zombies and Shadows 3 times in 4, abominations always.

## Waves

- Enemies appear just beyond what the player sees (3 tiles past the edge of the screen), never on an obstacle (Shadows excepted).
- How many, and which kinds, follow the weather: each stage starts at its own rate, and over the stage the rate grows little by little to the next one's.

| Weather | Enemies a second | Zombie / Shadow / abomination, by weight |
|---|---|---|
| Dusk | 1 | 10 / 0 / 0 |
| Night | 1.5 | 9 / 1 / 0 |
| Drizzle | 3 | 8 / 1 / 1 |
| Rain | 4.5 | 7 / 2 / 1 |
| Storm | 6 | 6 / 2 / 2 |
| Tempest | 7.5 | 5 / 2 / 3 |

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

A gun shoots at the **nearest enemies within its range**, a bullet per enemy, while its **magazine** holds rounds; empty, it **reloads**, then shoots again at once. The reserve of rounds is endless. A pistol's bullet is a white pixel; a machine gun's, a thin tracer, its trail fading behind it. A bullet flies over obstacles and stops at the first enemy it hits, unless it **pierces**: then it goes through that many enemies first, hitting each once. Over the top left of the map, under each gun's icon, a bar shows its rounds left, filling back in grey as it reloads.

| Level | Pistol | Machine gun |
|---|---|---|
| 1 | 6 rounds, a shot every 0.5 s, 10 damage, a 2-second reload, range 12, 20 tiles/s | 20 rounds, a shot every 0.17 s, 3 damage, a 3-second reload, range 10, 22 tiles/s |
| 2 | 9 rounds | 30 rounds |
| 3 | a 1.5-second reload | 5 damage |
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
| Compass | A red arrow on the edge of the screen, white on pulsing red cells, a healing cross beside it, points to the nearest healing (a herb, a burning campfire), while it is off screen: within 150 tiles, 50 more per level |
| Batteries | +25% to the torch's battery: it lasts longer, and charges faster |
| Loot | +10 points of percent for an enemy to leave a gem (an abomination always does) |
| Wisdom | +10% XP from the gems (tenths of a point kept until they make a whole one) |

## Legendaries

- A level-up offers a **legendary** **5%** of the time, while one is left to find: on a **golden card**, in place of its last choice. Each level-up without one raises the chance by a point (6%, 7%…), back to 5% once one is offered.
- A legendary has **one level**, and takes no room among the passives; once found, it is never offered again. It shows in the gear with a golden bar.

| Legendary | What it does |
|---|---|
| Illumination | **+1 XP every second**, of itself |

## Experience

- Fallen enemies may leave an XP gem (see Enemies). A gem covers one tile. A gem within **2 tiles** (more with Magnet) flies to the player (12 tiles/s) and keeps on until picked up.
- A gem shows by its worth: **blue** for a point, **green** for 2 to 4 (a Shadow's), **red** for 5 and more (an abomination's). Each glows its colour around it in the dark.
- A fallen **Shadow** also leaves an **energy cell**, yellow, beside its gem: it flies in as gems do, and gives the torch's battery **15%**.
- At most **500 gems** lie around: past that, the oldest go.
- The next level takes **5 + 3 × (level − 1)** XP: 5 from level 1, 8 from level 2, 11 from level 3. XP beyond it carries over.
- **Rarity**: each card is **common** (grey frame) or **rare** (blue frame, its word on the bottom border), the legendaries apart (see Legendaries). Common: every weapon, Vitality, Swiftness, Magnet, Flashlight, Loot, the heal. Rare: Might, Haste, Compass, Wisdom, Batteries. A weapon or a passive keeps its rarity at every level.
- **Level-up**: the game pauses and offers **3 different choices** at random among what can still be picked, each a rare one **25%** of the time (a common one otherwise, or the other if none is left of its rarity): a new weapon or passive (while there is room), or the next level of one owned. At dusk, neither Flashlight nor Batteries is offered: there is still light enough. With nothing left to pick, the one choice is **Heal 20 HP**. Several levels at once offer their choices one after the other.
- **Choosing**: ←/→ (or Q/A and D) then Enter (or Space), or 1, 2, 3 straight away.
