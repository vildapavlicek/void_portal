use {
    crate::{
        handle_generic_upgrades, handle_portal_upgrade, spawn_enemies, spawn_portal,
        IndependentStatConfig, LevelScaledStat, LevelScaledStats, LevelUpConfig, Portal,
        PortalConfig, PortalSpawnTracker, UpgradeSlot,
    },
    bevy::{prelude::*, time::TimePlugin},
    common::{GrowthStrategy, RequestUpgrade, Reward, UpgradePortal, UpgradeableStat},
    enemy::{AvailableEnemies, Enemy, EnemyConfig, Health, Lifetime},
    std::collections::HashMap,
    wallet::Wallet,
};

// Helper to setup the app with necessary resources
fn setup_app() -> App {
    let mut app = App::new();
    // Disable TimePlugin to control time manually
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());

    app.add_message::<UpgradePortal>();
    app.add_message::<RequestUpgrade>();

    // Mock Window
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(800, 600),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));

    let mut upgrades = HashMap::new();
    upgrades.insert(
        "Capacity".to_string(),
        IndependentStatConfig {
            value: 5.0,
            price: 200.0,
            growth_factor: 1.0,
            price_growth_factor: 1.5,
            growth_strategy: GrowthStrategy::Linear,
            price_growth_strategy: GrowthStrategy::Exponential,
        },
    );
    upgrades.insert(
        "Lifetime".to_string(),
        IndependentStatConfig {
            value: 0.0,
            price: 100.0,
            growth_factor: 1.0,
            price_growth_factor: 1.5,
            growth_strategy: GrowthStrategy::Linear,
            price_growth_strategy: GrowthStrategy::Exponential,
        },
    );

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
                growth_strategy: GrowthStrategy::Linear,
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
        upgrades,
    });

    app.insert_resource(AvailableEnemies(vec![EnemyConfig {
        health_coef: 1.0,
        lifetime_coef: 1.0,
        speed_coef: 1.0,
        reward_coef: 1.0,
    }]));

    app.init_resource::<PortalSpawnTracker>();
    // Time resource already inserted above

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
            handle_generic_upgrades,
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

    // Level 1:
    // Health = 50
    // Reward = 10
    // Lifetime = 5 (base) + 0 (bonus) = 5
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
    }
}

#[test]
fn test_capacity_upgrade() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Find Capacity Entity
    let capacity_entity = app
        .world_mut()
        .query::<(&UpgradeSlot, Entity)>()
        .iter(app.world())
        .find(|(slot, _)| slot.name == "Capacity")
        .map(|(_, e)| e)
        .expect("Capacity not found");

    // Check Initial
    {
        let stat = app.world().get::<UpgradeableStat>(capacity_entity).unwrap();
        assert_eq!(stat.value, 5.0);
        assert_eq!(stat.price, 200.0);
    }

    // Request Upgrade
    app.world_mut()
        .resource_mut::<Messages<RequestUpgrade>>()
        .write(RequestUpgrade {
            upgrade_entity: capacity_entity,
        });

    app.update();

    // Check Result
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 200 = 800
        assert_eq!(wallet.void_shards, 800.0);

        let stat = app.world().get::<UpgradeableStat>(capacity_entity).unwrap();
        // Value: 5 + 1 = 6
        assert_eq!(stat.value, 6.0);
        // Price: 200 * 1.5 = 300
        assert_eq!(stat.price, 300.0);
    }
}

#[test]
fn test_bonus_lifetime_upgrade() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Find Lifetime Entity
    let lifetime_entity = app
        .world_mut()
        .query::<(&UpgradeSlot, Entity)>()
        .iter(app.world())
        .find(|(slot, _)| slot.name == "Lifetime")
        .map(|(_, e)| e)
        .expect("Lifetime not found");

    // Check Initial
    {
        let stat = app.world().get::<UpgradeableStat>(lifetime_entity).unwrap();
        assert_eq!(stat.value, 0.0);
        assert_eq!(stat.price, 100.0);
    }

    // Request Upgrade (System checks wallet now)
    app.world_mut()
        .resource_mut::<Messages<RequestUpgrade>>()
        .write(RequestUpgrade {
            upgrade_entity: lifetime_entity,
        });

    app.update();

    // Check Result
    {
        let wallet = app.world().resource::<Wallet>();
        // 1000 - 100 = 900
        assert_eq!(wallet.void_shards, 900.0);

        let stat = app.world().get::<UpgradeableStat>(lifetime_entity).unwrap();
        // Value: 0 + 1 = 1
        assert_eq!(stat.value, 1.0);
        // Price: 100 * 1.5 = 150
        assert_eq!(stat.price, 150.0);
    }
}
