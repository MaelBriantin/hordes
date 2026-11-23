# hordes — Rulebook

What every rule of a run does, with its numbers. Rules not in the game yet are marked *(planned)*; numbers still to be set are marked *TBD*.

## Controls

- **Move**: the arrows, WASD or ZQSD, in **4 directions**, no diagonal: of the keys held down, the last one pressed wins (D then W goes up; letting go of W goes right again).
- **Start**: the game opens on its title screen; Enter starts a run, in the mode chosen with Tab (see Modes). Under the game's name, the mode as its subtitle, in small pixel letters: **Survive the night** in the clear night's grey, as its word is told, or **Endless Night** in gold, a gleam sweeping across it now and then. At the bottom left, the key and the other mode it changes to. The game's name, the mode and the prompt stand in the middle of the screen with what shows under them (the arcade's leaderboard, the collection or the skins once opened): they slide at a steady pace (a row and a half a frame, so a short way at once) whenever what shows under them changes (the collection or the skins opening or closing, Tab changing the mode and so showing the arcade's leaderboard or not), what shows under them appearing once they are in place.
- **Keys**: ? lists every key of a run, the run standing still; ?, Esc or Enter leave the list.
- **Gear**: I shows or hides the gear over the top left of the map, the choice kept: each card's icon in a frame of its rarity's colour, dimmed (grey, blue, gold), a pixel of room between, a group's frames side by side, its gauge under its frame, as wide as it, in groups (the hero's own weapon and action, the other weapons, the passives, the legendaries) a little apart, each in the order picked, its level written in its frame's bottom right corner, inside it, on a dark badge (MAX in gold at the last); right under a gun's, its rounds or its reload; under the Dash card's, its dashes; under the Barricade card's, the next barricade coming.
- **Torch**: F, once the night has fallen, at 0:26 (see The night).
- **Action**: Space or Enter plays the hero's own action, once it has come at the run's level 3: the survivor's barricade, the ninja's dash, the riot officer's shield ram, the demolisher's C4, the medic's first-aid kit, the exterminator's wall of fire, the ghost's smoke bomb, the veteran's gatling burst, the special agent's vault (see The player).
- **Pause**: P or Esc, during the run or a level-up: the run stands still, and a menu offers to resume (P or Esc again), open the options, start a new run, go back to the title screen or quit. ↑↓ choose, Enter confirms. Beside the menu, where the screen is wide enough, the **gear**: the hero from the front, their outfit's name, their perk and their action (its level, ready or charging), then each card taken, in groups a row apart, each in the order first picked (their own weapon and action; the other weapons; the passives; the legendaries), its icon in a frame of its rarity's colour, its level in the frame's bottom right corner as in the gear in play (MAX in gold at the last), its name, and what it brings: what it does, then what each level after the first added; a line each, without the icons, when the screen is too low for them.
- **Options**: O or Esc on the title screen (where they hold the way to quit), or from the pause menu. ↑↓ choose, ←→ or Enter change a setting, Esc goes back.
- **Updates**: as the game starts, it asks its releases, out of the way, for a newer version (once, a few seconds at most; not a word without a network). One out, the title screen's bottom right says so in gold before the version, `[U] Version 0.11.0 is out: update`: **U** quits the game and installs it, as `hordes update` does, for a game installed by its installer. The options' **Check for updates** (yes at first) turns the question off.
  - **Blood**: red, ketchup, mayo, pesto or off.
  - **Rain**: how it shows (full, reduced to half its drops, or off). The weather itself is the same whatever the choice.
  - **Arrow**: a small arrow over the player's head, bouncing, to see them at once in the crowd: off, white, yellow or red. Whatever the setting, **E** shows the arrow for **2 s** (white, if off): the player found again in the crowd.
  - **Announcements**: how a new weather and a horde are told: big, with a pause (in big letters, the world standing still 2 s); big, no pause (in big letters 2 s, the run going on); or a banner (on the top row of the map 3 s, the run going on). Big, no pause, at first. The opening is always told big, with a pause.
  - **Gear**: hidden, as icons, or as a list over the top left of the map (I in a run shows or hides it); hidden at first. As a list, a line a card: its name, its level (MAX in gold at the last), its gauge if it has one. A gauge under each weapon and the Dash: a gun's rounds, then its reload filling from empty; the melee weapons, the Molotov and the drones (one broken) filling between two blows, full once ready; the dashes ready, then coming back. The opening and the hordes are always told in big letters.
  - **HP over the hero**: shown or hidden: the player's HP in a small bar over their head (over the arrow, when it shows), blinking under **30%** of their HP.
  - **Coloured ground** (experimental): off or on: on, the bare ground is dark grass speckled with earth, lit as the rest, rather than the screen's dark. Off at first: it redraws most of the screen at every step, for a modern, fast terminal (Ghostty, kitty, WezTerm…).
  - **Erase the progress**, from the title screen's options alone: Enter asks to confirm, Enter again erases the achievements, their counts and so the collection (not the settings nor the leaderboard); any other key keeps them.
  - **Language**: English or French. At first, the system's language if the game speaks it (`LANG`), English otherwise.
