### Les textes de hordes en français. Ce qui manque ici s'affiche en anglais (voir `i18n/en/`,
### qui a toutes les clés).

## Touches, sur la dernière ligne de l'écran

help-title = Entrée jouer · ←→ mode · O / Échap options
help-options = ↑↓ choisir · ←→ / Entrée changer · Échap retour
help-paused = ↑↓ choisir · Entrée valider · ←→ changer · P / Échap reprendre
help-over = R nouvelle partie · Échap quitter
help-level-up = ←→ / 1 2 3 choisir · Entrée prendre · Échap pause

## Écran titre

title-tagline = Survis à la nuit.
title-start = Appuie sur Entrée pour jouer
mode-standard = Standard : tenir jusqu'à l'aube
mode-arcade = Arcade : sans fin, pour le score
dusk-played = joué
dusk-skipped = passé

## Réglages

setting-off = non
blood-red = rouge
blood-ketchup = ketchup
blood-mayo = mayo
blood-pesto = pesto
rain-full = forte
rain-reduced = réduite
rain-off = aucune

announcements-paused = gros, avec pause
announcements-big = gros, sans pause
announcements-banner = bandeau

arrow-off = aucune
arrow-white = blanche
arrow-yellow = jaune
arrow-red = rouge

## Menu pause

pause-title = Pause
pause-resume = Reprendre
pause-blood = Sang : { $value }
pause-rain = Pluie : { $value }
option-arrow = Flèche : { $value }
pause-announcements = Annonces : { $value }
options-title = Options
option-dusk = Crépuscule : { $value }
option-gear = Équipement : { $value }
option-skin = Héros : { $value }
skin-survivor = survivant
skin-classic = classique
gear-icons = icônes
gear-list = liste
gear-hidden = masqué
option-mini-bars = Vie sur le héros : { $value }
mini-bars-shown = affichée
mini-bars-hidden = masquée
option-ground = Sol coloré (expérimental) : { $value }
ground-coloured = activé
ground-plain = désactivé
option-back = Retour
pause-language = Langue : { $value }
pause-new-run = Nouvelle partie
pause-title-screen = Menu principal
pause-quit = Quitter

## Ligne d'état

weather-dusk = Crépuscule
announce-dusk = Tiens jusqu'à l'aube
announce-night = La nuit est tombée · Espace : lampe
announce-worse = Le temps se gâte
announce-last = L'aube dans deux minutes
weather-clear = Nuit
weather-drizzle = Bruine
weather-rain = Pluie
weather-storm = Orage
weather-tempest = Tempête

## Fin de partie

dawn-title = AUBE
dawn-line = Bravo, tu as survécu. Cette fois.
death-title = MORT
death-again = Appuie sur R pour rejouer
over-survived = Tu as survécu { $time }
over-score = Niveau { $level } · { $kills } tués

## Montée de niveau

level-up-title = Niveau { $level }
level-up-room = Armes { $weapons }/{ $most_weapons } · Passifs { $passives }/{ $most_passives }
card-weapon = ARME
card-passive = PASSIF
level-up-reroll = R : relancer ({ $left })
card-new = NOUVEAU
card-rare = RARE
card-legendary = LÉGENDAIRE
card-level = Niv { $level }/{ $max }

weapon-crowbar = Pied-de-biche
weapon-machete = Machette
weapon-pistol = Pistolet
weapon-machine-gun = Mitrailleuse
weapon-shotgun = Fusil à pompe
weapon-molotov = Molotov
weapon-drones = Drones

passive-vitality = Vitalité
passive-swiftness = Vitesse
passive-magnet = Aimant
passive-might = Force
passive-haste = Cadence
passive-flashlight = Lampe torche
passive-regeneration = Régénération
passive-glare = Éblouissement
passive-dash = Dash
passive-loot = Butin
passive-wisdom = Savoir
legendary-illumination = Illumination
heal = Soin

