# Changelog

What each version brought, for the player. The rules themselves, with their numbers, live in [docs/RULEBOOK.md](docs/RULEBOOK.md).

Each version's lines go under New (what didn't exist), Changed (what works otherwise, balance included) and Fixed (bugs), in that order, only those that have lines.

Numbers: a minor version (0.2.0) for new things to play with; a patch (0.1.1) for a fix or a balance change alone.

## Unreleased
Written as the work goes, under the same New, Changed and Fixed; it becomes the next version's section when it is released.

### New
- A strike of lightning falling with the player in its ring puts their torch out, up to them to light it again (F): the night all at once, the guns holding their fire. It no longer hurts the player.
- The Barricade, a second action card, unlocked by a new achievement, Untouchable (2 minutes without a hit after nightfall): Space or Enter heaps three crates just ahead, stopping the walking enemies, which batter it until it breaks; the Shadows fly over it, the shots go over it. The next in 8 s, two at most at once; more HP, a third, and unbreakable metal crates at the last level.

- The flamethrower, a rare weapon, unlocked by a new achievement, Cleaner (every horde of a run wiped out): a cone of fire ahead of the player setting every enemy in it on fire, its tank filling again once empty; at the last level, the tank leaks a trail of fire behind the player.
- The rookie, a new hero (fair hair, a blue uniform), unlocked by a new achievement, Cold blood (a minute under 30% of HP in one run), with a weapon of their own, the combat knife: quick stabs at the nearest enemy, whichever way.
- The demolisher, a new hero (bald, bearded, a grey tank top and jeans), unlocked by a new achievement, Demolisher (10 abominations felled in melee in one run), with a weapon of their own, the sledgehammer: a blow on the ground ahead, its shock hurting and throwing back every enemy around; stunning them at the last level.
- The exterminator, a skin for the hero once Cleaner is reached: a yellow hazmat suit, a dark grey gas mask and boots, a red tank on the back.
- The skins on the title screen, S showing them as the collection shows its cards: the hero picked turning round, the locked ones in grey with the achievement that unlocks them, Enter to wear one.

### Changed
- Each hero plays with their own HP, speed and perk, shown in the skin picker: the survivor versatile, the classic hero a commando (+10% damage), the exterminator a pyromaniac (fire burns twice as long), the demolisher unshakable (140 HP, slower, less damage taken, never thrown), the rookie quick (80 HP, faster, guns reloading 20% faster).
- Each hero starts a run with a melee weapon of their own, theirs alone, never dealt to another hero, as the skin picker tells: the survivor the crowbar, the classic hero the machete, the exterminator a new stun baton (a thrust ahead, paralysing what it hits), the demolisher the sledgehammer. The crowbar and the machete leave the common cards and the collection.
- The arcade mode shown as Endless Night, and opening on the night, the dusk skipped; a standard run always opens on the dusk, the setting to skip it gone.
- Low on HP, under 30%, the player's heart beats on the edges of the screen, faster the lower; the HP over the hero blinks too.
- The torch no longer lights up of itself as the night falls: up to the player, F shown under them until they first do in the run.
- The skin chosen on the title screen alone (S), no longer in the options.
- The arcade mode unlocked by reaching the dawn, the game won, rather than by seeing every weather; Illumination by a new achievement, Veteran (level 25 in one run). Come rain or shine (every weather seen) is gone, a progress holding it still read.
- The title screen in the middle of the screen when nothing shows under it, rather than over an empty room kept for the collection: it slides as the collection or the skins open and close, and as Tab changes the mode.
- Trees and rocks stop the shots: the guns and the drones aim only at enemies in sight, and a bullet meeting an obstacle stops there, in a spark; a Shadow over a rock can't be shot.
- The drones reworked, dealt again once unlocked by a new achievement, Swarm (2 hordes wiped out in one run): one at first, each shooting at the nearest enemy within 6 tiles, a bullet every 0.4 s, but in the dark; an enemy touching one takes a blow from it and breaks it for 3 s (2 at level 4). More drones at levels 2, 4 and 5, more damage at level 3.

### Fixed
- A gauge under an icon went wrong a moment (and the game could crash, built for debugging) when a level-up shortened a wait under way: Haste during a reload or between two swings.

## v0.8.2 · November 21, 2026
Wind-up.

### Changed
- Every weapon shows its wait under its icon, not the guns alone: the crowbar and the machete between two swings, the Molotov between two throws, the drones while one is broken; the gauge filling, full once ready.