- These choices are kept for the next runs (`settings.txt` in the game's data folder).
- The title screen and the pause menu show the game's version.
- **Quit**: from the pause menu, or the title screen's options; Esc on the game over; Ctrl+C at any time.
- **The camera** keeps the player in the middle of the screen.
- **Held keys**: with kitty's keyboard protocol (Ghostty, kitty, WezTerm, iTerm2…), a key holds exactly as long as it is held down. Elsewhere, the terminal only repeats presses: a key counts as held 0.5 s after a press and 0.15 s after each repeat, and holding two keys at once may not work.

## Modes

- **Standard**: from dusk to dawn, as below; no score, no leaderboard: the death screen tells the time survived, the level and the kills. The only mode at first.
- **Arcade**, shown as **Endless Night**: unlocked by an achievement, opening on the night, the dusk skipped, the torch lit (a standard run always opens on the dusk); the dawn reached in a standard run (see Progress); until then, no Tab on the title screen, the standard alone. Endless, for the score. The stages of weather last **3 minutes** and go up to the tempest and back down to the night, again and again (night, drizzle, rain, storm, tempest, storm, rain, drizzle, night…), with no dawn. The run grows harder with the time, **7% a minute**, compounded (×2 at 10 minutes, ×3.9 at 20): as many more enemies appear (1.5 a second at first), as many more HP they have, and the score's weather multiplier is that pressure. The leaderboard is the arcade's.

## The run

- The world moves on **30 times per second**.
- **Announcements**: the run opens on the dusk, told in big letters over the map, with the goal under it (hold out until dawn); each new weather (see Weather) and each horde (see Hordes) is told the same way, a line under it: the night has fallen, the weather worsens, dawn in two minutes. The line is in white capitals, as the dawn's on its screen. The world stands still for the **2 s** it shows: the clock, the weather and the enemies wait.
- **Victory**, in a standard run: the **dawn**, at **10:30**, after the tempest's two minutes. Over its last minute, the sun rises little by little, the rain thins out to nothing, and lightning no longer strikes. At 10:30, the dawn's word shows over the run, which goes on for **4 s**: the enemies flee the player, the risen sun at their back, and burn one by one, flashing white, worth nothing; none comes, nothing hurts the player. Then the run stops: the screen shows the player standing in the light of the risen sun, a line under the word (congratulations, you survived; this time), the rain over, the time, the level and the kills.
- **Death**: at 0 HP, the screen shows where the player fell, in the dark but for their torch, a pool of blood (if shown) where they lay, under the storm: the time survived, the level and the kills; in arcade, the score and the leaderboard too, a run that makes it asking for its three letters right there. R starts a new run on a new map; Esc goes back to the title screen (the prompt says both, side by side, steady, each key as a cap).

## The map

- **Endless**, in every direction. A tile shows as 2 × 2 pixels of pixel art.
- Each run draws a new map from a **seed**, chunk by chunk (32 × 32 tiles) as the player nears them: the same place always looks the same.
- **Obstacles** block the way: trees and rocks, each covering **2 × 2 tiles**. They come in clumps of up to **8**: groves (70% of clumps) and heaps of rocks (30%). Together they cover **8%** of each chunk.
- The player starts with no obstacle within **8 tiles**.
- Healing is **rare**: about one chunk in 33 holds a **herb** on its free ground, one in 100 a **campfire** in a clearing (no obstacle of the chunk within 2 tiles). The same place always holds the same.

## Healing

- A heal shows: small green crosses rise around the player.
- **Herbs**: a hurt player within **2 tiles** of a herb's middle picks it up and gets back **20 HP**. In full health, the herb is left for later. Herbs don't glow: only the torch or a strike of lightning shows them.
- **Campfires**: a hurt player as far as a burning campfire's light shows (its full light and its fading, 12 tiles at its brightest, shrinking as it gives) gets back **3 HP a second**. A campfire gives **24 HP** in all (8 seconds of healing), then dies out, leaving its logs. It lights the night around it, warm and swaying, apart from the torch's white, **6 tiles** at its brightest, less and less as it gives (never under a tile and a half while it burns).

## Hordes

- A **horde** comes in the middle of each stage of weather, once the night has fallen: at **1:30**, then every **2 minutes** (3:30, 5:30…). It brings **30** enemies, and **10** more for each horde before it.
- It is **announced 3 s before**: told in big letters (see The run), then a red glow on the edge it comes from, pulsing slowly, reddest on the edge's middle (on every edge for a ring).
- Until the storm, it comes as a dense **pack** out of one side (a band 16 tiles wide, 5 deep, just beyond the screen); from the storm on, as a **ring** out of every side, announced on every edge.
- Its enemies: zombies; a Shadow in ten from the rain on; an abomination in ten in the tempest. Their HP follow the weather, as every enemy's.
- While some of a horde still stands, a count shows on the top of the map, white on red: how many are left. While several hordes stand, each its own count, by its number, side by side (HORDE 3 · 1, HORDE 4 · 20); one of them wiped out is told with its number under the big letters.
- **Wiped out** to the last, a horde brings **a level**, 2 s on (once told), its cards rarer (**60%** of rare ones); in arcade, it **doubles** the points its kills were worth too. It is told in big gold letters, what it brought under them, the run going on.
- Its zombies show as **ghouls**, pale, in darker rags: a horde stands out from the rest.
- With **Horde Hunter**, its last ones still standing are targeted in red: the very last tenth (the last one at least) pulsing once a second, the others of the share veiled faint and steady (see Passives).

## Score

In arcade, every run is played for its score.

- Every enemy felled is worth **points**: zombie **10**, Shadow **30**, abomination **100**, parasite **1**.
- Times the **weather's multiplier**: ×1 at dusk and in the night, ×1.5 in the drizzle, ×2 in the rain, ×3 in the storm, ×5 in the tempest.
- Times the **combo**: every kill within **2 s** of the last adds ×0.1, up to ×2; 2 s without a kill and it is gone. Rounded down.
- The status line shows, in its middle after the time (on a white badge: in a standard run, the time left to the dawn, counting down; in arcade, the time survived), the score and the multiplier of the moment (the weather's times the combo's) on a gold badge, `SCORE:` in capitals before it, the score's thousands apart. While the run is ahead of the best one kept, its score gleams.
- **Leaderboard**: the 10 best runs, kept in `scores.txt` in the game's data folder. A run that makes it asks for three letters on the death screen (a **new high score** only when it beats the best; otherwise its place), where it shows in its place as they are typed: ↑↓ turn a letter or type it, ←→ move, Enter keeps the run, Esc leaves it off. The leaderboard shows on the title screen too, in arcade, in a box under the mode, the best run in gold.

## The night

- Runs start at **dusk**, for half a minute, just enough to look around: for its first 24 s, the setting sun lights the whole map, warm, less and less, down to half its light; then the night falls all of a sudden, in **6 s**, dimmer and bluer, until only the lights below show anything. Before the dawn, the sun rises the other way round, over the tempest's last minute.
- Then runs take place at night. Everything is dimmer and bluer.
- The player carries a **light**, a torch, out at dusk. It no longer lights up of itself: as the night falls, at **0:26**, it is **for the player to light it**, **F** shown under them, blinking ("[F] light the torch"), until they first do in the run; an Endless Night run, skipping the dusk, opens on it lit. **F** puts it out at once, or lights it again. Lighting up, it **flickers** for as long as luck has it, from not at all to a second and a half, before it holds; flickering, it lights nothing for the run (the enemies don't see the player by it, the guns hold their fire).
- **No battery**: the torch stays lit as long as the player wants, and lights again whenever they want. Lit, everything within **9 tiles** (2 more per level of Flashlight) is lit, fading into the dark over 6 more tiles. Beyond, the map and the enemies are barely seen (10% of their brightness).
- **The torch out**, the player is barely seen, as all the rest; the gems, the campfires and the lightning still show. The enemies no longer see them from afar: an enemy finds the player within **6 tiles**, and loses them again past **8**. One that hasn't found them goes for the nearest burning campfire within **20 tiles**, or wanders, at half its speed. In the dark, the **guns hold their fire**: nothing to aim at (the crowbar and the machete still strike what comes near). A **strike of lightning** gives the player away to every enemy on screen. The hordes always find the player. Lit, or by daylight, every enemy sees the player.
- **Shadows** go the other way round, at night: they hunt the player in the dark, wherever they are, their eyes shining red, faster the longer they hunt (up to **50%** faster after 5 s; back to their speed once lit). The torch lit, they keep away from it: caught in its light, a Shadow **bolts** out of it, 2.5 times its speed for a third of a second.
- **Gems, bullets and the machete's swings glow**: they show in the dark. A gem casts a small glow of its colour on the ground around it, two tiles or so, and lights what passes by it, an enemy, out of the dark.
- **What burns casts a halo**: orange, flickering, on the ground around it: a puddle of fire, a campfire, an enemy burning. A bottle bursts in a flash of fire, lighting **6 tiles** around and fading in a quarter of a second. Where halos meet, the strongest shows, not their sum.
- **Lightning** (see Weather) lights the whole map at once as it strikes, fading back to the dark in 0.8 s. A strike falling with the player **in its ring** (2.5 tiles, see Weather) **puts their torch out**, lit, as F does: the ring tells where to step out of to keep the light; up to them to light it again (F), quick, flickering as it does (up to 1.5 s). Meanwhile the guns hold their fire, the enemies no longer see the player from afar.
- **Blood** (red, ketchup, mayo or pesto, as the settings say; or none): a stain where an enemy falls (bigger for an abomination) and where the player is hit, fading away in **30 s**. A hit player also spurts a few drops every way, for a third of a second.
- **A blow** to the player flushes the edges of the screen red for half a second, whatever the blood setting.
- **Low on HP**, under **30%** (as the HP bar blinks), the player's **heart beats** on the edges of the screen: a soft red pulse, every 1.2 s just under, faster and faster down to every 0.5 s near 0 HP.

## Weather

The weather worsens as the run goes on, a stage every **2 minutes**, as the difficulty does, after the half minute of dusk (see The night). The status line shows, on its right, as wide as the HP bar on its left, the weather now as a gauge, its name in capitals in its middle, filling as its stage goes, each cell's colour on the way from the weather's to the next one's (the dawn's gold last), to start again empty in the next one's colour. The rain thickens over the last quarter of a stage, into the next one's.

| From | Weather | Rain | Lightning | Light | Enemies appearing |
|---|---|---|---|---|---|
| 0:00 | Dusk | none | none | — | — |
| 0:30 | Night | none | none | — | — |
| 2:30 | Drizzle | 4 drops per 1000 pixels | none | — | +15% HP |
| 4:30 | Rain | 8 | none | −1 tile | +30% HP |
| 6:30 | Storm | 14 | every 3 s, **hurting** | −2 tiles | +50% HP |
| 8:30 | Tempest | 20 | every 2 s, **hurting** | −3 tiles | +100% HP, +10% speed |
| 10:30 | Dawn | — | — | — | the run is won (see The run) |

- A strike falls somewhere the player sees, at a moment of its own: between half and one and a half times the weather's wait.
- From the storm on, a strike is foretold: **half a second** before it falls, the ring of the ground it will hurt, **2.5 tiles** around, blinks where it will, faster and faster, dim. Falling, it takes **30 HP** from every enemy within those 2.5 tiles and sets it **on fire** (the ring shows again); it never hurts the player. A strike in three falls where the player stands as it is foretold, one in two in the tempest; the others, anywhere the player sees.
- The light never falls under **4 tiles**.

## The player

The player plays a **hero**, as the skin worn has it (see Progress): each their own HP, speed, weapon and perk.

| Hero | HP | Speed | Weapon | Perk |
|---|---|---|---|---|
| Survivor | 100 | normal | crowbar | **Versatile**: no weakness, the measure of the others |
| Veteran | 120 | +5% | machete | **Commando**: every weapon hurts 10% more |
| Exterminator | 100 | −5% | stun baton | **Pyromaniac**: the enemies set on fire burn twice as long (the flamethrower, the Molotov, the napalm, lightning) |
| Demolisher | 140 | −15% | sledgehammer, and the C4 as their action | **Unshakable**: 10% less damage taken, and never thrown by a burst |
| Special agent | 80 | +10% | combat knife | **Reflexes**: the guns reload 20% faster |
| Medic | 90 | normal | defibrillator | **Healer**: herbs and the heal give back half again as much, campfires heal faster (as much in all); the defibrillator, Life drain and Regeneration heal as for anyone. No HP comes back of itself |
| Ghost (infiltrator) | 90 | +5% | fists | **Stealthy**: the torch out, they see 7 tiles around them all the same, in green, their goggles down over their eyes (blank 4 s once struck by lightning); the torch put out, the enemies further than 2 tiles lose them at once, then notice them within 2 tiles alone, the hordes apart; lightning shows them still, those that saw them following as ever; the Shadows hunt them only after 4 s of dark |
| Riot officer | 110 | −10% | truncheon | **Shield**: 30% less damage from an enemy ahead, the way they face |
| Ninja | 80 | +15% | katana | **Dodge**: a 15% chance to avoid a blow altogether (enemies, a burst), unhurt half a second after as after a hit; a dark ghost of the ninja left aside a moment shows it |

| Stat | Value |
|---|---|
| Speed | 6 tiles/s, the survivor's (the hero's own, then +8% per level of Swiftness) |
| Body | 0.9 × 0.9 tile, smaller than the sprite (2 × 2 tiles): brushing past an obstacle is forgiven |
| HP | The hero's (+20 per level of Vitality); the HP bar blinks under 30% |
| Invincibility after a hit | 0.5 s (the player blinks) |
| Dashes | The ninja's own action, theirs once the run reaches level 3, its card shown alone as it comes (see Passives). One, ready as the card is taken, back 8 s after it is taken (6 s from level 2, 4.5 s from level 4, 3 s at level 5); a second from level 3, coming back after the first; never more. With the card, they show under its icon in the gear, as a gun's rounds: full when all are ready, filling back as they come back |
| Dash | Space or Enter: 3 tiles the way the player faces, in 0.27 s, fading silhouettes of the hero behind; nothing hurts them meanwhile, neither enemies nor lightning |

