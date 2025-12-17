use {
    bevy::{prelude::*, time::TimePlugin},
    common::{
        components::{
            MonsterScaling, PortalLevel, PortalRoot, PortalSpawner, UpgradeCost, UpgradeSlot,
        },
        MonsterKilled, GrowthStrategy, Reward, SpawnEnemyRequest, UpgradeableStat,
    },
    monster_factory::SpawnMonsterEvent,
    monsters::{
        despawn_dead_bodies, manage_monster_lifecycle, move_monsters, update_monster_health_ui,
        AvailableEnemies, Monster, MonsterConfig, Health, Lifetime, Speed,
    },
    player_npcs::{move_projectiles, projectile_collision},
    portal::{portal_spawn_logic, portal_tick_logic, PortalSpawnTracker},
};

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());
    app.add_message::<MonsterKilled>();
    app.add_message::<SpawnEnemyRequest>();
    app.add_message::<SpawnMonsterEvent>();

    // Mock Window
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(800, 600),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));

    app.insert_resource(AvailableEnemies(vec![MonsterConfig {
        health_coef: 1.0,
        lifetime_coef: 1.0,
        speed_coef: 1.0,
        reward_coef: 1.0,
    }]));

    app.init_resource::<PortalSpawnTracker>();

    // Add Systems
    app.add_systems(
        Update,
        (
            (portal_tick_logic, portal_spawn_logic).chain(),
            move_monsters,
            (manage_monster_lifecycle, despawn_dead_bodies),
            update_monster_health_ui,
            move_projectiles,
            projectile_collision,
        ),
    );

    // Spawn Portal manually
    let portal_entity = app
        .world_mut()
        .spawn((
            PortalRoot,
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
            MonsterScaling {
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

    // Add Capacity Upgrade (required for spawning)
    let capacity = app
        .world_mut()
        .spawn((
            UpgradeSlot {
                name: "Capacity".to_string(),
            },
            UpgradeableStat {
                level: 0.0,
                value: 10.0, // Enough capacity
                price: 100.0,
                value_strategy: GrowthStrategy::Static(10.0),
                price_strategy: GrowthStrategy::Static(100.0),
            },
        ))
        .id();

    app.world_mut()
        .entity_mut(portal_entity)
        .add_child(capacity);

    app
}

#[test]
fn test_initial_portal_spawn() {
    let mut app = setup_app();
    app.update();
    let mut portal_query = app.world_mut().query::<&PortalRoot>();
    assert_eq!(portal_query.iter(app.world()).count(), 1);
}

#[test]
fn test_enemy_spawning() {
    let mut app = setup_app();
    app.update();

    // Capture events
    #[derive(Resource, Default)]
    struct CapturedEvents(Vec<SpawnMonsterEvent>);
    app.init_resource::<CapturedEvents>();
    app.add_systems(
        Update,
        |mut r: MessageReader<SpawnMonsterEvent>, mut c: ResMut<CapturedEvents>| {
            for e in r.read() {
                c.0.push(e.clone());
            }
        },
    );

    // Advance time to trigger spawn (1.0s)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update();
    app.update(); // One more for event propagation?

    let captured = app.world().resource::<CapturedEvents>();
    assert_eq!(captured.0.len(), 1, "Should emit SpawnMonsterEvent");
}

#[test]
fn test_enemy_movement() {
    let mut app = setup_app();
    app.update();

    // Spawn enemy manually
    let start_pos = Vec3::new(0.0, 0.0, 0.0);
    let target_pos = Vec2::new(100.0, 0.0);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::default(),
            Transform::from_translation(start_pos),
            Monster {
                target_position: target_pos,
            },
            Health {
                current: 100.0,
                max: 100.0,
            },
            Reward(10.0),
            Speed(100.0),
            Lifetime::default(), // Required for lifecycle system
        ))
        .id();

    // Advance time 0.5s
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.5));
    }
    app.update();

    let transform = app.world().get::<Transform>(entity).unwrap();
    // Should move 50 units
    assert!(
        (transform.translation.x - 50.0).abs() < 1.0,
        "Position: {}",
        transform.translation.x
    );
}
