use {
    crate::{
        handle_generic_upgrades, handle_portal_upgrade, portal_spawn_logic, portal_tick_logic,
        EnemyScaling, PortalLevel, PortalSpawnTracker, PortalSpawner, UpgradeCost, UpgradeSlot,
    },
    bevy::{prelude::*, time::TimePlugin},
    common::{
        GrowthStrategy, RequestUpgrade, Reward, SpawnEnemyRequest, UpgradePortal, UpgradeableStat,
    },
    enemy::{AvailableEnemies, Enemy, EnemyConfig, Health, Lifetime},
    wallet::Wallet,
};

// Helper to manually spawn the "Archetype" (Mocking the Scene)
fn spawn_test_portal(commands: &mut Commands) -> Entity {
    let portal_entity = commands
        .spawn((
            PortalLevel {
                active: 0,
                max_unlocked: 0,
            },
            UpgradeCost {
                strategy: GrowthStrategy::Linear {
                    base: 100.0,
                    coefficient: 50.0,
                },
                current_price: 100.0,
            },
            PortalSpawner {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                interval_strategy: GrowthStrategy::Linear {
                    base: 1.0,
                    coefficient: 0.1,
                },
            },
            EnemyScaling {
                health_strategy: GrowthStrategy::Linear {
                    base: 50.0,
                    coefficient: 10.0,
                },
                reward_strategy: GrowthStrategy::Linear {
                    base: 10.0,
                    coefficient: 0.0,
                },
                speed_strategy: GrowthStrategy::Static(20.0),
                lifetime_strategy: GrowthStrategy::Linear {
                    base: 5.0,
                    coefficient: 0.5,
                },
            },
        ))
        .id();

    // Spawn Children (Upgrades)
    commands.entity(portal_entity).with_children(|parent| {
        // Capacity Upgrade
        parent.spawn((
            UpgradeSlot {
                name: "Capacity".to_string(),
            },
            UpgradeableStat {
                level: 0.0,
                value: 5.0,
                price: 200.0,
                value_strategy: GrowthStrategy::Incremental {
                    base: 5.0,
                    step: 1.0,
                },
                price_strategy: GrowthStrategy::Exponential {
                    base: 200.0,
                    factor: 1.5,
                },
            },
        ));

        // Lifetime Upgrade
        parent.spawn((
            UpgradeSlot {
                name: "Lifetime".to_string(),
            },
            UpgradeableStat {
                level: 0.0,
                value: 0.0,
                price: 100.0,
                value_strategy: GrowthStrategy::Incremental {
                    base: 0.0,
                    step: 1.0,
                },
                price_strategy: GrowthStrategy::Exponential {
                    base: 100.0,
                    factor: 1.5,
                },
            },
        ));
    });

    portal_entity
}

// Wrapper system for spawn helper (since we need Commands)
fn spawn_setup_system(mut commands: Commands) {
    spawn_test_portal(&mut commands);
}

fn setup_app() -> App {
    let mut app = App::new();
    // Disable TimePlugin to control time manually
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());

    app.add_message::<UpgradePortal>();
    app.add_message::<RequestUpgrade>();
    app.add_message::<SpawnEnemyRequest>();

    // Mock Window
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(800, 600),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));

    app.insert_resource(AvailableEnemies(vec![EnemyConfig {
        health_coef: 1.0,
        lifetime_coef: 1.0,
        speed_coef: 1.0,
        reward_coef: 1.0,
    }]));

    app.init_resource::<PortalSpawnTracker>();

    app.insert_resource(Wallet {
        void_shards: 1000.0,
    });

    // Add Systems
    app.add_systems(Startup, spawn_setup_system);
    app.add_systems(
        Update,
        (
            (portal_tick_logic, portal_spawn_logic).chain(),
            handle_portal_upgrade,
            handle_generic_upgrades,
        ),
    );

    app
}

#[test]
fn test_portal_initial_level() {
    let mut app = setup_app();
    app.update(); // Spawns portal

    let mut portal_query = app.world_mut().query::<(&PortalLevel, &UpgradeCost)>();
    let (level, cost) = portal_query
        .iter(app.world())
        .next()
        .expect("Portal not spawned");

    assert_eq!(level.max_unlocked, 0);
    assert_eq!(level.active, 0);
    assert_eq!(cost.current_price, 100.0);
}

#[test]
fn test_enemy_stats_at_level_0() {
    let mut app = setup_app();
    app.update(); // Spawns portal

    // Advance time to trigger spawn
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        // Spawn timer base is 1.0 + 0*0.1 = 1.0
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update(); // Spawns enemy (timer finished)

    let mut enemy_query = app
        .world_mut()
        .query::<(&Enemy, &Health, &Reward, &Lifetime)>();
    let enemy = enemy_query.iter(app.world()).next();

    assert!(enemy.is_some(), "Enemy should be spawned at level 0");
    let (_, health, reward, lifetime) = enemy.unwrap();

    // Level 0:
    // Health: base 50 + 0*10 = 50
    // Reward: base 10 + 0*0 = 10
    // Lifetime: base 5 + 0*0.5 = 5 (plus 0 bonus)
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
        // Cost at L0 is 100. 1000 - 100 = 900
        assert_eq!(wallet.void_shards, 900.0);

        let (level, cost) = app
            .world_mut()
            .query::<(&PortalLevel, &UpgradeCost)>()
            .single(app.world());

        // Level 1
        assert_eq!(level.max_unlocked, 1);
        // Active Level snaps to new max (QoL)
        assert_eq!(level.active, 1);

        // New Price at L1: 100 + 1*50 = 150
        assert_eq!(cost.current_price, 150.0);
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

    // Check Initial (Level 0)
    {
        let stat = app.world().get::<UpgradeableStat>(capacity_entity).unwrap();
        // Base 5
        assert_eq!(stat.value, 5.0);
        // Base 200
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
        // Level 1
        assert_eq!(stat.level, 1.0);
        // Value: 5 + 1*1 = 6
        assert_eq!(stat.value, 6.0);
        // Price: 200 * 1.5^1 = 300
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
        // Base 0
        assert_eq!(stat.value, 0.0);
        // Base 100
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
        // Level 1
        assert_eq!(stat.level, 1.0);
        // Value: 0 + 1*1 = 1
        assert_eq!(stat.value, 1.0);
        // Price: 100 * 1.5^1 = 150
        assert_eq!(stat.price, 150.0);
    }
}