## Enemies

| Enemy | HP | XP | Speed | Damage | Body | Moves |
|---|---|---|---|---|---|---|
| Zombie | 15 | 1 (3 times in 4) | 2.5 tiles/s | 5 | 0.9 tile | Straight at the player |
| Shadow | 40 | 2 (3 times in 4) | 5.7 tiles/s | 8 | 0.7 tile | Straight, **through obstacles**; hunched, barely seen in the dark but for its red eyes |
| Hold the line | Stand 30 s among 12 enemies within 3 tiles in one run (every tick counted, not in a row) | The Eye-gouger, and the riot officer with their truncheon and their own action from the run's level 3, the shield ram (Space: the player braces 0.3 s, standing still, then charges 5 tiles straight ahead the way they faced (7 from level 4), nothing hurting them, every enemy within a tile of the way hurt, stunned 0.6 s and thrown 4 tiles aside in a V) |
| Close call | Fall under 5 HP, standing, and reach the dawn in the same run | Revival |
| Abomination | 40 | 5 (always) | 2 tiles/s | 15 | 1.6 tile | Straight at the player |
| Parasite | 12 | none | 4.5 tiles/s | 3 | 0.6 tile | Straight at the player, **always finding them**, dark or not; a green worm of zombie flesh, hopping |

- Every enemy is slower than the player: they can always be outrun.
- A felled abomination **swells**, blinking white, for **0.6 s**, then **bursts**: a disc of green bile, out to **3 tiles**, that costs everything it reaches **10 HP**, the player and the enemies alike, and pushes it back **2 tiles**. Out of it, **4 parasites** are thrown, 2.5 tiles away each its own way, as tough as the weather makes every enemy.
- **Obstacles** stop zombies, abominations and parasites, which slide along them; Shadows pass through.
- **Touching** the player costs them the damage of the hardest-hitting enemy touching them; then nothing hurts them for 0.5 s.
- Enemies that overlap **push each other apart**; one touching the player stops there.
- A hit enemy **flashes white**. At 0 HP it falls, and may leave an **XP gem** worth its XP: zombies and Shadows 3 times in 4, abominations always, parasites never.

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

