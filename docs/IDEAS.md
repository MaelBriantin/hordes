# hordes — Ideas

Ideas for later, not scheduled yet. What is in the game, or planned for the current step, lives in [RULEBOOK.md](RULEBOOK.md) (the weapons still to come are listed there, marked *planned*).

## v1.0

- **The v1.0 plan, decided 2026-11-28**: a finish line, the list closed; the game is rich enough, what it lacks is finish. In order, then v1.0 is frozen and released:
  1. **The upgrades by rarity**, the priority (see Level-ups): each level of a weapon or a passive dealt common, rare or epic, a stronger step the rarer.
  2. **The hordes readable**, how to be decided by the user: **no arrow on the screen's edge** (removed before, not to come back).
  3. **A small balance pass with the bot**, one was done in v0.10.
  4. **A full run with each hero**, noting what gets in the way, then the bugs found fixed.
  5. **The boss**, the giant armoured zombie (see Enemies), once there is a reference for its look: five looks drawn 2026-11-28, none kept.
- The HUD's HP, more visible, was done in v0.10 (the status line, top left): not to bring up again.
- **Not in v1.0**, for after or never: the abandoned car and the corner of town, the future traveller, the merchant and gold, the mecha, the evolutions, the bounded map, the experimental weapons, Godot.

## Characters

