use {
    crate::{
        handle_portal_bonus_lifetime_upgrade, handle_portal_capacity_upgrade,
        handle_portal_upgrade, spawn_enemies, spawn_portal, EnemySpawnTimer, IndependentStatConfig,
        IndependentlyLeveledStats, LevelScaledStat, LevelScaledStats, LevelUpConfig, Portal,
        PortalBonusLifetime, PortalCapacity, PortalConfig, PortalSpawnTracker,
    },
    bevy::{prelude::*, time::TimePlugin},
    common::{
        GrowthStrategy, Reward, UpgradePortal, UpgradePortalBonusLifetime, UpgradePortalCapacity,
    },
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
    app.add_message::<UpgradePortalBonusLifetime>();

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
        level: 1,
        level_up_price: LevelUpConfig {
            value: 100.0,
            growth_factor: 1.5,
            growth_strategy: GrowthStrategy::Linear,
        },
        portal_top_offset: 100.0,
        level_scaled_stats: LevelScaledStats {
            void_shards_reward: LevelScaledStat {
                value: 10.0,
                growth_factor: 0.0,
                growth_strategy: GrowthStrategy::Linear,
            },
            spawn_timer: LevelScaledStat {
                value: 1.0,
                growth_factor: 1.1,
                growth_strategy: GrowthStrategy::Linear,
            },
            enemy_health: LevelScaledStat {
                value: 50.0,
                growth_factor: 1.2,
                growth_strategy: GrowthStrategy::Linear, // Using Linear to match test expectations if possible, or adapt test
            },
            base_enemy_speed: LevelScaledStat {
                value: 20.0,
                growth_factor: 0.0,
                growth_strategy: GrowthStrategy::Linear,
            },
            base_enemy_lifetime: LevelScaledStat {
                value: 5.0,
                growth_factor: 1.1,
                growth_strategy: GrowthStrategy::Linear,
            },
        },
        independently_leveled_stats: IndependentlyLeveledStats {
            capacity: IndependentStatConfig {
                value: 5.0,
                price: 200.0,
                growth_factor: 1.0,
                price_growth_factor: 1.5,
                growth_strategy: GrowthStrategy::Linear,
                price_growth_strategy: GrowthStrategy::Exponential,
            },
            lifetime: IndependentStatConfig {
                value: 0.0,
                price: 100.0,
                growth_factor: 1.0,
                price_growth_factor: 1.5,
                growth_strategy: GrowthStrategy::Linear,
                price_growth_strategy: GrowthStrategy::Exponential,
            },
        },
    });

    app.insert_resource(AvailableEnemies(vec![EnemyConfig {
        health_coef: 1.0,
        lifetime_coef: 1.0,
        speed_coef: 1.0,
        reward_coef: 1.0,
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
            handle_portal_bonus_lifetime_upgrade,
        ),
    );

    app
}

#[test]
fn test_portal_initial_level() {
    let mut app = setup_app();
    app.update();

    let mut portal_query = app.world_mut().query::<&Portal>();
    let portal = portal_query.iter(app.world()).next();

    assert!(portal.is_some());
    let portal = portal.unwrap();
    assert_eq!(portal.level, 1);
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

    // Level 1 (effective level 0 for scaling):
    // Health = 50 + (0 * 1.2) = 50
    // Reward = 10 + (0 * 0) = 10
    // Lifetime = 5 + (0 * 1.1) = 5
    assert_eq!(health.max, 50.0);
    assert_eq!(reward.0, 10.0);
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

        let portal = app
            .world_mut()
            .query::<&Portal>()
            .single(app.world())
            .unwrap();
        assert_eq!(portal.level, 1);
        assert_eq!(portal.upgrade_price, 100.0);
    }

    app.update(); // Process upgrade

    // Post upgrade check
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 100 = 900
        assert_eq!(wallet.void_shards, 900.0);

        let portal = app
            .world_mut()
            .query::<&Portal>()
            .single(app.world())
            .unwrap();
        // Level 2
        assert_eq!(portal.level, 2);
        // Price: Linear Growth Strategy in Test Setup: 100 + 1.5 = 101.5
        // Wait, setup says GrowthStrategy::Linear for level_up_price.
        // Formula: value + (level * factor) ?? No, upgrade_price in Portal component is modified directly.
        // In handle_portal_upgrade:
        // Linear: portal.upgrade_price += portal.price_growth_factor
        // So 100 + 1.5 = 101.5
        assert_eq!(portal.upgrade_price, 101.5);
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
    let portal = app
        .world_mut()
        .query::<&Portal>()
        .single(app.world())
        .unwrap();
    assert_eq!(portal.level, 2);

    // Reset timer
    app.world_mut().resource_mut::<EnemySpawnTimer>().0.reset();
    // Advance time
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(2.5)); // Plenty of time

    app.update(); // Spawn enemy at Level 2

    let mut enemy_query = app
        .world_mut()
        .query::<(&Enemy, &Health, &Reward, &Lifetime)>();
    let enemy = enemy_query.iter(app.world()).next();

    assert!(enemy.is_some(), "Enemy should be spawned at level 2");
    let (_, health, reward, lifetime) = enemy.unwrap();

    // Level 2 (effective level 1 for scaling):
    // Health (Linear): 50 + (1 * 1.2) = 51.2
    // Reward (Linear): 10 + (1 * 0) = 10
    // Lifetime (Linear): 5 + (1 * 1.1) = 6.1
    // Note: Config says spawn_timer factor 1.1, Linear. 1 + (1*1.1) = 2.1s duration.

    assert!((health.max - 51.2).abs() < 0.001);
    assert!((reward.0 - 10.0).abs() < 0.001);
    assert!((lifetime.timer.duration().as_secs_f32() - 6.1).abs() < 0.001);

    // Verify Spawn Timer Duration Update
    let spawn_timer = app.world().resource::<EnemySpawnTimer>();
    // Spawn Timer (Linear): 1.0 + (1 * 1.1) = 2.1
    assert!((spawn_timer.0.duration().as_secs_f32() - 2.1).abs() < 0.001);
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

    let portal = app
        .world_mut()
        .query::<&Portal>()
        .single(app.world())
        .unwrap();
    assert_eq!(portal.level, 1); // No level up
}