They strike by themselves. Up to **4 weapons**, each up to **level 5**. The run starts with the **hero's own weapon**, at level 1, as the skin worn has it (see Progress), a melee one: the survivor's **crowbar**, the veteran's **machete**, the exterminator's **stun baton**, the demolisher's **sledgehammer**, the special agent's **combat knife**, the ninja's **katana**, the medic's **defibrillator**, the ghost's **fists**, the riot officer's **truncheon**. A hero's own weapon is **theirs alone**: its next levels are dealt to them, never to another hero. The others come from level-ups. Might raises their damage and Haste shortens their cooldowns: the machete's swings, the guns' reloads, the drones' rest (see Passives). Damage is rounded down.

**Obstacles and shots**: the guns and the drones aim only at enemies in sight, no tree nor rock between (a Shadow over a rock is out of sight); a bullet that meets a tree or a rock stops there, a spark where it hits. The melee weapons and the Molotov's bottles, thrown over, don't care.

### Crowbar

The survivor's own weapon, and so the one most runs start with: a heavy blow in an arc before the player, once an enemy is within **2.5 tiles**, every **1.5 s** (less with Haste): **18 damage** to every enemy in the arc (90°), each pushed back **a tile**. Its blow shows iron grey.

**Levels**: 2: a wider arc (120°) · 3: 24 damage · 4: a backhand behind the player after the blow · 5: the enemies hit stand stunned half a second.

### Machete

The veteran's own weapon. A swing in an arc before the player, the way they face, hitting **every enemy in it** for **12 damage**: within reach of the player (plus the enemy's half width) and within the arc. It swings every **0.9 s**, once an enemy is within its reach, whichever way. A swing may hold several **slashes**, 0.2 s apart, each hitting anew: ahead, then **behind**, then ahead again.

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
| 1 | 6 rounds, a shot every 0.5 s, 10 damage, a 2-second reload, range 12, 20 tiles/s | bursts of 8 rounds, a shot every 0.1 s, 3 damage, a 2.5-second reload, range 10, 22 tiles/s |
| 2 | 9 rounds | bursts of 12 |
| 3 | a 1.5-second reload | 5 damage |
| 4 | 2 bullets a shot, at the 2 nearest | a 1.8-second reload |
| 5 | 12 rounds, 15 damage | bursts of 16, bullets pierce 1 enemy |

### Shotgun

A gun (see Guns: its rounds, its reload, it holds its fire in the dark) firing a **double shot**: two shots **0.3 s** apart, then a **2.5-second** reload. Each shot sprays **5 pellets** at random over a fan of **40°**, towards the nearest enemy within **6 tiles**, **6 damage** each, at 18 tiles/s give or take 20%. A pellet shows as a small pixel, gold or red, as the shells.

**Levels**: 2: 7 pellets · 3: 8 damage · 4: a 1.5-second reload · 5: the pellets pierce 1 enemy, the fan spreads over 60°.

### Drones

**A drone** circling the player **2.5 tiles** away, a turn every **2 s**. Each shoots at the nearest enemy within **6 tiles** of it, a bullet every **0.4 s** (less with Haste): **4 damage**, flying 16 tiles a second, 7 tiles far. Not in the dark, as the guns. **Nothing breaks** a drone. Their shots come from one **magazine**, **6 shots** a drone; empty, it reloads in **3 s** (less with Reflexes), the drones holding their fire meanwhile, its rounds then its reload under its icon, as a gun's.

**Levels**: 2: +1 drone (2) · 3: 6 damage · 4: +1 drone (3), reloaded in 2 s · 5: +1 drone (4), 3.5 tiles away, a turn every 1.5 s. The drones are evenly spaced on their circle.

### Molotov

Every **3 s** (less with Haste), a bottle thrown at an enemy at random within **7 tiles**: it flies **0.4 s** in an arc, then bursts into a **puddle of fire**, **1.5 tiles** around, burning **3 s**. Every enemy in the puddle catches fire (see Fire), and burns on once out of it. The puddle flickers with flames and lights around it, warm. In the dark, the torch out, no bottle is thrown: nothing to aim at.

**Levels**: 2: puddles 2 tiles around · 3: 2 bottles a throw, each at an enemy of its own · 4: puddles burn 5 s · 5: a throw every 2 s.

### Flamethrower