- **v0.10 plan, decided 2026-11-23: each hero's own action**, on Space, theirs once the run reaches **level 3**, its card shown alone ("Hero skill available"), its next levels to them alone. Done: the survivor's **barricade**, the ninja's **dash**, the riot officer's **shield ram**, the demolisher's **C4**. To do, in this order, each with a gallery (icon and effect), then code and tests, a play in Rex, a commit (numbers to tune in play):
  1. **Medic: first-aid kit** (done, dosed 2026-11-23). 15 HP back at once, the medic's own numbers (Healer not counted again); recharge 20 s. The medic lost his 1 HP every 5 s; Healer stays (herbs and the heal). Levels: 2, 20 HP; 3, recharge 15 s; 4, 25 HP; 5, recharge 12 s.
  2. **Exterminator: wall of fire** (done, look F2 burning, F3's embers going out; icon W1). For 1.5 s (shortened 2026-11-23 from 3 s: some 8 tiles), an unbroken line of fire follows the exterminator's path, each stretch burning 5 s: walls of fire drawn around a crowd or across a way. The zombies don't avoid it: they burn crossing it. Recharge 10 s. Levels: 2, walls of 7 s; 3, a 2 s trail; 4, recharge 7 s; 5, walls 2 tiles wide. The flamethrower's level-5 leak goes: +1 tile of reach instead.
  3. **Scout (infiltrator): smoke bomb** (done: look M4, a dense cloud, the scout a grey silhouette in it; icon G1). A cloud 3 tiles round at their feet, 4 s: inside, invisible even with the torch on, the enemies losing track of them. Recharge 15 s. Levels: 2, 5 s; 3, 4 tiles round; 4, recharge 11 s; 5, enemies in the smoke 30% slower.
  4. **Veteran: gatling burst** (done: the gun shown as it fires, the cases falling at their feet, look C2; icon drawn by the user). 2 s of fire straight ahead, the way they face; they may turn during the burst but move at half speed. 10 bullets a second, 8 damage each, through 1 enemy. Recharge 12 s. Levels: 2, 10 damage; 3, a 3 s burst; 4, recharge 9 s; 5, through 3 enemies.
  5. **Special agent: vault** (a nod to RE4; done: look V5, a low quick jump, shadow and speed lines; icon A1). Over the obstacle just ahead (tree, rock, crate), landing behind it, nothing hurting them during the 0.4 s jump; nothing ahead, a short hop of 2 tiles. Recharge 6 s. Levels: 1, an obstacle of one tile; 2, a kick on landing throws the nearby enemies back; 3, over an enemy too; 4, recharge 4 s; 5, up to 3 tiles in a row, obstacles or enemies (a small group, never a whole horde).
- **Rewards decided 2026-11-23**: **Hold the line** unlocks, with the riot officer, the **Eye-gouger** (done: look F3, icon E1) (FR "Éborgneur", first "Arracheur d'œil"), a flashball: a weapon for the crowd rather than another gun shooting ahead: a slow shot every 1.5 s, 30 damage, the enemy thrown back 3 tiles and stunned 1 s, a zombie felled bursting in a spray ("non-lethal"). **Revival** gets a new achievement, say **Close call** (FR "Sur le fil"): fall under 5 HP and hold until dawn. Untouchable and Lightning rod were removed (their rewards are heroes' own actions now).
- **The future traveller** (FR "Voyageur du futur" or "de l'inconnu"), the last hero planned: unlocks the **energy shield** (a passive) and has a **teleporter** (portal maker) as their action. Weapon, perk and achievement to decide.

- **Unlockable characters**, each with a speciality of its own: a starting weapon, different stats. A werewolf could be one of them.
- **Characters by achievement**: a character, its own look and its own starting weapon, unlocked by an achievement fitting it. The Cleaner (every horde of a run wiped out) could bring a cleaner in overalls, starting with the flamethrower.
- **A nod to Dungeon Crawler Carl**: a character called Carl, a look of his own that recalls the book's hero (boxers, a jacket, bare feet) in the few colours at hand; and a card to go with him, a cat thrown that whirls through the enemies, a nod to Princess Donut. A nod, not a copy: our own pixels, our own words.
- **The classic hero, a Rambo**: the hero of the first versions, in his red headband, no longer there from the start but unlocked by an **arcade achievement**, Legend: **25,000 points in one arcade run** (the best on the leaderboard so far, 9,144), a goal for the arcade at last. The same achievement brings the **napalm bombardment** (see Weapons), as Cleaner brings the flamethrower with the exterminator.
- **Found on the map**: a character to unlock is met during a run (a survivor to reach, to rescue); once found, it can be played in the next runs. With the weapons to unlock (see Weapons), the game's progression from one run to the next.

## Enemies

- **v0.11 plan, decided 2026-11-23: new enemies, then a few places of their own.** In order, each with a gallery, then code and tests, a play in Rex, a commit: the **kamikaze** (done: a zombie, look K2, reworked 2026-11-28: its own orange skin, coming at the player as any, blowing on them after 0.6 s, no rush; blast E1, the C4's fireball and 4 fires left burning, a fire as any), the **spitter** (done: a classic zombie, its skin yellow acid moving (green, turquoise, red tried 2026-11-28, not kept), no glow in the dark; spit C2, lobbed over everything, its target blinking on the ground; keeps 8 tiles off, a puddle left; from the rain), the **ram**, the charger in the code (done 2026-11-24, then dropped 2026-11-28: a lone threat charging the player, beside the point of a game about crowds), a **boss** (one a standard run, about the tempest, its HP bar on top; felled, a chest: a legendary, or a free level-up), decided 2026-11-28 to be the **giant armoured zombie**: guessed far off in the dark, slow, hitting hard. Then the places, rare, an arrow on the screen's edge as the healing's: an **abandoned car** to search (2 s by it: XP, healing or a reroll), a **corner of town** (buildings in the way, street lights, some flickering: fought well in, seen from afar). The balance with the bot once all are in. Not in v0.11: the merchant, the mecha, the colour swaps, the treasure carrier, the traveller.
- **Reading the threats in the dark, decided 2026-11-24**: nothing glows for good, the dark stays dark; only an enemy about to act shows over the night, a moment (the kamikaze on the player, the ram scraping the ground; the spitter shows nothing, its glob's target blinking). The hordes, hard to read today even with Horde Hunter: a way to be found with the user, **never an arrow on the screen's edge** (removed before, refused again 2026-11-28).
- **Kamikaze**: a swollen zombie that rushes at the player and blows up on touching them. The blast hurts the enemies around too, so it can be turned against the horde.
- **Parasites**: an enemy (the abomination?) that bursts into a swarm of small parasites when it falls.
- **Treasure carrier**: a chest on legs that runs about the map and **flees** the player instead of chasing them. Felled, it drops XP and/or a free upgrade. Rough tuning: rare (one at a time, every 2–3 minutes), a little slower than the player (5 tiles/s), gone if not caught within 20–30 s, an arrow on the edge of the screen while it is out of sight.
- **Enemy fire**: an enemy that sets things on fire (an incendiary, a zombie in flames spreading it): fire that burns the player too, a danger to keep away from. The player's own Molotov never burns them.
- **Colour swaps**: zombies in other colours, each with a way of its own, as cheap as the ghouls of the hordes: a red one, faster and frailer; a green one leaving a pool of acid where it falls; a grey one, armoured, with many HP. They would take the place of part of the abominations in the storm and the tempest, where there are too many of them.

## The map

- **A bounded map**: still drawn from the seed, but large and bounded (say 20 × 20 chunks), with **places set from the start**: the merchant's camp (see Gold), healing, a wrecked mecha. Goals to explore for; the Compass could point to them.
- **Exploration rewarded**, as Megabonk does: the map drawn anew each run, but its size fixed and **a list of things always on it**, never in the same place: a merchant, one or two weapons not unlocked yet (see Weapons), caches. The torch makes it a game of its own: put out to cross unseen, lit to fight. What is found has to be worth the detour, the XP not gathered meanwhile: a rare passive, a reroll, healing, an unlock for the next runs. Biomes, later, on the same frame.
- **An abandoned town**, a place on the map: **street lights**, some flickering, some steady, lighting zones where the player fights well but is seen; **buildings** that block the way and can be searched; an **abandoned car**, a cache in it; the **mecha's wreck** now and then.
- **A real compass**: the view always faces north, so the compass points to a goal rather than to the north. A heading line at the bottom of the screen, the goal chosen among the places found (Tab): its icon, its name, an arrow in 8 ways and its distance (`Merchant ↗ 140 m`); arrows on the edge of the screen only for the places near (30 tiles or so), as the healing's today. The compass itself could be a find, or an unlock: without it, the player explores blind, the night all the stronger.
- **Mecha**: a rare wreck to find on the map. Climbed in, the player strikes their hardest, with weapons of its own (a minigun, rockets), and takes blows far better, for as long as its **fuel** lasts: a gauge, then they are thrown out. Found, the wreck doesn't run yet: **3 parts** lie elsewhere on the map; brought back to it, it repairs while the player stays by it, a gauge filling the while, the horde about.
- **The torch on or off**: a key puts the torch out or lights it again, once the night has fallen. Lit, the player sees around them, and the enemies see them from afar; out, they see next to nothing (the gems, the campfires, the lightning), and the enemies only find them close by (6 tiles or so), wandering otherwise, or drawn to another light. The hordes find the player whatever. Shooting might give their place away a moment.

## Weapons

- **Weapons to unlock**, as the characters: only a few in the cards at first (the crowbar, the pistol, the machete?), the others unlocked for the next runs by **exploring** the map (a cache, a crate, an abandoned car holding the shotgun) or by **challenges** (100 enemies down with the crowbar for the machete; the dawn without lighting the torch again; and so on). Fewer weapons about at first, builds easier to balance.
- **Flamethrower**: from time to time, a jet of flame sweeps around the player, further out than the drones, burning what it passes over.
- **Sniper rifle**: aims at an enemy far off (12–15 tiles, the furthest or the toughest), a red laser dot on it for a moment before the shot; a heavy hit (enough for a Shadow at once), a shot every few seconds; later, the bullet goes through several. Unlike the other guns, it aims at the enemies out of the light, in the dark: it shoots with the torch out too. Unlocked by Shadow, in place of Stealth (decided 2026-11-22).
- **Napalm bombardment**: strikes falling around the player from the sky, each shown a moment before by a mark on the ground (as the lightning's), leaving sheets of fire that set the enemies on fire. Unlocked with the classic hero by the arcade achievement Legend (see Characters). Rather than a Gatling, kept for the machine gun's evolution.
- **Grenade** (done in v0.10, unlocked by Demolisher): thrown at the biggest crowd near, arcing half a second, it blows where it lands, no fire: damage over an area, the enemies caught in it blown outwards, away from its centre, giving the player room.
- **Experimental weapons**, a kind of their own, found in **crates** on the map rather than dealt: a crate guarded by a "capture the hill" goal, the player holding a zone a while against the enemies before it opens.
- **Laser beam** (experimental): a beam across the whole map, hurting every enemy that crosses it.
- **Portal maker** (an action card): two portals laid, the player stepping into one comes out of the other.
- **Energy shield** (a passive): it soaks up the damage taken, then sends it back out in a wave around the player.

## Level-ups

- **Evolutions**: a weapon at level 5 may later be offered a legendary card that transforms it. Drones → Swarm: the drones fire bullets in a spiral around the player. Shotgun → Dragon's breath: the pellets set the enemies they hit on fire (needs fire). Pistol → Akimbo: two pistols, two targets a shot, twice the rounds. Machine gun → Gatling: twice the rate, a 60-round magazine. Machete → Whirlwind: the swing turns all the way round the player.
- **Rare upgrades of common passives**: a rare Vitality that regenerates HP over time, and so on.
- **Upgrades of a rarity, as Megabonk's** (noted 2026-11-23, after v0.10's balance): each level of a weapon or a passive dealt with a rarity of its own, common, rare or epic, its card framed in its colour (the gear's rarity frames already show it): a stronger step the rarer (say +8%, +15%, +25% damage), or a modifier on top (a pierce, one more projectile). Goes with the rare passives above and the evolutions: one legendary a run stays apart from it.

- **Rarity**: some cards rarer than others (common, rare…), shown on the card, so that the best bonuses are a lucky find.
- **Drones that shoot**: the drones as they are do next to nothing, and nobody takes them: a blow now and then, then broken for 2 s. They should shoot around the player, each at the enemies near its side of the circle (the Swarm's spiral of bullets, from the first level?), or go until they do.
- **Rare drones**: other kinds of drones, found on rare cards. Missile drones, for one: circling the player like the drones, each fires a missile in turn, so salvos streak across the screen.

## Healing

- **Herbs** to pick up on the map, as in survival horror: a green one heals a little; maybe mixed with others for more.
- **Campfires**: standing next to one heals little by little, but staying put is dangerous with the horde around, and the fire dies out on its own after a while.
- **Revival** (decided 2026-11-23, unlocked by Close call; done: looks R3 + R4, icon H4; a legendary since, one level): at 0 HP, the hero comes back once with half their HP, 2 s untouchable, a wave throwing the enemies back.

## Gold

- Enemies drop **gold** besides the XP gems, once there is something to spend it on:
  - **boosting upgrades**: paying to raise a card's level further, or to draw new cards on a level-up;
  - a **merchant met on the map**, in a lit spot of his own, selling herbs, weapons and upgrades for the gold gathered.

## Building

- **Materials** to pick up on the map (dropped by enemies, or gathered from trees and rocks), and **barricades** the player places with a key. A barricade blocks like a rock: zombies slide around it, bats fly over, the abomination could break it.

## Runs

- **Trophies**, in the standard mode (which keeps no leaderboard): goals reached in a run, kept from one run to the next. Hold out until dawn; wipe out every horde of a run; reach dawn without lighting the torch again once it is out; and so on.

- Show the run's **seed** on the game over, to play the same map again.

## Light

The game's strength: a black night, lit here and there. The rule: **a light makes a halo**, around it, on what is there; what isn't a light doesn't glow.

- **Lightning puts the torch out**: a strike near the player puts their torch out a moment, longer or shorter as luck has it, then it flickers back on of itself. The night all at once, the horde about.
- **Shooting in the dark**: the guns fire on with the torch out, each shot flashing at the gun's mouth, a small halo of light a blink, which the enemies see and come to. Better in Godot, with finer lights and shadows.
- Tried, and dropped: **muzzle flashes** in the terminal, a few pixels at the gun's mouth, with a light and a halo, then without: too many lights, and ugly at this size.
- Tried, and dropped: **glowing bullets**, a light and a halo each: the torch lights enough, and they aren't lights.
- Tried, and dropped (2026-11-24): **shadows** in the terminal, a blob under the feet or the figure's shape laid on the ground away from the torch (or the campfire): ugly at this size. Kept for Godot, its 2D lights casting them.
- Tried, and dropped: **glowing ghouls**, the horde's skin lit green with a halo. Together, their halos melt into a green fog, harder to read than before.

## Interface

- **The HUD redone, our own pixels** (noted 2026-11-23, after v0.10's actions): stop leaning on Nerd Font glyphs; draw as many icons and pictures as we can in pixels ourselves. Make the **HP** stand out more. Each hero having their own action now, give it **a bar of its own**, out of the classic cards' row, its recharge on it. Maybe the **hero's portrait in a diamond** beside the HP bar, the special action set apart from the rest.

## Engine

- **Godot, one day**: the game in Godot rather than the terminal. The rules in `src/game` stay as they are, pure Rust, as a Godot extension (godot-rust's gdext); only the screen is made again, in scenes, shaders and Godot's 2D lights, which would draw the night, the torch and the halos better. The sprites of `assets/sprites.txt` turn into PNGs with a script. Until then, no rule of the game goes into `src/ui`, so the move stays a matter of drawing.