gain-crowbar-1 = Un coup lourd qui repousse
gain-crowbar-2 = Arc plus large
gain-crowbar-3 = Dégâts 18 → 24
gain-crowbar-4 = Un revers derrière toi
gain-crowbar-5 = Étourdit ce qu'il frappe
gain-machete-1 = Tranche les ennemis devant toi
gain-machete-2 = Portée 2,5 → 3,5
gain-machete-3 = Un deuxième coup, derrière
gain-machete-4 = Coup plus large
gain-machete-5 = Un troisième coup, devant
gain-pistol-1 = 6 balles sur les ennemis proches
gain-pistol-2 = Chargeur 6 → 9
gain-pistol-3 = Recharge en 1,5 s
gain-pistol-4 = +1 balle par tir
gain-pistol-5 = Chargeur 12, dégâts 15
gain-machine-gun-1 = Rafales de 8 balles
gain-machine-gun-2 = Rafales de 8 → 12 balles
gain-machine-gun-3 = Dégâts 3 → 5
gain-machine-gun-4 = Recharge en 1,8 s
gain-machine-gun-5 = Rafales de 16, les balles traversent un ennemi
gain-shotgun-1 = Une gerbe de 5 plombs, de près
gain-shotgun-2 = 7 plombs par tir
gain-shotgun-3 = Dégâts 6 → 8
gain-shotgun-4 = Recharge en 1,5 s
gain-shotgun-5 = Les plombs traversent un ennemi, gerbe plus large
gain-molotov-1 = Une bouteille de feu, toutes les 3 s
gain-molotov-2 = Flaques plus larges
gain-molotov-3 = 2 bouteilles par lancer
gain-molotov-4 = Les flaques brûlent 5 s
gain-molotov-5 = Un lancer toutes les 2 s
gain-drones-1 = Un drone tourne autour de toi
gain-drones-more = +1 drone
gain-drones-3 = Dégâts 8 → 12
gain-drones-5 = Plus large et plus rapide
gain-vitality = +20 PV max, soignés (total { $total })
gain-swiftness = +8 % de vitesse (total +{ $total } %)
gain-magnet = Les gemmes viennent de 1 case plus loin (total { $total })
gain-might = +10 % de dégâts (total +{ $total } %)
gain-haste = Armes 8 % plus rapides (total −{ $total } %)
gain-flashlight = +2 cases de lumière (total { $total })
gain-regeneration = +0,4 PV par seconde (total { $whole },{ $tenth })
gain-glare = -10 % de vitesse aux ennemis dans la lumière (total -{ $total } %)
gain-dash = Maj : un dash, rien ne blesse pendant ; rechargé en 8 s
gain-dash-second = Un deuxième dash
gain-dash-sooner = Dash rechargé en { $tenth ->
    [0] { $whole } s
   *[other] { $whole },{ $tenth } s
}
gain-loot = +10 % de chances de gemme (total +{ $total } %)
gain-wisdom = +10 % d'XP des gemmes (total +{ $total } %)
gain-illumination = +1 XP chaque seconde
gain-heal = Soigne 20 PV

## Score

multiplier = ×{ $whole },{ $tenth }
over-points = { $score } points
leaderboard-title = Meilleurs scores
name-title = Nouveau record !
name-place = { $place }ᵉ du classement !
name-prompt = Ton nom :
help-name = ←→ bouger · ↑↓ ou une lettre · Entrée valider · Échap passer

## Hordes

horde-title = HORDE
horde-coming = Elle arrive !
horde-everywhere = Ils arrivent de partout !
horde-left = HORDE · { $left }
horde-left-of = HORDE { $number } · { $left }
horde-number = Horde { $number }
horde-cleared = HORDE ABATTUE
horde-cleared-points = +{ $points } points · +1 niveau
horde-cleared-level = +1 niveau

## Keys

keys-title = Commandes
keys-move = Flèches / ZQSD / WASD  bouger
keys-torch = F                      lampe
keys-dash = Maj                    dash (sa carte) : rien ne blesse
keys-cards = ← → / 1 2 3 · Entrée   prendre une carte (R : relancer)
keys-gear = I                      afficher l'équipement
keys-ping = E                      une flèche sur le héros, 2 s
keys-pause = P / Échap              pause
keys-keys = ?                      ces commandes