A **rare** weapon. A **cone of fire** from the player, **4.5 tiles** long, 60° wide, lit at the **nearest enemy** within its reach, in sight (trees and rocks hide what stands behind them; a barricade doesn't), whichever way the player faces, then burning its **whole tank**, enemies left or not, turning after the nearest enemy as it burns, **180° a second** at most, its last way kept with none left; its fire leaves the nozzle, almost a tile ahead of the player, a stream as from a hose, widening as it goes, a yellow core, orange around, its end red and ragged, the nozzle sweeping from side to side (20° each way, about 1.6 s there and back), its far end lagging, so the stream bends: every enemy in the cone takes **2 damage** about 4 times a second (every 8 ticks, the first at once) and catches fire (see Fire). Its **tank** holds **2 s** of fire; empty, it fills again in **3 s** (less with Haste), its fuel then its filling under its icon, as a gun's rounds and reload. It burns **in the dark** too, the torch out (to be tried).

**Levels**: 2: the cone 5.5 tiles long · 3: 3 damage · 4: a 3 s tank · 5: the cone 6.5 tiles long.

### Stun baton

The exterminator's own weapon. Every **1.2 s** (less with Haste), once an enemy stands on its line, a **thrust** straight ahead of the player, the way they face, **3 tiles** long and a tile wide: every enemy on the line takes **16 damage** and stands **paralysed 1 s**, blinking blue the while. In the dark too. It shows as the baton held out, its double prong ahead, a jagged bolt running from it to the end of its reach, white then cyan, a spark on each enemy it paralyses.

From level 3, every third strike is a **whirlwind**, once an enemy stands within **2 tiles**: the player spins round, the baton held out, from the way they face (down, right, up, left), its prong leaving a white, cyan then blue trail, every enemy within 2 tiles hit and paralysed as by a thrust; every other strike at level 5.

**Levels**: 2: 3.5 tiles long · 3: 22 damage, a whirlwind every third strike · 4: a thrust every 0.9 s · 5: paralysed 2 s, a whirlwind every other strike.

### Katana

The ninja's own weapon. Every **0.8 s** (less with Haste), a **chain of cuts**: the nearest enemy within **1.8 tiles**, then the nearest to it within **2.5 tiles**, and so on, **3** enemies at most: **14 damage** to the first, each next cut a quarter of that less than the one before (100%, 75%, 50%, then 25% on). In the dark too. It shows as a dark ghost of the ninja by each enemy cut, in turn, come from the one before, a white slash across it.

**Levels**: 2: 4 enemies · 3: 20 damage · 4: a chain every 0.6 s · 5: 6 enemies.

### Chainsaw

A **rare** melee weapon, unlocked by Cold blood. It grinds on and on every enemy touching it ahead of the player, the way they face, within **1.5 tiles** (90°): a bite of **4 damage** every **0.2 s** (less with Haste), and each one ground goes **50% slower** while it is. In the dark too. Grinding, it shows held out ahead of the hero, an orange motor then the blade, its teeth running along its edges, blood on its end and its edge (as the blood setting has it). Its **tank** holds **5 s** of grinding, emptying only as it grinds; empty, it stops and fills again in **3 s** (less with Haste), its fuel then its filling under its icon, as the flamethrower's. While it grinds, **nothing hurts the player**: they charge into the crowd, their corners gleaming.

**Levels**: 2: 2 tiles · 3: 6 damage · 4: a wider arc (150°) · 5: 8 damage.

### Vault

The special agent's own **action**, theirs once the run reaches **level 3**, its card shown alone as it comes, played with Space or Enter: over the obstacle **just ahead** (within a tile: a tree, a rock, a barricade), a quick, low jump of **0.4 s**, landing on the first free ground behind it, nothing hurting them on the way; nothing ahead, a **hop of 2 tiles**, short of what stands in its way; something it can't clear right on them, nothing. Drawn a little off the ground, their shadow under them, speed lines behind. It waits **6 s** (less with Haste), its wait under its icon. Level 1: one block (an obstacle) deep; 2: landing, a **kick** throws the enemies within 1.5 tiles back 2 tiles; 3: over an **enemy** too; 4: back in 4 s; 5: up to **3 blocks** in a row, obstacles or enemies. Its next levels are dealt to the agent alone.

### Gatling burst

The veteran's own **action**, theirs once the run reaches **level 3**, its card shown alone as it comes, played with Space or Enter: **2 s** of fire straight ahead, the way they face, **10 bullets a second**, each a tracer as the machine gun's, **8 damage** (with Might), through **1** enemy, stopped by trees and rocks; they may turn as it fires, but go at **half their pace**. The gun shows in their hands the way they face, its muzzle flashing; each spent case falls at their feet, lying there a moment, dulling. It waits **12 s** from when it starts (less with Haste), its wait under its icon. Level 2: 10 damage; 3: a 3 s burst; 4: back in 9 s; 5: through 3 enemies. Its next levels are dealt to the veteran alone.

### Smoke bomb

The ghost's own **action** (the infiltrator's, the scout in their second outfit), theirs once the run reaches **level 3**, its card shown alone as it comes, played with Space or Enter: a cloud of smoke at their feet, **3 tiles** round, for **4 s**. While the scout stands in it, the enemies lose track of them, even with the torch on, a horde's too, the Shadows waiting: all but those within 2 tiles, right on them. Drawn as a dense grey cloud, churning, its edge ragged, spreading out in 0.4 s and thinning out over its last 0.7 s; the scout in it a grey silhouette. It waits **15 s** (less with Haste), its wait under its icon. Level 2: 5 s; 3: 4 tiles round; 4: back in 11 s; 5: the enemies in the smoke 30% slower. Its next levels are dealt to the scout alone.

### Wall of fire

The exterminator's own **action**, theirs once the run reaches **level 3**, its card shown alone as it comes, played with Space or Enter: for **1.5 s** (some 8 tiles as they go), their tank leaks, an unbroken line of fire following their path, a stretch every half tile, **a tile wide**; each stretch burns **5 s**, setting on fire every enemy in it, as a Molotov's puddle (the enemies don't avoid it). Drawn as a band of red and orange on the ground, tongues of fire licking up from it, their tips yellow; its last second, scattered embers. It waits **10 s** from when it is lit (less with Haste), its wait under its icon. Level 2: walls of 7 s; 3: a 2 s trail; 4: back in 7 s; 5: walls 2 tiles wide. Its next levels are dealt to the exterminator alone.

### First-aid kit

The medic's own **action**, theirs once the run reaches **level 3**, its card shown alone as it comes, played with Space or Enter: **15 HP** back at once (the medic's alone: Healer not counted again), small green crosses rising; then it waits **20 s** (less with Haste), its wait under its icon. Not opened at full HP: it stays ready. Level 2: 20 HP; 3: back in 15 s; 4: 25 HP; 5: back in 12 s. Its next levels are dealt to the medic alone.

### C4

The demolisher's own **action**, theirs once the run reaches **level 3**, its card shown alone as it comes ("Hero skill available", the run standing still until Space or Enter, "PRESS [SPACE] TO GO ON" blinking under it, the key on its cap), played with Space or Enter. A first press sets a **charge** down at their feet: a small dark block, its red light blinking faster and faster, glowing in the dark. A second press **blows it, from afar**, wherever the player stands; left alone, it blows by itself **3 s** later. Its blast: every enemy within **2.5 tiles** takes **25 damage** and is thrown back **3 tiles**; a white flash over the whole blast, then a fireball shrinking, lighting around it, then smoke rising. One charge at a time; the next can be set down **5 s** after the blast (less with Haste), its wait under its icon. Its next levels are dealt to the demolisher alone.

**Levels**: 2: a 3-tile blast · 3: 40 damage · 4: ready 4 s after the blast · 5: **two charges**, set down one after the other, the next press blowing both together.

