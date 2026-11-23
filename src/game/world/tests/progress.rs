//! The achievements and what they unlock.

use super::*;

#[test]
fn the_drizzle_unlocks_haste_and_300_kills_in_a_run_might() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.progress = Progress::default();
    world.next_horde = u32::MAX;
    world.ticks = (weather::DUSK_SECONDS + 2 * weather::stage_seconds(Mode::Standard)) * 30 - 1;
    wait(&mut world, 1);
    assert!(world.progress.deals(Choice::Passive(Passive::Haste)));
    assert!(!world.progress.deals(Choice::Passive(Passive::Might)));
    // The drizzle told, the world going on.
    world.announcement = None;
    world.kills = 300;
    wait(&mut world, 1);
    assert!(world.progress.deals(Choice::Passive(Passive::Might)));
}

#[test]
fn reaching_the_rain_unlocks_the_machine_gun_for_the_next_runs() {
    let mut world = on(Map::empty(), Vec2::new(100.0, 50.0));
    disarm(&mut world);
    world.progress = Progress::default();
    world.next_horde = u32::MAX;
    world.ticks = (weather::DUSK_SECONDS + 3 * weather::stage_seconds(Mode::Standard)) * 30 - 1;
    assert!(!world.progress.deals(Choice::Weapon(Weapon::MachineGun)));
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::Rain));
    assert!(world.progress.deals(Choice::Weapon(Weapon::MachineGun)));
    // Dealt from the next run on, not this one.
    world.dealt = Progress::default();
    let next = World::with(2, Mode::Standard, false, world.progress.clone());
    assert!(next.dealt.deals(Choice::Weapon(Weapon::MachineGun)));
}

#[test]
fn a_run_counts_itself_and_its_kills_in_the_progress() {
    let world = World::with(1, Mode::Standard, false, Progress::default());
    assert_eq!(world.progress.runs, 1);
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    let count = wipe_out_a_horde(&mut world);
    assert_eq!(world.progress.kills, u64::try_from(count).unwrap());
}

#[test]
fn a_horde_wiped_out_is_the_hunters_achievement() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    wipe_out_a_horde(&mut world);
    assert!(world.progress.achieved.contains(&Achievement::Hunter));
    assert!(world.progress.deals(Choice::Passive(Passive::HordeHunter)));
}

#[test]
fn two_hordes_wiped_out_in_a_run_unlock_the_drones() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    wipe_out_a_horde(&mut world);
    assert!(!world.progress.deals(Choice::Weapon(Weapon::Drones)));
    wipe_out_a_horde(&mut world);
    assert!(world.progress.deals(Choice::Weapon(Weapon::Drones)));
}

#[test]
fn every_horde_of_a_run_wiped_out_unlocks_the_flamethrower() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    world.next_horde = weather::dawn() - 1;
    wipe_out_a_horde(&mut world);
    assert!(
        !world.progress.deals(Choice::Weapon(Weapon::Flamethrower)),
        "one still to come"
    );
    // The last of the run, none after it before the dawn.
    world.next_horde = weather::dawn() + 1;
    wipe_out_a_horde(&mut world);
    assert!(world.progress.deals(Choice::Weapon(Weapon::Flamethrower)));
}

#[test]
fn level_25_in_a_run_unlocks_illumination() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.progress = Progress::default();
    world.player.level = 24;
    wait(&mut world, 1);
    // The survivor's barricade come on the way: its box seen.
    world.seen_action();
    assert!(!world.progress.achieved.contains(&Achievement::Veteran));
    world.player.level = 25;
    wait(&mut world, 1);
    assert!(
        world
            .progress
            .deals(Choice::Legendary(Legendary::Illumination))
    );
}

#[test]
fn ten_abominations_felled_in_melee_in_a_run_unlock_the_demolisher_and_the_c4() {
    let mut world = armed_with(Weapon::Crowbar, 1);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.player.facing = Facing::Right;
    for felled in 0..10 {
        assert!(
            !world.progress.achieved.contains(&Achievement::Demolisher),
            "{felled}"
        );
        assert!(!world.progress.deals(Choice::Weapon(Weapon::Grenade)));
        world.melee.clear();
        world
            .enemies
            .push(enemy(EnemyKind::Abomination, Vec2::new(101.5, 50.0)));
        let last = world.enemies.len() - 1;
        world.enemies[last].hp = 1;
        world.enemies[last].stunned = u32::MAX;
        wait(&mut world, 1);
        world.enemies.clear();
        world.swellings.clear();
    }
    assert!(world.progress.achieved.contains(&Achievement::Demolisher));
    assert!(world.progress.deals(Choice::Weapon(Weapon::Grenade)));
}

#[test]
fn a_minute_under_30_percent_of_hp_in_a_run_unlocks_the_special_agent_and_the_chainsaw() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.weather_news = WeatherNews::Banner;
    world.player.hp = 29;
    wait(&mut world, 30 * 60 - 1);
    assert!(!world.progress.achieved.contains(&Achievement::ColdBlood));
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::ColdBlood));
    assert_eq!(Skin::Agent.achievement(), Some(Achievement::ColdBlood));
    assert!(world.progress.deals(Choice::Weapon(Weapon::Chainsaw)));
}

