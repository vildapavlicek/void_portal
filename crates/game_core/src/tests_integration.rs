use {
    bevy::{prelude::*, time::TimePlugin},
    common::{EnemyKilled, Reward},
    enemy::{
        despawn_dead_bodies, enemy_lifetime, handle_dying_enemies, move_enemies,
        update_enemy_health_ui, AvailableEnemies, Enemy, EnemyConfig,
    },
    player_npcs::{
        move_projectiles, player_npc_attack_logic, player_npc_decision_logic,
        player_npc_movement_logic, projectile_collision, spawn_player_npc, SoldierConfig,
    },
    portal::{
        spawn_enemies, spawn_portal, EnemySpawnTimer, Portal, PortalConfig, PortalSpawnTracker,
    },
};

// Helper to setup the app with necessary resources
fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());
    app.add_message::<EnemyKilled>();

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
        base_enemy_health: 100.0,
        base_enemy_speed: 100.0,
        base_enemy_lifetime: 5.0,
        base_enemy_reward: 5.0,
        enemy_health_growth_factor: 1.0,
        enemy_reward_growth_factor: 1.0,
    });

    app.insert_resource(AvailableEnemies(vec![EnemyConfig {
        health_coef: 1.0,
        lifetime_coef: 1.0,
        speed_coef: 1.0,
        reward_coef: 1.0,
        spawn_limit: 10,
    }]));

    app.insert_resource(SoldierConfig {
        attack_timer: 1.0,
        projectile_speed: 100.0,
        projectile_damage: 10.0,
        projectile_lifetime: 2.0,
        attack_range: 200.0,
        move_speed: 100.0,
    });

    app.init_resource::<PortalSpawnTracker>();
    app.insert_resource(EnemySpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));

    // Add Systems
    app.add_systems(
        Update,
        (
            spawn_portal,
            spawn_enemies,
            move_enemies,
            enemy_lifetime,
            handle_dying_enemies,
            despawn_dead_bodies,
            update_enemy_health_ui,
            spawn_player_npc,
            player_npc_decision_logic,
            player_npc_movement_logic,
            player_npc_attack_logic,
            move_projectiles,
            projectile_collision,
        ),
    );

    app
}

#[test]
fn test_initial_portal_spawn() {
    let mut app = setup_app();
    app.update();

    // Check if Portal spawned
    let mut portal_query = app.world_mut().query::<&Portal>();
    assert_eq!(portal_query.iter(app.world()).count(), 1);
}

#[test]
fn test_enemy_spawning() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Advance time to trigger spawn (1.0s)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.1));
    }
    app.update(); // Spawn enemy

    let mut enemy_query = app.world_mut().query::<&Enemy>();
    assert_eq!(enemy_query.iter(app.world()).count(), 1);
}

#[test]
fn test_enemy_movement() {
    let mut app = setup_app();
    app.update(); // Spawn portal

    // Spawn enemy manually to control position
    let start_pos = Vec3::new(0.0, 0.0, 0.0);
    let target_pos = Vec2::new(100.0, 0.0);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::default(),
            Transform::from_translation(start_pos),
            Enemy {
                target_position: target_pos,
            },
            enemy::SpawnIndex(0),
            enemy::Health {
                current: 100.0,
                max: 100.0,
            },
            Reward(10.0),
            enemy::Speed(100.0), // 100 units/sec
        ))
        .id();

    // Advance time 0.5s
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.5));
    }
    app.update(); // Move

    let transform = app.world().get::<Transform>(entity).unwrap();
    // Should move 50 units towards (100,0) -> (50, 0)
    assert!((transform.translation.x - 50.0).abs() < 1.0);
}