### Eye-gouger

A **rare** gun, the flashball unlocked by Hold the line: every **1.5 s** (less with Haste), a single slow shot, a black ball, at the nearest enemy in sight within **7 tiles**: **30 damage**; the enemy hit is **thrown back 3 tiles** and **stunned 1 s**, stars turning over its head, showing white a moment; one it fells **bursts**, a spray of blood and chunks thrown out the way the shot went ("non-lethal"), the blood as the settings have it, a white flash alone without it. Not in the dark, as the other guns. Level 2: 40 damage; 3: every 1.2 s; 4: stunned 1.5 s; 5: the ball bounces to a second enemy near, a quarter less damage.

### Sniper rifle

A **rare** weapon, unlocked by Shadow. Every **3 s** (less with Haste), it aims at the **toughest** enemy within **12 tiles**, the furthest of those, in sight past trees and rocks, **light or dark**: unlike the other guns, it aims at the enemies out of the light, and shoots with the torch out. A **red dot** shows on the enemy aimed at, glowing a little, for **half a second**; then the shot, a short bright tracer flying to it in an eighth of a second: **45 damage**, enough for a Shadow, a white star where it strikes and blood thrown out behind the enemy, the way it went (as the blood setting has it), all of it showing in the dark too. The enemy gone meanwhile, it aims anew.

**Levels**: 2: 15 tiles · 3: 70 damage · 4: a shot every 2 s · 5: the bullet goes through **every enemy on its line**.

### Grenade

A **rare** weapon, unlocked by Demolisher. Every **3 s** (less with Haste), a **grenade** thrown at the biggest crowd within 8 tiles, arcing, its fuse sparking, for half a second; where it lands, every enemy within **2 tiles** takes **20 damage** and is thrown back 1.5 tiles, a fireball as the C4's. Not in the dark, as the Molotov.

**Levels**: 2: a 2.5-tile blast · 3: 30 damage · 4: a grenade every 2 s · 5: **two grenades** at once, each at a crowd of its own.

### Napalm

A **rare** weapon, for every hero, unlocked with the veteran by Legend. Every **20 s** (less with Haste), a **carpet of fire bombs** on the biggest crowd within **8 tiles** of the player (the enemy with the most others within 2 tiles, the nearest of those): a line of **5 bombs**, a tile apart, through it, the way from the player to it, falling from the player's side, one every 0.1 s. Each shows as a streak dropping from the sky, a white head, grey behind, for a quarter of a second; where it lands, every enemy within **1.25 tiles** takes **30 damage** and catches fire (see Fire), a fireball shows as the C4's, and the ground burns **2 s**, 0.9 tile around, as a Molotov's puddle: the strip left burning. In the dark too: it strikes from the sky.

**Levels**: 2: a line of 7 bombs · 3: 45 damage · 4: a strike every 15 s · 5: **two lines crossed**.

### Shurikens

A gun of a kind, unlocked by Shadow hunter: volleys of **3 stars** thrown at the nearest enemy in sight, every 0.3 s, **9 damage**, each piercing an enemy, flying 8 tiles, a small white star with a streak behind it; then **1.4 s** to take the next ones out. As the guns, they hold in the dark.

**Levels**: 2: 4 stars a volley · 3: taken out in 1 s · 4: a star that hits an enemy **ricochets** on the nearest enemy left within 4 tiles, then on, 3 times: 100%, 75%, 50%, 25% of its damage, piercing no more · 5: 2 targets at once.

### Defibrillator

The medic's own weapon. Every **1 s** (less with Haste), a **shock** at the nearest enemy within **2 tiles**, leaping on to the nearest to it within 2 tiles, and so on, **3** enemies at most, **12 damage** each; each enemy it fells gives the medic back **1 HP**. In the dark too. It shows as a jagged bolt leaping from the hero through each enemy shocked, white then cyan.

**Levels**: 2: 4 enemies · 3: 16 damage · 4: a shock every 0.8 s · 5: 2 HP a kill.

### Truncheon

The riot officer's own weapon. Every **1 s** (less with Haste), once an enemy is within its reach, a **blow** in an arc ahead, the way they face, **2 tiles**, 120° wide: every enemy in it takes **14 damage** and is thrown **1.5 tiles** back. It shows as the crowbar's blow, iron grey.

**Levels**: 2: 2.5 tiles · 3: 20 damage · 4: a wider blow (180°) · 5: the blow **stuns** 0.5 s.

### Fists

The ghost's (infiltrator's) own weapon. Every **0.4 s** (less with Haste), a **punch** at the nearest enemy within **1.2 tiles**, whichever way: **8 damage**; every third a **heavier** one, twice as hard, throwing the enemy **a tile** back. In the dark too. It shows as the fist out halfway to the enemy struck and a small white burst on it, bigger and yellow for a heavier one.

**Levels**: 2: 1.6 tiles · 3: 12 damage · 4: a punch every 0.3 s · 5: the heavier punch **stuns** 1 s.

### Combat knife

The special agent's own weapon. Every **0.4 s** (less with Haste), a quick **stab** at the **nearest enemy** within **1.5 tiles**, whichever way: **8 damage**. Little a stab, but many: the weapon of the tight melee. In the dark too. It shows as the blade flicking out from the hero towards the enemy stabbed, and a drop of blood falling from it (as the blood setting has it; none when off).

**Levels**: 2: 2 tiles · 3: 12 damage · 4: the 2 nearest enemies · 5: a stab every 0.2 s.

### Sledgehammer

The demolisher's own weapon, a melee one. Every **2 s** (less with Haste), once an enemy stands where it would land, the hammer is **raised** over the player's head **0.2 s**, then falls on the ground **2.5 tiles** ahead of them, the way they face then: its **shock** takes **28 HP** from every enemy within **2 tiles** of where it lands and throws them **1.5 tiles** back, away from it, over **0.2 s**, as its wave goes out. In the dark too. It shows as the hammer raised, then down on the ground a moment, its head three pixels wide; and the **wave** of its shock, a thick band on the ground, white at its front, going out to its reach as fast as it throws the enemies out, fading where it stops. Its wait shows under its icon.

From level 3, **shocks** follow on, one after the other, every **2 tiles** further ahead, every **0.13 s**: **3**, each as the blow's own, 1.25 tiles around, throwing back what it hits; each enemy hit once by a blow and its shocks. Never back towards the player.

**Levels**: 2: the shock 2.5 tiles around, 40 HP · 3: the shocks · 4: a blow every 1.5 s · 5: as many shocks **left and right** of where it lands too, the enemies **stunned** 1 s.

### Fire