## v0.8.1 · November 21, 2026
Fine print.

### Changed
- Horde Hunter says its enemies targeted rather than veiled, and targets less of a horde: a tenth, a quarter, the targets seen in the dark at level 3, a half, the whole horde at the last (rather than three quarters at level 4).

### Fixed
- Card texts too long for their card, wrapping out of it: Horde Hunter's, Glare's, Magnet's, the shotgun's last level, made shorter.

## v0.8.0 · November 21, 2026
Hunted.

### New
Progress:
- Achievements, reached once and kept from one run to the next, each unlocking a card, a reroll or the arcade mode, from the next run on; told as reached, a box rising on the bottom right of the map, in the rarity's colour. At first, the crowbar, the machete, the pistol, the shotgun, and Vitality, Swiftness, Magnet, Flashlight, Wisdom and Loot alone are dealt.
- The first and easiest, Hunter, a horde wiped out to the last, unlocks a new passive, Horde Hunter: +10% damage to the hordes per level, and their last ones veiled in red, more with the levels, the whole horde at the last; from level 3, seen in the dark too.
- Haste and Might are unlocked too: Haste by First drops, the drizzle reached; Might by Carnage, 300 enemies down in a single run.
- The arcade mode is unlocked by an achievement too, Come rain or shine: every weather seen once, every run counted; until then, the standard alone.
- Hidden achievements, the dawn's and the arcade's: in the collection a pixel question mark and "Hidden achievement" until reached.
- The collection, on the title screen in standard mode: every card, the reroll and the arcade, in colour once had, grey until then, from the easiest to unlock to the hardest; the arrows go over them, the one picked framed, a box telling it: available from the start, or the achievement that unlocks it and how far along it is.

Menace:
- An abomination felled swells, blinking white, for 0.6 s, then bursts: a disc of green bile out to 3 tiles, costing everything it reaches 10 HP, the player and the enemies alike, and pushing it back 2 tiles. Out of it come 4 parasites, small green worms hopping, fast (12 HP, 4.5 tiles/s, 3 damage), that always find the player; worth a point and no gem.

Options:
- Erase the progress, from the title screen's options, after a confirmation.

Cards:
- Action cards, a third kind beside the weapons and the passives: played with Space or Enter; one a run. The Dash is the first.

### Changed
- The title screen's and the death screen's prompts, each key as a cap (Press [Space] to start); on the death screen, steady, R for a new run, Esc back to the main menu rather than quitting.
- A darker night: what the light doesn't reach keeps 10% of its brightness rather than 14%.
- A strike of lightning sets on fire the enemies it hurts.
- The line under the big letters (a weather, a horde, the dawn) in white capitals.
- The title screen: the mode is the game's subtitle, in small pixel letters, Survive the night in the night's grey or Arcade in gleaming gold; Tab changes it, ←→ no more; the title stays in place from one mode to the other.
- A horde comes out of the other side from the enemies about the player, catching them between, rather than a side at random.
- Tougher enemies: a zombie has 15 HP rather than 10, a Shadow 40 rather than 25; and the weather toughens them more, +15% in the drizzle, +30% in the rain, +50% in the storm, +100% in the tempest (rather than none, +10%, +25%, +50%).
- The drones are dealt no more, until they are made worth taking.
- Shadows hunt faster the longer they hunt in the dark, up to 50% after 5 seconds, and bolt out of the torch's light as it catches them: lighting it at the last moment no longer leaves them to be shot.

### Fixed
- A game left open while `hordes reset` erases the progress no longer writes it back: it forgets it too.
- The arrow over the player, and their HP bar, show over everything on the map: gems, rain, lightning; they went under the gems.
- A gun's gauge empties as its last round goes, its reload filling back from nothing, rather than starting again at once from a pixel.
- The night's announcement no longer names a key for the torch: it said Space, the torch long on F.

## v0.7.0 · November 20, 2026
In a flash.

### New
- A reroll: once a run, R on a level-up deals its cards again.
- Under the level-up's cards, the room left: weapons and passives out of 4 each.
- A card says whether it is a weapon or a passive, on its top border, by its number.
- A new passive, Dash: Shift dashes, 3 tiles in a blink, nothing hurting the hero meanwhile, fading silhouettes of them left behind. Slow to come back at first (8 s), sooner with the levels (3 s at the last); a second dash at level 3, no more. The gear shows them under its icon, as a gun's rounds. The torch is F alone now.
- A new rare passive, Glare: the enemies in the torch's light go 10% slower per level, while it is lit.
- A Molotov bottle bursts in a flash of fire, lighting all around a moment.

