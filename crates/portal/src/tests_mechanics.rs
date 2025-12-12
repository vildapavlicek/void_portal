use {
    crate::{
        handle_portal_capacity_upgrade, handle_portal_upgrade, spawn_enemies, spawn_portal,
        Capacity, CapacityUpgradePrice, EnemySpawnTimer, Level, Portal, PortalConfig,
        PortalSpawnTracker, UpgradePrice,
    },
    bevy::{prelude::*, time::TimePlugin},
    common::{Reward, UpgradePortal, UpgradePortalCapacity},
    enemy::{AvailableEnemies, Enemy, EnemyConfig, Health, Lifetime},
    wallet::Wallet,
};

// Helper to setup the app with necessary resources
fn setup_app() -> App {
    let mut app = App::new();
    // Disable TimePlugin to control time manually
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());

    app.add_message::<UpgradePortal>();
    app.add_message::<UpgradePortalCapacity>();

    // Mock Window
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(800, 600),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));

    // Resources
    app.insert_resource(PortalConfig {
        spawn_timer: 1.0,
        base_void_shards_reward: 10.0,
        base_upgrade_price: 100.0,
        upgrade_price_increase_coef: 1.5,
        portal_top_offset: 100.0,
        base_enemy_health: 50.0,
        base_enemy_speed: 20.0,
        base_enemy_lifetime: 5.0,
        base_enemy_reward: 5.0,
        enemy_health_growth_factor: 1.2,
        enemy_reward_growth_factor: 1.1,
        // New configs
        spawn_time_growth_factor: 1.1,
        enemy_lifetime_growth_factor: 1.1,
        base_capacity: 5,
        capacity_upgrade_base_price: 200.0,
        capacity_upgrade_coef: 1.5,
    });

    app.insert_resource(AvailableEnemies(vec![EnemyConfig {
        health_coef: 1.0,
        lifetime_coef: 1.0,
        speed_coef: 1.0,
        reward_coef: 1.0,
        spawn_time_coef: 1.0,
    }]));

    app.init_resource::<PortalSpawnTracker>();
    // Time resource already inserted above

    app.insert_resource(EnemySpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    app.insert_resource(Wallet {
        void_shards: 1000.0,
    }); // Rich wallet

    // Add Systems
    app.add_systems(
        Update,
        (
            spawn_portal,
            spawn_enemies,
            handle_portal_upgrade,
            handle_portal_capacity_upgrade,
        ),
    );

    app
}

#[test]
fn test_portal_initial_level() {
    let mut app = setup_app();
    app.update();

    let mut portal_query = app.world_mut().query::<(&Portal, &Level)>();
    let portal_entity = portal_query.iter(app.world()).next();

    assert!(portal_entity.is_some());
    let (_, level) = portal_entity.unwrap();
    assert_eq!(level.0, 1);
}

#[test]
fn test_enemy_stats_at_level_1() {
    let mut app = setup_app();
    app.update(); // Spawns portal

    // Advance time to trigger spawn
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update(); // Spawns enemy (timer finished)

    let mut enemy_query = app
        .world_mut()
        .query::<(&Enemy, &Health, &Reward, &Lifetime)>();
    let enemy = enemy_query.iter(app.world()).next();

    assert!(enemy.is_some(), "Enemy should be spawned at level 1");
    let (_, health, reward, lifetime) = enemy.unwrap();

    // Level 1: Multiplier = Growth ^ (1 - 1) = 1.0
    // Health = 50 * 1.0 * 1.0 = 50
    // Reward = 5 * 1.0 * 1.0 = 5
    // Lifetime = 5 * 1.0 * 1.0 = 5
    assert_eq!(health.max, 50.0);
    assert_eq!(reward.0, 5.0);
    assert_eq!(lifetime.timer.duration().as_secs_f32(), 5.0);
}