What sets enemies on fire (the Molotov, the flamethrower; the shotgun's Dragon's breath to come): an enemy on fire **burns 3 s**, losing **1 HP every half second** (6 HP in all); set on fire again, it burns anew, never longer. Flames flicker over its head, and it lights a tile and a half around it, warm. Burnt to death, it leaves **ash**, grey, rather than blood (whatever the blood setting).

### To come

| Weapon | What it does |
|---|---|
| Gas *(planned)* | A toxic cloud around the player. |
| Airstrike *(planned)* | Strikes falling at random around the player, each marked on the ground just before it blows. |
| Explosive rounds *(planned)* | Felled enemies blow up, hurting their neighbours: chain reactions. |

## Passives

Up to **4 passives**, each up to **level 5**.

| Passive | Per level |
|---|---|
| Vitality | +20 max HP, and heals as much |
| Swiftness | +8% speed |
| Magnet | Gems fly in from 1 tile further (2 tiles at first) |
| Might | +10% damage of every weapon, rounded to the nearest, a bonus giving at least 1 (so a weak hit gains too); the hero's own bonus (Commando) on top |
| Haste | −8% to every weapon's cooldown: the melee blows, the guns' and the drones' fire and reload, the throws, the barricade's and the shield ram's recharge |
| Flashlight | +2 tiles of light around the player (9 at first) |
| Loot | +10 points of percent for an enemy to leave a gem (an abomination always does) |
| Regeneration | +0.4 HP a second, of itself (tenths kept until they make a whole one) |
| Horde Hunter | +10% damage from the weapons to an enemy of a horde, per level (+50% at level 5). A horde's last ones targeted in red: its last tenth at level 1, a quarter at 2, seen in the dark too from level 3, a half at 4, the whole horde at 5 |
| Life drain | **Rare**. Each enemy felled has a chance to give back **1 HP**: 10% at level 1, 5 points more each level (30% at level 5) |
| Glare | The enemies as far as the torch's light shows, its fading included (15 tiles, and Flashlight's), go 10% slower, while it is lit: every kind, the hordes too |
| Wisdom | +10% XP from the gems (tenths of a point kept until they make a whole one) |

## Actions

Cards played with **Space or Enter**, when the player wants. A run holds **one**: once taken, no other action card is dealt, only its next levels. A card says ACTION on its top border, as others say WEAPON or PASSIVE.

| Action | What it does |
|---|---|
| Dash | The ninja's own action, at the run's level 3, its next levels dealt to them alone. A dash (see The player): back in 8 s, then sooner and sooner; a second dash at level 3 |
| Barricade | The survivor's own action, at the run's level 3, its next levels dealt to them alone. Three **crates**, set down just ahead of the player, the way they face, **4 tiles across** their way and 2 along it, centred on them: facing up or down, heaped wide; facing left or right, in a staggered file going away. Drawn among the figures, in front of those above it. Not set down on a tree, a rock or another barricade: the card stays ready, and the barricade shows where it would have stood in red, blinking, 0.4 s; the walking enemies in its way are thrown just beyond it. It stops the **walking** enemies as a rock does, and the player too; the **Shadows** fly over it, the **shots** go over it (it hides no enemy from the guns). Every second, each walking enemy touching a wooden one hits it for its damage, as it would the player (a zombie 5, a parasite 3, an abomination 15), the crate showing white a moment; at 0 HP it breaks, the ground free again. An abomination's **burst** blows away the wooden barricades within its reach. The next comes 8 s after one is set down, its gauge under the card's icon in the gear; at most 2 stand at once, the oldest taken away for a new one. Level 1: 80 HP; 2: 120 HP; 3: a third at once, the next in 6 s; 4: 180 HP; 5: **metal** crates, dark steel and grey, unbreakable (neither the enemies nor a burst harm them; they go only when the oldest is taken away for a new one), the wooden ones already standing staying wood |

## Legendaries

- A level-up offers a **legendary** **5%** of the time, until one is found: **one a run**, no other offered after it; on a **golden card**, in place of its last choice. Each level-up without one raises the chance by a point (6%, 7%…), back to 5% once one is offered.
- A legendary has **one level**, and takes no room among the passives; once found, no legendary is offered again in the run. Its card says PASSIVE on its top border, "takes no slot" in its text. It shows in the gear with no level written.

| Legendary | What it does |
|---|---|
| Illumination | **+1 XP every second**, of itself |
| Revival | Down to **0 HP**, the hero comes back **once** with **half** their HP, untouchable **2 s**, a **wave** throwing the enemies within 4 tiles back 3 tiles: their soul out of their body, up, then back into it, the body grey meanwhile; then a white ring and the green crosses of a heal. Used up, its icon is greyed and crossed out in red in the gear and on the pause screen, its name too in the list |

## Progress

