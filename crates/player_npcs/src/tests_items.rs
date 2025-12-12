
use bevy::prelude::*;
use items::{spawn_weapon, Weapon, Armor, Item};
use crate::*;
use std::time::Duration;

// Helper to set up the test app
fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register types
    app.register_type::<Soldier>()
        .register_type::<AttackRange>()
        .register_type::<Moving>()
        .register_type::<Attacking>()
        .register_type::<Projectile>()
        .register_type::<SoldierConfig>()
        .register_type::<Equipment>()
        .register_type::<CombatStats>()
        .register_type::<Weapon>()
        .register_type::<Armor>()
        .register_type::<Item>();

    // Add required resources
    app.insert_resource(SoldierConfig {
        attack_timer: 1.0,
        projectile_speed: 100.0,
        projectile_damage: 10.0,
        projectile_lifetime: 2.0,
        attack_range: 200.0,
        move_speed: 100.0,
    });

    // Add the system we are testing
    app.add_systems(Update, recalculate_stats);

    app
}

#[test]
fn test_initial_stats_from_config() {
    let mut app = setup_test_app();

    let soldier_config = app.world().resource::<SoldierConfig>().clone();

    let soldier = app.world_mut().spawn((
        Soldier {
            attack_timer: Timer::from_seconds(soldier_config.attack_timer, TimerMode::Repeating),
            target: None,
        },
        AttackRange(soldier_config.attack_range),
        Equipment::default(),
        CombatStats {
            damage: soldier_config.projectile_damage,
            attack_range: soldier_config.attack_range,
            attack_cooldown: soldier_config.attack_timer,
            projectile_speed: soldier_config.projectile_speed,
            move_speed: soldier_config.move_speed,
        },
    )).id();

    // Run one frame to let systems run
    app.update();

    let stats = app.world().get::<CombatStats>(soldier).unwrap();

    assert_eq!(stats.damage, 10.0);
    assert_eq!(stats.attack_range, 200.0);
    assert_eq!(stats.attack_cooldown, 1.0);
    assert_eq!(stats.projectile_speed, 100.0);
    assert_eq!(stats.move_speed, 100.0);
}

#[test]
fn test_equip_weapon_updates_stats() {
    let mut app = setup_test_app();

    let soldier_config = app.world().resource::<SoldierConfig>().clone();

    let soldier = app.world_mut().spawn((
        Soldier {
            attack_timer: Timer::from_seconds(soldier_config.attack_timer, TimerMode::Repeating),
            target: None,
        },
        AttackRange(soldier_config.attack_range),
        Equipment::default(),
        CombatStats {
            damage: soldier_config.projectile_damage,
            attack_range: soldier_config.attack_range,
            attack_cooldown: soldier_config.attack_timer,
            projectile_speed: soldier_config.projectile_speed,
            move_speed: soldier_config.move_speed,
        },
    )).id();

    // Spawn a weapon
    let weapon = spawn_weapon(
        &mut app.world_mut().commands(),
        "Test Sword",
        5.0, // damage +5
        50.0, // range +50
        0.5, // attack_speed_modifier *0.5 (half cooldown = double speed)
        20.0, // projectile_speed +20
    );
    app.update(); // Flush commands to ensure weapon entity exists

    // Equip the weapon
    let mut equip = app.world_mut().get_mut::<Equipment>(soldier).unwrap();
    equip.main_hand = Some(weapon);

    // Run update to trigger recalculate_stats
    app.update();

    let stats = app.world().get::<CombatStats>(soldier).unwrap();
    let soldier_cmp = app.world().get::<Soldier>(soldier).unwrap();

    // Base: Damage 10, Range 200, Cooldown 1.0, Speed 100
    // Mod: Damage +5, Range +50, Cooldown *0.5, Speed +20
    // Expected: Damage 15, Range 250, Cooldown 0.5, Speed 120

    assert_eq!(stats.damage, 15.0);
    assert_eq!(stats.attack_range, 250.0);
    assert_eq!(stats.attack_cooldown, 0.5);
    assert_eq!(stats.projectile_speed, 120.0);

    // Verify Soldier attack_timer duration was updated
    assert_eq!(soldier_cmp.attack_timer.duration(), Duration::from_secs_f32(0.5));
}
