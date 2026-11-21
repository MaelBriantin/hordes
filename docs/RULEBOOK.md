# hordes — Rulebook

What every rule of a run does, with its numbers. Rules not in the game yet are marked *(planned)*; numbers still to be set are marked *TBD*.

## Controls

- **Move**: the arrows, WASD or ZQSD, in **4 directions**, no diagonal: of the keys held down, the last one pressed wins (D then W goes up; letting go of W goes right again).
- **Start**: the game opens on its title screen; Enter starts a run, in the mode chosen with Tab (see Modes). Under the game's name, the mode as its subtitle, in small pixel letters: **Survive the night** in the clear night's grey, as its word is told, or **Arcade** in gold, a gleam sweeping across it now and then. At the bottom left, the key and the other mode it changes to.
- **Keys**: ? lists every key of a run, the run standing still; ?, Esc or Enter leave the list.
- **Gear**: I shows or hides the gear over the top left of the map, the choice kept: each card's icon, in the order picked, its level written on its bottom right corner on a dark badge (MAX in gold at the last); right under a gun's, its rounds or its reload; under the Dash card's, its dashes.
- **Torch**: F, once it has lit up of itself, at 0:26 (see The night).
- **Dash**: Shift, the way the player faces, once the Dash card is taken and a dash is ready (see The player). Without kitty's keyboard protocol, a terminal doesn't tell Shift alone: Shift with a way (a capital W, A, S, D, Z or Q) dashes.
- **Pause**: P or Esc, during the run or a level-up: the run stands still, and a menu offers to resume (P or Esc again), open the options, start a new run, go back to the title screen or quit. ↑↓ choose, Enter confirms.
- **Options**: O or Esc on the title screen (where they hold the way to quit), or from the pause menu. ↑↓ choose, ←→ or Enter change a setting, Esc goes back.
  - **Blood**: red, ketchup, mayo, pesto or off.
  - **Rain**: how it shows (full, reduced to half its drops, or off). The weather itself is the same whatever the choice.
  - **Arrow**: a small arrow over the player's head, bouncing, to see them at once in the crowd: off, white, yellow or red. Whatever the setting, **E** shows the arrow for **2 s** (white, if off): the player found again in the crowd.
  - **Announcements**: how a new weather and a horde are told: big, with a pause (in big letters, the world standing still 2 s); big, no pause (in big letters 2 s, the run going on); or a banner (on the top row of the map 3 s, the run going on). Big, no pause, at first. The opening is always told big, with a pause.
  - **Gear**: hidden, as icons, or as a list over the top left of the map (I in a run shows or hides it); hidden at first. As a list, a line a card: its name, its level (MAX in gold at the last), its gauge if it has one. The opening and the hordes are always told in big letters.
  - **HP over the hero**: shown or hidden: the player's HP in a small bar over their head (over the arrow, when it shows).
  - **Coloured ground** (experimental): off or on: on, the bare ground is dark grass speckled with earth, lit as the rest, rather than the screen's dark. Off at first: it redraws most of the screen at every step, for a modern, fast terminal (Ghostty, kitty, WezTerm…).
  - **Dusk**: played or skipped: skipped, a run opens on the night, the torch lit.
  - **Language**: English or French. At first, the system's language if the game speaks it (`LANG`), English otherwise.
