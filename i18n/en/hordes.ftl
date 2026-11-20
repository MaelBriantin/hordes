### The texts of hordes, in English: every line the player reads. Other languages translate
### these messages, in a folder of their own; what they lack shows in English.

## Keys, on the last line of the screen

help-title = Enter start · ←→ mode · O / Esc options
help-options = ↑↓ choose · ←→ / Enter change · Esc back
help-paused = ↑↓ choose · Enter confirm · ←→ change · P / Esc resume
help-over = R new run · Esc quit
help-level-up = ←→ / 1 2 3 choose · Enter take · Esc pause

## Title screen

title-tagline = Survive the night.
title-start = Press Enter to start
mode-standard = Standard: hold out until dawn
mode-arcade = Arcade: endless, for the score
dusk-played = played
dusk-skipped = skipped

## Settings

setting-off = off
blood-red = red
blood-ketchup = ketchup
blood-mayo = mayo
blood-pesto = pesto
rain-full = full
rain-reduced = reduced
rain-off = off

announcements-paused = big, with a pause
announcements-big = big, no pause
announcements-banner = banner

arrow-off = off
arrow-white = white
arrow-yellow = yellow
arrow-red = red

## Pause menu

pause-title = Paused
pause-resume = Resume
pause-blood = Blood: { $value }
pause-rain = Rain: { $value }
option-arrow = Arrow: { $value }
pause-announcements = Announcements: { $value }
options-title = Options
option-dusk = Dusk: { $value }
option-gear = Gear: { $value }
option-skin = Hero: { $value }
skin-survivor = survivor
skin-classic = classic
gear-icons = icons
gear-list = list
gear-hidden = hidden
option-mini-bars = HP over the hero: { $value }
mini-bars-shown = shown
mini-bars-hidden = hidden
option-ground = Coloured ground (experimental): { $value }
ground-coloured = on
ground-plain = off
option-back = Back
pause-language = Language: { $value }
pause-new-run = New run
pause-title-screen = Main menu
pause-quit = Quit

## Status line

weather-dusk = Dusk
announce-dusk = Hold out until dawn
announce-night = The night has fallen · Space: torch
announce-worse = The weather worsens
announce-last = Dawn in two minutes
weather-clear = Night
weather-drizzle = Drizzle
weather-rain = Rain
weather-storm = Storm
weather-tempest = Tempest

## Game over

dawn-title = DAWN
dawn-line = Congratulations, you survived. This time.
death-title = DEAD
death-again = Press R for a new run
over-survived = You survived { $time }
over-score = Level { $level } · { $kills } kills

## Level-up

level-up-title = Level { $level }
card-weapon = WEAPON
card-passive = PASSIVE
level-up-room = Weapons { $weapons }/{ $most_weapons } · Passives { $passives }/{ $most_passives }
level-up-reroll = R: deal again ({ $left })
card-new = NEW
card-rare = RARE
card-legendary = LEGENDARY
card-level = Lv { $level }/{ $max }

weapon-crowbar = Crowbar
weapon-machete = Machete
weapon-pistol = Pistol
weapon-machine-gun = Machine gun
weapon-shotgun = Shotgun
weapon-molotov = Molotov
weapon-drones = Drones

passive-vitality = Vitality
passive-swiftness = Swiftness
passive-magnet = Magnet
passive-might = Might
passive-haste = Haste
passive-flashlight = Flashlight
passive-regeneration = Regeneration
passive-glare = Glare
passive-dash = Dash
passive-loot = Loot
passive-wisdom = Wisdom
legendary-illumination = Illumination
heal = Heal