#[test]
fn shadow_hunter_unlocks_the_shurikens_and_the_ninja() {
    let mut progress = Progress::default();
    assert!(!progress.deals(Choice::Weapon(Weapon::Shuriken)));
    progress.reach(Achievement::ShadowHunter);
    assert!(progress.deals(Choice::Weapon(Weapon::Shuriken)));
    assert_eq!(Skin::Ninja.achievement(), Some(Achievement::ShadowHunter));
}

#[test]
fn seven_hundred_enemies_felled_in_a_run_unlock_the_veteran_and_the_napalm() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    world.kills = 699;
    wait(&mut world, 1);
    assert!(!world.progress.achieved.contains(&Achievement::Legend));
    assert!(!world.progress.deals(Choice::Weapon(Weapon::Napalm)));
    world.kills = 700;
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::Legend));
    assert!(world.progress.deals(Choice::Weapon(Weapon::Napalm)));
}

#[test]
fn three_hundred_hp_got_back_in_a_run_unlock_the_medic_and_life_drain() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.progress = Progress::default();
    for _ in 0..3 {
        world.player.hp = 1;
        world.heal(99);
    }
    wait(&mut world, 1);
    assert!(
        !world.progress.achieved.contains(&Achievement::FirstAid),
        "297"
    );
    assert!(!world.progress.deals(Choice::Passive(Passive::Drain)));
    world.player.hp = 1;
    world.heal(3);
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::FirstAid));
    assert!(world.progress.deals(Choice::Passive(Passive::Drain)));
}

#[test]
fn the_dawn_reached_with_a_hero_unlocks_their_second_colours() {
    let mut world = World::with(1, Mode::Standard, false, Progress::default());
    world.play_as(Skin::Medic);
    world.announcement = None;
    world.ticks = weather::dawn() + weather::daybreak() - 1;
    world.update(Input::default());
    assert!(world.survived());
    assert!(world.progress.wears(Skin::Medic, 1));
    assert!(!world.progress.wears(Skin::Survivor, 1));
}

#[test]
fn a_hero_once_with_one_outfit_has_their_second_unlocked_by_the_dawn_too() {
    let mut world = World::with(1, Mode::Standard, false, Progress::default());
    world.announcement = None;
    assert!(!world.progress.wears(Skin::Survivor, 1));
    world.ticks = weather::dawn() + weather::daybreak() - 1;
    world.update(Input::default());
    assert!(world.survived());
    assert!(world.progress.wears(Skin::Survivor, 1));
}

#[test]
fn three_minutes_with_the_torch_out_in_a_run_unlock_the_infiltrator_and_the_sniper() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.unhurt = true;
    world.progress = Progress::default();
    world.ticks = 30 * 40;
    world.torch_switch = false;
    world.announcement = None;
    // Until the tick asked for: an announcement holds the ticks back a while.
    let until = |world: &mut World, tick: u32| {
        while world.ticks < tick {
            world.spawn_debt = 0.0;
            world.update(Input::default());
        }
    };
    let start = world.ticks;
    until(&mut world, start + 30 * 180 - 1);
    assert!(!world.progress.achieved.contains(&Achievement::Shadow));
    until(&mut world, start + 30 * 180);
    assert!(world.progress.achieved.contains(&Achievement::Shadow));
    assert!(world.progress.deals(Choice::Weapon(Weapon::Sniper)));
    assert_eq!(Skin::Infiltrator.achievement(), Some(Achievement::Shadow));
    assert_eq!(Skin::Infiltrator.weapon(), Weapon::Fists);
    assert!(Skin::Infiltrator.perk().ghost());
}

#[test]
fn thirty_seconds_among_a_dozen_enemies_in_a_run_unlock_the_riot_officer_and_the_ram() {
    let mut world = on(Map::empty(), Vec2::new(100.5, 50.5));
    disarm(&mut world);
    world.next_horde = u32::MAX;
    world.unhurt = true;
    world.progress = Progress::default();
    for k in 0..12 {
        let angle = std::f32::consts::TAU * f32::from(u8::try_from(k).unwrap()) / 12.0;
        world.enemies.push(enemy(
            EnemyKind::Zombie,
            Vec2::new(100.5 + 2.0 * angle.cos(), 50.5 + 2.0 * angle.sin()),
        ));
    }
    for enemy in &mut world.enemies {
        enemy.stunned = u32::MAX;
    }
    wait(&mut world, 30 * 30 - 1);
    assert!(!world.progress.achieved.contains(&Achievement::HoldTheLine));
    wait(&mut world, 1);
    assert!(world.progress.achieved.contains(&Achievement::HoldTheLine));
    // The riot officer, their ram their own: neither in the collection.
    assert_eq!(Achievement::HoldTheLine.unlocks(), Unlock::Hero(Skin::Riot));
    assert!(!COLLECTION.contains(&Unlock::Card(Choice::Action(Action::Ram))));
    assert!(!COLLECTION.contains(&Unlock::Hero(Skin::Riot)));
    assert_eq!(Skin::Riot.achievement(), Some(Achievement::HoldTheLine));
    assert_eq!(Skin::Riot.weapon(), Weapon::Truncheon);
}