### Changed
- The torch out, an enemy loses the player past 8 tiles rather than 12: putting it out lets the player slip away.
- The herbs are picked up within 2 tiles, rather than walking right over them.
- The machine gun shoots short bursts: 8 rounds at level 1, a shot every 0.1 s, then a 2.5-second reload, rather than 20 rounds; 12 from level 2, 16 at level 5, the reload 1.8 s from level 4. About a quarter less damage at first, as strong as before at the top.
- In the gear, each card's level is written on its icon's corner, MAX at the last, rather than as a row of pixels under it.
- The gear can show as a list, a setting: each card's name, its level, its gauge.
- The gear shows the cards in the order they were picked, weapons and passives mixed.
- From the storm on, lightning hurts the player too: a blinking ring shows where it will strike half a second before it does, often where the player stands; caught in it, they lose 20 HP.
- The torch has no battery any more: lit or out as the player wants, it is a choice, seen or unseen, rather than a gauge to watch. The Batteries card and the Shadows' energy cells go with it; the bar over the hero shows the HP alone.
- `hordes update` updates the game where it is, rather than offering to move one installed in `~/.cargo/bin`: such a copy stays there, as uv does; to have it in `~/.local/bin`, uninstall it and install it again.
- A horde's side shows as a red glow on the edge of the screen, pulsing slowly, rather than a band of arrows.
- The Compass card goes: healing is found well enough without it, and nobody took it.
- Two hordes standing at once are counted apart, each by its number, rather than together; the one wiped out is told with its number.
- Fire lights further, the puddles of fire and the enemies burning half again as far, and all that burns casts a flickering orange halo, the campfires included.

## v0.6.1 · November 19, 2026
A new home.

### Changed
- The game installs in `~/.local/bin` now, rather than `~/.cargo/bin`, a Rust programmer's folder; `hordes update` offers to move a game installed in the old place.

## v0.6.0 · November 19, 2026
Fire, and a crowbar.

### New
- The crowbar, the weapon every run starts with now: a heavy blow, slower than the machete's, pushing back what it hits. The machete is a card.
- A new weapon, the Molotov: bottles thrown at the enemies nearby, bursting into puddles of fire.
- Fire: an enemy set on fire burns a while, 6 HP in all, flames over its head, lighting around it, and leaves ash once burnt to death.
- A new rare passive, Regeneration: HP back of themselves, every second.
- A new weapon, the shotgun: a double shot, each a fan of pellets, close by.
- An arrow bouncing over the player's head, to see them at once in the crowd: always, in a colour of the Arrow setting, or for 2 s on E.
- Small HP and battery bars over the player's head: a setting, off at first.
- A coloured ground, dark grass speckled with earth: an experimental setting, off at first. It redraws most of the screen at every step: best turned on in a modern, fast terminal (Ghostty, kitty, WezTerm…), rather than over SSH or tmux.

### Changed
- The options start otherwise: the gear hidden, the announcements big, with no pause.
- The arrow takes the place of the line around the player: the Outline setting is the Arrow setting now, its colour kept.
- The gems take the cards' colours: silver for a point, blue, then gold for the most; the XP bar turns a bluish silver, the energy cells and the battery's gauge green.
- The hero is a survivor now, with very dark brown hair, a leather jacket and jeans. The hero of the first versions, in a red headband, can still be chosen: Hero, in the options.
- Lighting up, the torch flickers for as long as luck has it, lighting nothing for the run until it holds; it goes out at once.
- A campfire's light is warm and sways, told from the torch's at a glance; the fire heals as far as it shows, fading included; a heal shows, small green crosses rising around the player.
- A horde wiped out brings a level, once told, its cards rarer; in arcade, it doubles its points too.

## v0.5.0 · November 18, 2026
Light is a choice.