- **Achievements**: goals reached once, kept from one run to the next, in both modes; each unlocks a card, dealt from then on, or a reroll more a run. At first, the cards dealt are two weapons, the **pistol** and the **shotgun** (and the hero's own weapon, see Weapons); Vitality, Swiftness, Magnet, Flashlight, Wisdom, Loot; and the heal.

| Achievement | Goal | Unlocks |
|---|---|---|
| Hunter | Wipe out a horde to the last | Horde Hunter |
| First drops | Hold out until the drizzle | Haste |
| Carnage | 300 enemies down in a single run | Might |
| Swarm | Wipe out 2 hordes in one run | The drones |
| Rain of lead | Hold out until the rain | Machine gun |
| In the eye | Hold out until the storm | Molotov |
| A thousand down | 1,000 enemies down, in all (every run counted) | Regeneration |
| Raw light | 50 Shadows down, in all (every run counted) | Glare |
| Dawn | Hold out until dawn | The arcade mode (legendary) |
| Cleaner | Wipe out every horde of a standard run: none left standing and none to come before the dawn | The flamethrower, and the exterminator's skin |
| Demolisher | Fell 10 abominations by a melee blow (the crowbar's, the machete's, the truncheon's, the sledgehammer's and its shocks', the baton's, the knife's, the katana's, the fists', the defibrillator's, the chainsaw's, the shield ram's) in one run | The grenade, and the demolisher with their sledgehammer and their C4 |
| Cold blood | Hold a minute under 30% of HP in one run (every tick counted, not in a row) | The chainsaw, and the special agent with their combat knife |
| Shadow hunter | Fell 25 Shadows in one run | The shurikens, and the ninja with their katana |
| Legend | Fell 700 enemies in one run | The napalm, and the veteran with their machete |
| First aid | Get 300 HP back in one run (herbs, campfires, regeneration, the heal, the defibrillator) | Life drain, and the medic with their defibrillator |
| Shadow | Keep the torch out 3 minutes in one run (every tick counted, not in a row) | The sniper rifle, and the ghost with their fists |
| A regular | 25 runs played | A second reroll a run |
| Seasoned | Reach level 25 in a single run | Illumination |

- **Told as reached**: a box rises on the bottom right of the map, the picture of what it unlocks and the achievement's name, in its **rarity's** colour (the rarity of what it unlocks: white, blue for rare, gold for legendary; a reroll, rare); it stays 4 s, then goes down. Several, one after the other. What it unlocks is dealt from the next run on: a run's cards are set as it starts.
- The **collection**, on the title screen in standard mode, where the arcade shows its leaderboard: every card of a build but the heroes' own weapons and actions, and the reroll, a tile each: by kind: the weapons, the passives, the legendaries, then the reroll and the arcade mode, its tile a joystick; each kind by rarity, the common first, and among them those there from the start, then from the easiest to the hardest to unlock; in colour once the player has it, grey until then. Neither the drones nor the heal. Hidden at first, its key at the bottom of the screen, by the mode's ([C] COLLECTION 16 / 22): **C** shows or hides it, shown at its first tile, the arrows going from tile to tile, Esc hiding it too; **Tab** changes the mode. The tile picked is framed. A box under the tiles tells it: its name in its rarity's colour, its kind on the right (weapon, passive, the legendary one too, bonus for the reroll, mode for the arcade); available from the start, or the achievement unlocking it, its goal and, for those counted over every run, a row apart, in capitals in the box's middle, **Unlocked** in green or **Locked** in grey; for those counted over every run and not yet reached, a bar as the XP bar instead, across the box, the count in its middle. The box as wide for every tile. Two achievements are **hidden** until reached, the dawn's (the arcade) and the seasoned one's (Illumination): their tiles a pixel question mark, their box saying HIDDEN ACHIEVEMENT in its middle, nothing more. Short of room, only the picked tile's row shows, arrows on its sides where more tiles lie.
- The **heroes**, on the title screen in either mode, where the collection fits: **H** shows them in the collection's place (or the leaderboard's), at the hero worn. A tile each, as the collection's: the hero from the front (the survivor in a leather jacket and jeans, the veteran in a red headband, the exterminator in a yellow hazmat suit, a dark grey gas mask and boots, a red tank on the back, the demolisher bald, a ginger beard, in a white tank top and jeans, the special agent fair-haired in a brown leather jacket, a dark harness across, charcoal trousers, the ninja all in dark, the eyes alone showing, the medic all in white, a green bag on the back), twice as big, in colour once unlocked, grey until then; the tile picked framed, its hero turning round once it has been picked a second (front, side, back, side). The skin worn has a green check on the bottom right of its picture. While they show, the blinking prompt says Space wears the skin picked (nothing for a locked one). ←→ go from tile to tile; **Enter** or Space wear the one picked, once unlocked, and hide them; Esc or H again hide them. Each hero comes in **two outfits**, the second with a **name** of its own, shown in place of the hero's in the box and on the run's end screen, the hero playing the same (the survivor, or the **biker**: a black jacket, a white tee, charcoal jeans; the veteran: a red headband and a blue tunic, or the **guerrilla**: a blue headband, bare-chested, a leather strap across, dark green trousers; the exterminator, or the **physicist**: an orange suit, dark grey arms and legs, the grey mask; the demolisher, or the **blacksmith**: a dark beard, a black tank top, brown trousers; the special agent: a brown leather jacket, a dark harness across, charcoal trousers, or the **rookie**: a young police officer's blue uniform; the ninja, or the **scarlet ninja**: all in black, red across the middle; the medic: white and a green bag, or the **surgeon**: green and a red bag). The hero picked shows little pixel arrows over and under their tile, picked: **↑↓** (or Z/W and S) go through them, kept at once, each hero their own. A hero's other colours stay hidden while the hero is locked: no arrows, their first colours alone. A hero's **second colours** wait for the **dawn** reached with them (kept in the progress): until then they show in grey, the box saying so, and the hero wears their first in a run. Each hero starts a run with a **weapon of their own** (see Weapons), and plays with their own HP, speed and perk (see The player): the box under the tiles says them, under where the hero comes from: their HP and speed, their perk; then their **starting gear**, a card each side by side: their weapon and their own action (coming at the run's level 3), its icon on the left, its name and what it does on the right. The skin worn is kept for the next runs; the exterminator chosen, the progress erased since, the survivor shows. A box under the tiles tells it, as the collection's: its name, HERO on the right; available from the start, or the achievement unlocking it and its goal; a row apart, **Worn** in gold, Unlocked in green or Locked in grey. Its key at the bottom of the screen, after the collection's: [H] HEROES 2 / 7.
- The progress is kept in `progress.ron`, in the game's data folder.

## Experience

- Fallen enemies may leave an XP gem (see Enemies). A gem covers one tile. A gem within **2 tiles** (more with Magnet) flies to the player (12 tiles/s) and keeps on until picked up.
- A gem shows by its worth: **silver** for a point, **blue** for 2 to 4 (a Shadow's), **gold** for 5 and more (an abomination's). Each glows its colour around it in the dark.
- At most **500 gems** lie around: past that, the oldest go.
- The next level takes **5 + 3 × (level − 1)** XP: 5 from level 1, 8 from level 2, 11 from level 3. XP beyond it carries over.
- **Level 3** brings the hero's own action (its card shown alone, "Hero skill available") in place of a choice of cards: one reward a level. Two levels at once, 2 to 4, bring the action and a choice for level 4.
- **Rarity**: each card is **common** (grey frame) or **rare** (blue frame, its word on the bottom border), the legendaries apart (see Legendaries). Common: every other weapon (the heroes' own among them), Vitality, Swiftness, Magnet, Flashlight, Loot, the heroes' own actions, the heal. Rare: the flamethrower, the chainsaw, the grenade, the sniper rifle, the Eye-gouger, the napalm, Might, Haste, Wisdom, Regeneration, Glare, Life drain. A weapon or a passive keeps its rarity at every level.
- **Level-up**: the game pauses and offers **3 different choices** at random among what can still be picked, each a rare one **25%** of the time (a common one otherwise, or the other if none is left of its rarity): a new weapon or passive (while there is room), or the next level of one owned. At dusk, neither Flashlight nor Glare is offered: the torch isn't lit yet. With nothing left to pick, the one choice is **Heal 20 HP**. Several levels at once offer their choices one after the other.
- **Kind**: a card's top border says whether it is a weapon or a passive, by its number.
- **Room and reroll**: under the cards, the room left: weapons and passives the player has, out of 4 each. Once a run, **R** deals the cards on offer again, as rare as they were; the rerolls left show under the cards too.
- **Choosing**: ←/→ (or Q/A and D) then Enter (or Space), or 1, 2, 3 straight away.