#[test]
fn test_capacity_upgrade() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    let portal_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Portal>>()
        .single(app.world())
        .expect("Portal should exist");

    // Initial check
    {
        let Ok((_, capacity)) = app
            .world_mut()
            .query::<(&Portal, &PortalCapacity)>()
            .single(app.world())
        else {
            panic!("Could not find portal capacity");
        };
        assert_eq!(capacity.0.value, 5.0); // Base capacity
        assert_eq!(capacity.0.price, 200.0); // Base capacity price
    }

    // Trigger capacity upgrade
    // Manually subtract funds to simulate UI logic, since game system no longer does it
    app.world_mut().resource_mut::<Wallet>().void_shards -= 200.0;
    app.world_mut()
        .resource_mut::<Messages<UpgradePortalCapacity>>()
        .write(UpgradePortalCapacity {
            entity: portal_entity,
        });

    app.update();

    // Post upgrade check
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 200 = 800
        assert_eq!(wallet.void_shards, 800.0);

        let Ok((_, capacity)) = app
            .world_mut()
            .query::<(&Portal, &PortalCapacity)>()
            .single(app.world())
        else {
            panic!("Could not find portal capacity");
        };

        // Capacity value growth: Linear, factor 1.0. 5 + 1 = 6.
        assert_eq!(capacity.0.value, 6.0);

        // Price growth: Exponential, factor 1.5. 200 * 1.5^1 = 300.
        assert_eq!(capacity.0.price, 300.0);
    }
}

#[test]
fn test_bonus_lifetime_upgrade() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Initial check and get Entity
    let portal_entity = {
        let Ok((entity, _, _, lifetime)) = app
            .world_mut()
            .query::<(Entity, &Portal, &PortalCapacity, &PortalBonusLifetime)>()
            .iter(app.world())
            .next()
            .ok_or(())
        else {
            panic!("Could not find portal bonus lifetime");
        };
        assert_eq!(lifetime.0.value, 0.0); // Base lifetime
        assert_eq!(lifetime.0.price, 100.0); // Base lifetime price
        entity
    };

    // Simulate UI Action: Check funds, Subtract Funds, Send Message
    {
        let mut wallet = app.world_mut().resource_mut::<Wallet>();
        if wallet.void_shards >= 100.0 {
            wallet.void_shards -= 100.0;
            app.world_mut()
                .resource_mut::<Messages<UpgradePortalBonusLifetime>>()
                .write(UpgradePortalBonusLifetime {
                    entity: portal_entity,
                });
        }
    }

    app.update();

    // Post upgrade check
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 100 = 900
        assert_eq!(wallet.void_shards, 900.0);

        let Ok((_, _, lifetime)) = app
            .world_mut()
            .query::<(&Portal, &PortalCapacity, &PortalBonusLifetime)>()
            .single(app.world())
        else {
            panic!("Could not find portal bonus lifetime");
        };

        // Lifetime value growth: Linear, factor 1.0. 0 + 1 = 1.
        assert_eq!(lifetime.0.value, 1.0);

        // Price growth: Exponential, factor 1.5. 100 * 1.5^1 = 150.
        assert_eq!(lifetime.0.price, 150.0);
    }
}