### New
- Two modes, chosen on the title screen: standard, from dusk to dawn, with no score; arcade, endless, the weather going up to the tempest and back down, again and again, 7% harder a minute, for the score and its leaderboard. The pause menu leads back to the title screen.
- The dusk can be skipped (C on the title screen): the run opens on the night.
- ? lists every key during a run; I shows or hides the gear.
- An options menu, O on the title screen or from the pause menu, holds every setting: the title screen keeps only the mode, Enter and O.
- How new weathers and hordes are told, an option: big with a pause, as before; big with no pause; or on a banner on the top of the map, the run going on. The gear shows or hides from the options too.
- In arcade, the score shows on a gold badge, its thousands apart.
- The torch on or off, with Space or F: it lights up of itself as the night falls. Out, the player sees next to nothing, but the enemies only find them close by, wandering or drawn to the campfires otherwise; a strike of lightning gives them away. The guns hold their fire in the dark.
- The torch's battery: two minutes lit, a minute to charge back out; empty, the torch dies out. A new rare passive, Batteries, makes it last longer and charge faster; a fallen Shadow leaves an energy cell that charges it.
- Gems by their worth: blue, green, red, each glowing its colour. A new passive, Loot, makes them likelier.
- A count of the horde still standing, on the top of the map.
- The Shadows, in place of the bats: hunched, barely seen in the dark but for their red eyes, passing through obstacles, faster and far tougher than the bats were. They go the other way round: they hunt in the dark, and shun the torch.

### Changed
- The machine gun is weaker, it did the work alone: 3 damage (5 from level 3) rather than 4 (6), a shot every 0.17 s rather than 0.13: about 30% less damage a second.
- The machine gun shoots thin tracers, their trail fading behind them: told from the pistol's bullets at a glance.
- Easier: every weapon is a common card again, the rare ones are the strongest passives; a quarter fewer enemies appear, and each horde brings 10 more than the last rather than 15.
- The HP show in the middle of their bar, as the level in the XP bar.
- A horde wiped out is told in big gold letters; its arrows make a blinking band on the edge it comes from; the compass's arrow comes with a healing cross: all seen at a glance.

## v0.4.0 · November 18, 2026
Rare finds in a sharper night.

### New
- A new passive, Wisdom: +10% XP from the gems per level.
- Legendaries, on golden cards that come now and then, more and more likely the longer without one: Illumination, the first, gives an XP point every second.
- Rare cards, in a blue frame, a time in four: the machine gun, the drones, Might, Haste, Compass and Wisdom.
- An outline around the player, to see them at once in the crowd: off, white, yellow or red, with O on the title screen or from the pause menu.
- A blow to the player spurts blood (as the settings say) and flushes the edges of the screen red for a moment.
- The title screen and the pause menu show the game's version.

### Changed
- The XP gems cast a small blue glow around them in the dark, and light the enemies passing by: they light the way rather than glow alone.
- The menus, their titles and the cards' names are in capitals.
- The dusk lasts half a minute, just enough to look around, dimming little by little, and the night falls all of a sudden, in its last 6 seconds. The rest of the run comes 30 seconds sooner: the hordes at 1:30, 3:30…, the dawn at 10:30.
- Lightning waits for the storm: the drizzle and the rain only rain.
- The clear night is the night, plainly. The torch is out at dusk: it lights up once the night has been told, flickering on.
- The level shows in the middle of the XP bar, without an icon before it.
- The HP and XP bars stand in relief, their top lighter.
- The weather shows as a row of every weather's icon, the dawn's last, the one now lit, the next one coming out of the dark as the stage goes, rather than as a gauge.
- The selected card stands out bright in its own colour, the others dimmer: a rare one stays blue, a legendary one gold.
- A passive's card shows the total it brings the player to, not only what it adds.

## v0.3.0 · November 18, 2026
Hold out until dawn.

### New
- The dawn: hold out until 11:00, the sun rising over the last minute as the rain stops, and the enemies flee the risen sun and burn; the run is won, with a bonus of half its score. A sun marks those runs on the leaderboard.
- Each new weather and each horde is told in big letters over the map, the world standing still for 2 seconds; the run opens on the dusk, told the same way.

### Changed
- Flashlight is no longer offered at dusk, while the sun still lights the map.
- The dusk lasts a minute rather than two: half of it in sunlight, half as the night falls. The weather comes a minute sooner.
- A horde comes in the middle of each stage of weather, once the night has fallen: at 2:00, then every 2 minutes, 15 enemies bigger each time.
- How many enemies appear, and of which kinds, follows the weather rather than the clock: zombies alone at dusk, abominations from the drizzle. Bats come less often, in the hordes too.

## v0.2.0 · November 18, 2026
The hordes come at dusk.

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