- These choices are kept for the next runs (`settings.txt` in the game's data folder).
- The title screen and the pause menu show the game's version.
- **Quit**: from the pause menu, or the title screen's options; Esc on the game over; Ctrl+C at any time.
- **The camera** keeps the player in the middle of the screen.
- **Held keys**: with kitty's keyboard protocol (Ghostty, kitty, WezTerm, iTerm2…), a key holds exactly as long as it is held down. Elsewhere, the terminal only repeats presses: a key counts as held 0.5 s after a press and 0.15 s after each repeat, and holding two keys at once may not work.

## Modes

- **Standard**: from dusk to dawn, as below; no score, no leaderboard: the death screen tells the time survived, the level and the kills. The only mode at first.
- **Arcade**: unlocked by an achievement, every weather seen once (see Progress); until then, no Tab on the title screen, the standard alone. Endless, for the score. The stages of weather last **3 minutes** and go up to the tempest and back down to the night, again and again (night, drizzle, rain, storm, tempest, storm, rain, drizzle, night…), with no dawn. The run grows harder with the time, **7% a minute**, compounded (×2 at 10 minutes, ×3.9 at 20): as many more enemies appear (1.5 a second at first), as many more HP they have, and the score's weather multiplier is that pressure. The leaderboard is the arcade's.

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
- With **Horde Hunter**, its last ones still standing are veiled in red: the very last tenth (the last one at least) pulsing once a second, the others of the share veiled faint and steady (see Passives).

## Score

In arcade, every run is played for its score.

- Every enemy felled is worth **points**: zombie **10**, Shadow **30**, abomination **100**, parasite **1**.
- Times the **weather's multiplier**: ×1 at dusk and in the night, ×1.5 in the drizzle, ×2 in the rain, ×3 in the storm, ×5 in the tempest.
- Times the **combo**: every kill within **2 s** of the last adds ×0.1, up to ×2; 2 s without a kill and it is gone. Rounded down.
- The status line shows the score and the multiplier of the moment (the weather's times the combo's) on a gold badge, the score's thousands apart. While the run is ahead of the best one kept, its score gleams.
- **Leaderboard**: the 10 best runs, kept in `scores.txt` in the game's data folder. A run that makes it asks for three letters on the death screen (a **new high score** only when it beats the best; otherwise its place), where it shows in its place as they are typed: ↑↓ turn a letter or type it, ←→ move, Enter keeps the run, Esc leaves it off. The leaderboard shows on the title screen too, in arcade, in a box under the mode, the best run in gold.

## The night

- Runs start at **dusk**, for half a minute, just enough to look around: for its first 24 s, the setting sun lights the whole map, warm, less and less, down to half its light; then the night falls all of a sudden, in **6 s**, dimmer and bluer, until only the lights below show anything. Before the dawn, the sun rises the other way round, over the tempest's last minute.
- Then runs take place at night. Everything is dimmer and bluer.
- The player carries a **light**, a torch, out at dusk: it lights up of itself at **0:26**, as the night falls. **F** puts it out at once, or lights it again. Lighting up, it **flickers** for as long as luck has it, from not at all to a second and a half, before it holds; flickering, it lights nothing for the run (the enemies don't see the player by it, the guns hold their fire).
- **No battery**: the torch stays lit as long as the player wants, and lights again whenever they want. The status line shows the torch, on or out, as a bulb: green while it is on, grey-blue while it is out. Lit, everything within **9 tiles** (2 more per level of Flashlight) is lit, fading into the dark over 6 more tiles. Beyond, the map and the enemies are barely seen (10% of their brightness).
- **The torch out**, the player is barely seen, as all the rest; the gems, the campfires and the lightning still show. The enemies no longer see them from afar: an enemy finds the player within **6 tiles**, and loses them again past **8**. One that hasn't found them goes for the nearest burning campfire within **20 tiles**, or wanders, at half its speed. In the dark, the **guns hold their fire**: nothing to aim at (the machete and the drones still strike what comes near). A **strike of lightning** gives the player away to every enemy on screen. The hordes always find the player. Lit, or by daylight, every enemy sees the player.
- **Shadows** go the other way round, at night: they hunt the player in the dark, wherever they are, their eyes shining red, faster the longer they hunt (up to **50%** faster after 5 s; back to their speed once lit). The torch lit, they keep away from it: caught in its light, a Shadow **bolts** out of it, 2.5 times its speed for a third of a second.
- **Gems, bullets and the machete's swings glow**: they show in the dark. A gem casts a small glow of its colour on the ground around it, two tiles or so, and lights what passes by it, an enemy, out of the dark.
- **What burns casts a halo**: orange, flickering, on the ground around it: a puddle of fire, a campfire, an enemy burning. A bottle bursts in a flash of fire, lighting **6 tiles** around and fading in a quarter of a second. Where halos meet, the strongest shows, not their sum.
- **Lightning** (see Weather) lights the whole map at once as it strikes, fading back to the dark in 0.8 s.
- **Blood** (red, ketchup, mayo or pesto, as the settings say; or none): a stain where an enemy falls (bigger for an abomination) and where the player is hit, fading away in **30 s**. A hit player also spurts a few drops every way, for a third of a second.
- **A blow** to the player flushes the edges of the screen red for half a second, whatever the blood setting.

## Weather

The weather worsens as the run goes on, a stage every **2 minutes**, as the difficulty does, after the half minute of dusk (see The night). The status line shows every weather's icon in a row, and the dawn's last, the weather now in its colour with its name after the row, those gone put out in grey, those to come barely there; the next one comes out of the dark as the stage goes. The rain thickens over the last quarter of a stage, into the next one's.

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
- From the storm on, a strike is foretold: **half a second** before it falls, the ring of the ground it will hurt, **2.5 tiles** around, blinks where it will, faster and faster, dim. Falling, it takes **30 HP** from every enemy within those 2.5 tiles and sets it **on fire** (the ring shows again), and **20 HP** from the player, if they are still in it (not at daybreak). A strike in three falls where the player stands as it is foretold, one in two in the tempest; the others, anywhere the player sees.
- The light never falls under **4 tiles**.

## The player

| Stat | Value |
|---|---|
| Speed | 6 tiles/s (+8% per level of Swiftness) |
| Body | 0.9 × 0.9 tile, smaller than the sprite (2 × 2 tiles): brushing past an obstacle is forgiven |
| HP | 100 (+20 per level of Vitality); the HP bar blinks under 30% |
| Invincibility after a hit | 0.5 s (the player blinks) |
| Dashes | None at first: the Dash card brings them (see Passives). One, ready as the card is taken, back 8 s after it is taken (6 s from level 2, 4.5 s from level 4, 3 s at level 5); a second from level 3, coming back after the first; never more. With the card, they show under its icon in the gear, as a gun's rounds: full when all are ready, filling back as they come back |
| Dash | Space or Enter: 3 tiles the way the player faces, in 0.27 s, fading silhouettes of the hero behind; nothing hurts them meanwhile, neither enemies nor lightning |

## Enemies

| Enemy | HP | XP | Speed | Damage | Body | Moves |
|---|---|---|---|---|---|---|
| Zombie | 15 | 1 (3 times in 4) | 2.5 tiles/s | 5 | 0.9 tile | Straight at the player |
| Shadow | 40 | 2 (3 times in 4) | 5.7 tiles/s | 8 | 0.7 tile | Straight, **through obstacles**; hunched, barely seen in the dark but for its red eyes |
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

They strike by themselves. Up to **4 weapons**, each up to **level 5**. The run starts with the **crowbar**; the others come from level-ups. Might raises their damage and Haste shortens their cooldowns: the machete's swings, the guns' reloads, the drones' rest (see Passives). Damage is rounded down.

### Crowbar

The weapon every run starts with: a heavy blow in an arc before the player, once an enemy is within **2.5 tiles**, every **1.5 s** (less with Haste): **18 damage** to every enemy in the arc (90°), each pushed back **a tile**. Its blow shows iron grey.

**Levels**: 2: a wider arc (120°) · 3: 24 damage · 4: a backhand behind the player after the blow · 5: the enemies hit stand stunned half a second.

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
| 1 | 6 rounds, a shot every 0.5 s, 10 damage, a 2-second reload, range 12, 20 tiles/s | bursts of 8 rounds, a shot every 0.1 s, 3 damage, a 2.5-second reload, range 10, 22 tiles/s |
| 2 | 9 rounds | bursts of 12 |
| 3 | a 1.5-second reload | 5 damage |
| 4 | 2 bullets a shot, at the 2 nearest | a 1.8-second reload |
| 5 | 12 rounds, 15 damage | bursts of 16, bullets pierce 1 enemy |

### Shotgun

A gun (see Guns: its rounds, its reload, it holds its fire in the dark) firing a **double shot**: two shots **0.3 s** apart, then a **2.5-second** reload. Each shot sprays **5 pellets** at random over a fan of **40°**, towards the nearest enemy within **6 tiles**, **6 damage** each, at 18 tiles/s give or take 20%. A pellet shows as a small pixel, gold or red, as the shells.

**Levels**: 2: 7 pellets · 3: 8 damage · 4: a 1.5-second reload · 5: the pellets pierce 1 enemy, the fan spreads over 60°.

### Drones

Drones circling the player **2.5 tiles** away, a turn every **2 s**. A drone hits the first enemy it runs into: **8 damage**, and a push of **half a tile** away from the player. The blow **breaks** it: it keeps circling, grey and harmless, and repairs in **2 s** (less with Haste).

**Levels**: 2: +1 drone · 3: 12 damage · 4: +1 drone (3) · 5: 3.5 tiles away, a turn every 1.5 s. The drones are evenly spaced on their circle.

### Molotov

Every **3 s** (less with Haste), a bottle thrown at an enemy at random within **7 tiles**: it flies **0.4 s** in an arc, then bursts into a **puddle of fire**, **1.5 tiles** around, burning **3 s**. Every enemy in the puddle catches fire (see Fire), and burns on once out of it. The puddle flickers with flames and lights around it, warm. In the dark, the torch out, no bottle is thrown: nothing to aim at.

**Levels**: 2: puddles 2 tiles around · 3: 2 bottles a throw, each at an enemy of its own · 4: puddles burn 5 s · 5: a throw every 2 s.

### Fire

What sets enemies on fire (the Molotov; the shotgun's Dragon's breath to come): an enemy on fire **burns 3 s**, losing **1 HP every half second** (6 HP in all); set on fire again, it burns anew, never longer. Flames flicker over its head, and it lights a tile and a half around it, warm. Burnt to death, it leaves **ash**, grey, rather than blood (whatever the blood setting).

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
| Might | +10% damage of every weapon |
| Haste | −8% to every weapon's cooldown, the drones' repair included |
| Flashlight | +2 tiles of light around the player (9 at first) |
| Loot | +10 points of percent for an enemy to leave a gem (an abomination always does) |
| Regeneration | +0.4 HP a second, of itself (tenths kept until they make a whole one) |
| Horde Hunter | +10% damage from the weapons to an enemy of a horde, per level (+50% at level 5). A horde's last ones veiled in red: its last tenth at level 1, a quarter at 2, a half at 3, three quarters at 4, the whole horde at 5; as far as the light shows, then from level 3 in the dark too |
| Glare | The enemies as far as the torch's light shows, its fading included (15 tiles, and Flashlight's), go 10% slower, while it is lit: every kind, the hordes too |
| Wisdom | +10% XP from the gems (tenths of a point kept until they make a whole one) |

## Actions

Cards played with **Space or Enter**, when the player wants. A run holds **one**: once taken, no other action card is dealt, only its next levels. A card says ACTION on its top border, as others say WEAPON or PASSIVE.

| Action | What it does |
|---|---|
| Dash | A dash (see The player): back in 8 s, then sooner and sooner; a second dash at level 3 |

## Legendaries

- A level-up offers a **legendary** **5%** of the time, while one is left to find: on a **golden card**, in place of its last choice. Each level-up without one raises the chance by a point (6%, 7%…), back to 5% once one is offered.
- A legendary has **one level**, and takes no room among the passives; once found, it is never offered again. Its card says PASSIVE on its top border, "takes no slot" in its text. It shows in the gear with no level written.

| Legendary | What it does |
|---|---|
| Illumination | **+1 XP every second**, of itself |

## Progress

- **Achievements**: goals reached once, kept from one run to the next, in both modes; each unlocks a card, dealt from then on, or a reroll more a run. At first, the cards dealt are four weapons, the **crowbar**, the **machete**, the **pistol** and the **shotgun**; Vitality, Swiftness, Magnet, Flashlight, Wisdom, Loot; and the heal.

| Achievement | Goal | Unlocks |
|---|---|---|
| Hunter | Wipe out a horde to the last | Horde Hunter |
| First drops | Hold out until the drizzle | Haste |
| Carnage | 300 enemies down in a single run | Might |
| Rain of lead | Hold out until the rain | Machine gun |
| In the eye | Hold out until the storm | Molotov |
| Struck | Struck by lightning, and standing | Dash |
| A thousand down | 1,000 enemies down, in all (every run counted) | Regeneration |
| Raw light | 50 Shadows down, in all (every run counted) | Glare |
| Dawn | Hold out until dawn | Illumination |
| A regular | 25 runs played | A second reroll a run |
| Come rain or shine | Every weather seen once, every run counted, even one lost in it | The arcade mode (legendary) |

- **Told as reached**: a box rises on the bottom right of the map, the picture of what it unlocks and the achievement's name, in its **rarity's** colour (the rarity of what it unlocks: white, blue for rare, gold for legendary; a reroll, rare); it stays 4 s, then goes down. Several, one after the other. What it unlocks is dealt from the next run on: a run's cards are set as it starts.
- The **drones** are dealt no more, until they are made worth taking.
- The **collection**, on the title screen in standard mode, where the arcade shows its leaderboard: every card of a build and the reroll, a tile each: those there from the start, then those unlocked, from the easiest to the hardest (Horde Hunter, Haste, Might, machine gun, Molotov, Dash, the reroll, Glare, Regeneration, Illumination), and last the arcade mode, its tile a joystick; in colour once the player has it, grey until then. Neither the drones nor the heal. Hidden at first, its key at the bottom of the screen, by the mode's ([C] COLLECTION 16 / 21): **C** shows or hides it, shown at its first tile, the arrows going from tile to tile, Esc hiding it too; **Tab** changes the mode. The tile picked is framed. A box under the tiles tells it: its name in its rarity's colour, its kind on the right (weapon, passive, the legendary one too, action, bonus for the reroll, mode for the arcade); available from the start, or the achievement unlocking it, its goal and, for those counted over every run, a row apart, in capitals in the box's middle, **Unlocked** in green or **Locked** in grey; for those counted over every run and not yet reached, a bar as the XP bar instead, across the box, the count in its middle. The box as wide for every tile. Two achievements are **hidden** until reached, the dawn's and the arcade's: their tiles a pixel question mark, their box saying HIDDEN ACHIEVEMENT in its middle, nothing more. Short of room, only the picked tile's row shows, arrows on its sides where more tiles lie.
- The progress is kept in `progress.ron`, in the game's data folder.

## Experience

- Fallen enemies may leave an XP gem (see Enemies). A gem covers one tile. A gem within **2 tiles** (more with Magnet) flies to the player (12 tiles/s) and keeps on until picked up.
- A gem shows by its worth: **silver** for a point, **blue** for 2 to 4 (a Shadow's), **gold** for 5 and more (an abomination's). Each glows its colour around it in the dark.
- At most **500 gems** lie around: past that, the oldest go.
- The next level takes **5 + 3 × (level − 1)** XP: 5 from level 1, 8 from level 2, 11 from level 3. XP beyond it carries over.
- **Rarity**: each card is **common** (grey frame) or **rare** (blue frame, its word on the bottom border), the legendaries apart (see Legendaries). Common: every weapon, Vitality, Swiftness, Magnet, Flashlight, Loot, Dash, the heal. Rare: Might, Haste, Wisdom, Regeneration, Glare. A weapon or a passive keeps its rarity at every level.
- **Level-up**: the game pauses and offers **3 different choices** at random among what can still be picked, each a rare one **25%** of the time (a common one otherwise, or the other if none is left of its rarity): a new weapon or passive (while there is room), or the next level of one owned. At dusk, neither Flashlight nor Glare is offered: the torch isn't lit yet. With nothing left to pick, the one choice is **Heal 20 HP**. Several levels at once offer their choices one after the other.
- **Kind**: a card's top border says whether it is a weapon or a passive, by its number.
- **Room and reroll**: under the cards, the room left: weapons and passives the player has, out of 4 each. Once a run, **R** deals the cards on offer again, as rare as they were; the rerolls left show under the cards too.
- **Choosing**: ←/→ (or Q/A and D) then Enter (or Space), or 1, 2, 3 straight away.
