use {
    crate::*,
    bevy::prelude::*,
    items::{
        Armor, AttackRange as ItemAttackRange, AttackSpeed, BaseDamage, Item, Melee,
        ProjectileStats as ItemProjectileStats, Ranged,
    },
    std::time::Duration,
};

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
        .register_type::<Armor>()
        .register_type::<Item>()
        .register_type::<Melee>()
        .register_type::<Ranged>()
        .register_type::<BaseDamage>()
        .register_type::<ItemAttackRange>()
        .register_type::<AttackSpeed>()
        .register_type::<ItemProjectileStats>();

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

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(
                    soldier_config.attack_timer,
                    TimerMode::Repeating,
                ),
                target: None,
            },
            AttackRange(soldier_config.attack_range),
            Equipment::default(),
            CombatStats {
                damage: soldier_config.projectile_damage,
                attack_range: soldier_config.attack_range,
                attack_cooldown: soldier_config.attack_timer,
                projectile_speed: soldier_config.projectile_speed,
                projectile_lifetime: soldier_config.projectile_lifetime,
                move_speed: soldier_config.move_speed,
            },
        ))
        .id();

    // Run one frame to let systems run
    app.update();

    let stats = app.world().get::<CombatStats>(soldier).unwrap();

    assert_eq!(stats.damage, 10.0);
    assert_eq!(stats.attack_range, 200.0);
    assert_eq!(stats.attack_cooldown, 1.0);
    assert_eq!(stats.projectile_speed, 100.0);
    assert_eq!(stats.move_speed, 100.0);
    assert_eq!(stats.projectile_lifetime, 2.0);
}

#[test]
fn test_equip_melee_weapon_updates_stats() {
    let mut app = setup_test_app();

    let soldier_config = app.world().resource::<SoldierConfig>().clone();

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(
                    soldier_config.attack_timer,
                    TimerMode::Repeating,
                ),
                target: None,
            },
            AttackRange(soldier_config.attack_range),
            Equipment::default(),
            CombatStats {
                damage: soldier_config.projectile_damage,
                attack_range: soldier_config.attack_range,
                attack_cooldown: soldier_config.attack_timer,
                projectile_speed: soldier_config.projectile_speed,
                projectile_lifetime: soldier_config.projectile_lifetime,
                move_speed: soldier_config.move_speed,
            },
        ))
        .id();

    // Spawn a weapon manually with components
    let weapon = app
        .world_mut()
        .spawn((
            Item {
                name: "Test Sword".to_string(),
            },
            Melee,
            BaseDamage(20.0),      // Override damage to 20
            ItemAttackRange(50.0), // Override range to 50
            AttackSpeed(0.5),      // Override cooldown to 0.5
        ))
        .id();

    // Equip the weapon
    let mut equip = app.world_mut().get_mut::<Equipment>(soldier).unwrap();
    equip.main_hand = Some(weapon);

    // Run update to trigger recalculate_stats
    app.update();

    let stats = app.world().get::<CombatStats>(soldier).unwrap();
    let soldier_cmp = app.world().get::<Soldier>(soldier).unwrap();

    assert_eq!(stats.damage, 20.0);
    assert_eq!(stats.attack_range, 50.0);
    assert_eq!(stats.attack_cooldown, 0.5);
    // Projectile speed/lifetime should remain base as not specified in melee weapon
    assert_eq!(stats.projectile_speed, 100.0);

    // Verify Soldier attack_timer duration was updated
    assert_eq!(
        soldier_cmp.attack_timer.duration(),
        Duration::from_secs_f32(0.5)
    );

    // Check for Melee marker
    assert!(app.world().entity(soldier).contains::<Melee>());
    assert!(!app.world().entity(soldier).contains::<Ranged>());
}

#[test]
fn test_equip_ranged_weapon_updates_stats() {
    let mut app = setup_test_app();

    let soldier_config = app.world().resource::<SoldierConfig>().clone();

    let soldier = app
        .world_mut()
        .spawn((
            Soldier {
                attack_timer: Timer::from_seconds(
                    soldier_config.attack_timer,
                    TimerMode::Repeating,
                ),
                target: None,
            },
            AttackRange(soldier_config.attack_range),
            Equipment::default(),
            CombatStats {
                damage: soldier_config.projectile_damage,
                attack_range: soldier_config.attack_range,
                attack_cooldown: soldier_config.attack_timer,
                projectile_speed: soldier_config.projectile_speed,
                projectile_lifetime: soldier_config.projectile_lifetime,
                move_speed: soldier_config.move_speed,
            },
        ))
        .id();

    // Spawn a ranged weapon manually
    let weapon = app
        .world_mut()
        .spawn((
            Item {
                name: "Test Bow".to_string(),
            },
            Ranged,
            BaseDamage(12.0),
            ItemAttackRange(300.0),
            AttackSpeed(1.5),
            ItemProjectileStats {
                speed: 400.0,
                lifetime: 3.0,
            },
        ))
        .id();

    // Equip the weapon
    let mut equip = app.world_mut().get_mut::<Equipment>(soldier).unwrap();
    equip.main_hand = Some(weapon);

    // Run update
    app.update();

    let stats = app.world().get::<CombatStats>(soldier).unwrap();

    assert_eq!(stats.damage, 12.0);
    assert_eq!(stats.attack_range, 300.0);
    assert_eq!(stats.attack_cooldown, 1.5);
    assert_eq!(stats.projectile_speed, 400.0);
    assert_eq!(stats.projectile_lifetime, 3.0);

    // Check for Ranged marker
    assert!(app.world().entity(soldier).contains::<Ranged>());
    assert!(!app.world().entity(soldier).contains::<Melee>());
}