gain-crowbar-1 = A heavy blow that pushes back
gain-crowbar-2 = A wider arc
gain-crowbar-3 = Damage 18 → 24
gain-crowbar-4 = A backhand behind you
gain-crowbar-5 = Stuns what it hits
gain-machete-1 = Slashes enemies in front
gain-machete-2 = Reach 2.5 → 3.5
gain-machete-3 = A second slash, behind
gain-machete-4 = Wider swing
gain-machete-5 = A third slash, ahead
gain-pistol-1 = 6 rounds at the nearest enemies
gain-pistol-2 = Magazine 6 → 9
gain-pistol-3 = Reloads in 1.5 s
gain-pistol-4 = +1 bullet per shot
gain-pistol-5 = Magazine 12, damage 15
gain-machine-gun-1 = Bursts of 8 rounds
gain-machine-gun-2 = Bursts of 8 → 12 rounds
gain-machine-gun-3 = Damage 3 → 5
gain-machine-gun-4 = Reloads in 1.8 s
gain-machine-gun-5 = Bursts of 16, bullets through an enemy
gain-shotgun-1 = A fan of 5 pellets, close by
gain-shotgun-2 = 7 pellets a shot
gain-shotgun-3 = Damage 6 → 8
gain-shotgun-4 = Reload in 1.5 s
gain-shotgun-5 = Pellets pierce one enemy, a wider fan
gain-molotov-1 = A bottle of fire, every 3 s
gain-molotov-2 = Wider puddles
gain-molotov-3 = 2 bottles a throw
gain-molotov-4 = Puddles burn 5 s
gain-molotov-5 = A throw every 2 s
gain-drones-1 = A drone circling you
gain-drones-more = +1 drone
gain-drones-3 = Damage 8 → 12
gain-drones-5 = Wider and faster
gain-vitality = +20 max HP, healed (total { $total })
gain-swiftness = +8% speed (total +{ $total }%)
gain-magnet = Gems fly in from 1 tile further (total { $total })
gain-might = +10% damage (total +{ $total }%)
gain-haste = −8% weapon cooldowns (total −{ $total }%)
gain-flashlight = +2 tiles of light (total { $total })
gain-regeneration = +0.4 HP a second (total { $whole }.{ $tenth })
gain-glare = -10% speed for the enemies in the light (total -{ $total }%)
gain-dash = Shift: a dash, nothing hurts meanwhile; back in 8 s
gain-dash-second = A second dash
gain-dash-sooner = The dash back in { $tenth ->
    [0] { $whole } s
   *[other] { $whole }.{ $tenth } s
}
gain-loot = +10% chance of a gem (total +{ $total }%)
gain-wisdom = +10% XP from gems (total +{ $total }%)
gain-illumination = +1 XP every second
gain-heal = Heal 20 HP

## Score

multiplier = ×{ $whole }.{ $tenth }
over-points = { $score } points
leaderboard-title = Best scores
name-title = New high score!
name-place = Number { $place } on the leaderboard!
name-prompt = Your name:
help-name = ←→ move · ↑↓ or a letter · Enter confirm · Esc skip

## Hordes

horde-title = HORDE
horde-coming = It is coming!
horde-everywhere = They come from everywhere!
horde-left = HORDE · { $left }
horde-left-of = HORDE { $number } · { $left }
horde-number = Horde { $number }
horde-cleared = HORDE WIPED OUT
horde-cleared-points = +{ $points } points · +1 level
horde-cleared-level = +1 level

## Keys

keys-title = Keys
keys-move = Arrows / WASD / ZQSD   move
keys-torch = F                      torch on or off
keys-dash = Shift                  dash (its card): nothing hurts
keys-cards = ← → / 1 2 3 · Enter    take a card (R: deal again)
keys-gear = I                      show or hide the gear
keys-ping = E                      an arrow over the hero, 2 s
keys-pause = P / Esc                pause
keys-keys = ?                      these keys

## Achievements: told as reached, a box on the bottom right of the map.
achievement-new = NEW ACHIEVEMENT!
achievement-new-rare = NEW RARE ACHIEVEMENT!
achievement-new-legendary = NEW LEGENDARY ACHIEVEMENT!
achievement-unlocked = Unlocked:
achievement-reroll = a reroll more a run
achievement-rain = Rain of lead
achievement-rain-goal = Hold out until the rain
achievement-storm = In the eye
achievement-storm-goal = Hold out until the storm
achievement-struck = Lightning rod
achievement-struck-goal = Struck by lightning, and standing
achievement-thousand = A thousand down
achievement-thousand-goal = 1,000 enemies down, every run counted
achievement-shadows = Raw light
achievement-shadows-goal = 50 Shadows down, every run counted
achievement-dawn = Dawn
achievement-dawn-goal = Hold out until dawn
achievement-regular = A regular
achievement-regular-goal = Play 10 runs