#[test]
fn test_portal_upgrade() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Trigger upgrade
    let mut messages = app.world_mut().resource_mut::<Messages<UpgradePortal>>();
    messages.write(UpgradePortal);

    // Initial check
    {
        let wallet = app.world().resource::<Wallet>();
        assert_eq!(wallet.void_shards, 1000.0);

        let (_, level, price) = app
            .world_mut()
            .query::<(&Portal, &Level, &UpgradePrice)>()
            .single(app.world())
            .unwrap();
        assert_eq!(level.0, 1);
        assert_eq!(price.0, 100.0);
    }

    app.update(); // Process upgrade

    // Post upgrade check
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 100 = 900
        assert_eq!(wallet.void_shards, 900.0);

        let (_, level, price) = app
            .world_mut()
            .query::<(&Portal, &Level, &UpgradePrice)>()
            .single(app.world())
            .unwrap();
        // Level 2
        assert_eq!(level.0, 2);
        // Price: 100 * 1.5 = 150
        assert_eq!(price.0, 150.0);
    }
}

#[test]
fn test_enemy_stats_at_level_2() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Upgrade to Level 2
    app.world_mut()
        .resource_mut::<Messages<UpgradePortal>>()
        .write(UpgradePortal);
    app.update();

    // Verify Level 2
    let (_, level) = app
        .world_mut()
        .query::<(&Portal, &Level)>()
        .single(app.world())
        .unwrap();
    assert_eq!(level.0, 2);

    // Reset timer
    app.world_mut().resource_mut::<EnemySpawnTimer>().0.reset();
    // Advance time
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(1.1));

    app.update(); // Spawn enemy at Level 2

    let mut enemy_query = app
        .world_mut()
        .query::<(&Enemy, &Health, &Reward, &Lifetime)>();
    let enemy = enemy_query.iter(app.world()).next();

    assert!(enemy.is_some(), "Enemy should be spawned at level 2");
    let (_, health, reward, lifetime) = enemy.unwrap();

    // Level 2: Multiplier = Growth ^ (2 - 1) = Growth ^ 1
    // Health Growth = 1.2
    // Reward Growth = 1.1
    // Lifetime Growth = 1.1
    // Health = 50 * 1.2 = 60
    // Reward = 5 * 1.1 = 5.5
    // Lifetime = 5 * 1.1 = 5.5
    assert!((health.max - 60.0).abs() < 0.001);
    assert!((reward.0 - 5.5).abs() < 0.001);
    assert!((lifetime.timer.duration().as_secs_f32() - 5.5).abs() < 0.001);

    // Verify Spawn Timer Duration Update
    let spawn_timer = app.world().resource::<EnemySpawnTimer>();
    // Base Spawn Time = 1.0. Growth = 1.1. Level 2 -> 1.0 * 1.1 = 1.1
    assert!((spawn_timer.0.duration().as_secs_f32() - 1.1).abs() < 0.001);
}

#[test]
fn test_upgrade_insufficient_funds() {
    let mut app = setup_app();
    app.world_mut().resource_mut::<Wallet>().void_shards = 50.0; // Poor wallet
    app.update(); // Spawn portal (Price 100)

    app.world_mut()
        .resource_mut::<Messages<UpgradePortal>>()
        .write(UpgradePortal);

    app.update();

    let wallet = app.world().resource::<Wallet>();
    assert_eq!(wallet.void_shards, 50.0); // No change

    let (_, level) = app
        .world_mut()
        .query::<(&Portal, &Level)>()
        .single(app.world())
        .unwrap();
    assert_eq!(level.0, 1); // No level up
}

#[test]
fn test_capacity_upgrade() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Initial check
    {
        let (_, capacity, price) = app
            .world_mut()
            .query::<(&Portal, &Capacity, &CapacityUpgradePrice)>()
            .single(app.world())
            .unwrap();
        assert_eq!(capacity.0, 5); // Base capacity
        assert_eq!(price.0, 200.0); // Base capacity price
    }

    // Trigger capacity upgrade
    app.world_mut()
        .resource_mut::<Messages<UpgradePortalCapacity>>()
        .write(UpgradePortalCapacity);

    app.update();

    // Post upgrade check
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 200 = 800
        assert_eq!(wallet.void_shards, 800.0);

        let (_, capacity, price) = app
            .world_mut()
            .query::<(&Portal, &Capacity, &CapacityUpgradePrice)>()
            .single(app.world())
            .unwrap();
        assert_eq!(capacity.0, 6);
        // Price: 200 * 1.5 = 300
        assert_eq!(price.0, 300.0);
    }
}
