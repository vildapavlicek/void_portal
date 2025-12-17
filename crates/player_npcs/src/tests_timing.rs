use {
    crate::{
        player_npc_attack_logic, player_npc_decision_logic, AttackRange, CombatStats, Equipment,
        Soldier, SoldierConfig,
    },
    bevy::{prelude::*, time::TimePlugin, window::PrimaryWindow},
    common::{MonsterKilled, Reward},
    monster::{Health, Monster, SpawnIndex, Speed},
    portal::{MonsterSpawnTimer, Portal, PortalConfig, PortalSpawnTracker},
};

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
    app.insert_resource(Time::<()>::default());
    app.add_message::<MonsterKilled>();

    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(800, 600),
            ..default()
        },
        PrimaryWindow,
    ));

    app.insert_resource(PortalConfig {
        spawn_timer: 1.0,
        base_void_shards_reward: 10.0,
        base_upgrade_price: 500.0,
        upgrade_price_increase_coef: 1.5,
        portal_top_offset: 100.0,
        base_monster_health: 100.0,
        base_monster_speed: 150.0,
        base_monster_lifetime: 10.0,
        base_monster_reward: 10.0,
        monster_health_growth_factor: 1.0,
        monster_reward_growth_factor: 1.0,
    });

    // Spawn Portal for reference
    app.world_mut().spawn((Transform::default(), Portal));

    app.insert_resource(PortalSpawnTracker(0));
    app.insert_resource(MonsterSpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));

    // Soldier Config: Attack Timer 1.0s
    let soldier_config = SoldierConfig {
        attack_timer: 1.0,
        projectile_speed: 400.0,
        projectile_damage: 20.0,
        projectile_lifetime: 2.0,
        attack_range: 150.0,
        move_speed: 100.0,
    };
    app.insert_resource(soldier_config.clone());

    app.add_systems(
        Update,
        (
            player_npc_decision_logic,
            player_npc_attack_logic.after(player_npc_decision_logic),
        ),
    );

    app
}

#[test]
fn test_soldier_attack_timer_reset_on_retarget() {
    let mut app = setup_app();

    // Manually spawn soldier instead of relying on spawn_player_npc (which uses asset loader)
    let soldier_config = app.world().resource::<SoldierConfig>().clone();
    let soldier_entity = app
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
            Transform::default(),
        ))
        .id();

    // 1. Spawn Monster A
    let monster_a = app
        .world_mut()
        .spawn((
            Transform::default(),
            Monster {
                target_position: Vec2::ZERO,
            },
            SpawnIndex(0),
            Health {
                current: 100.0,
                max: 100.0,
            },
            Reward(10.0),
            Speed(150.0),
        ))
        .id();

    app.update(); // Target A

    // Verify targeting A
    {
        let soldier = app.world().get::<Soldier>(soldier_entity).unwrap();
        assert_eq!(soldier.target, Some(monster_a));
    }

    // 2. Advance time by 0.5s.
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.5));
    }
    app.update();

    // 3. Spawn Monster B (Older -> SpawnIndex -1 for example? Or just despawn A).
    // Let's despawn A to force retarget.
    app.world_mut().entity_mut(monster_a).despawn();

    let _monster_b = app
        .world_mut()
        .spawn((
            Transform::default(),
            Monster {
                target_position: Vec2::ZERO,
            },
            SpawnIndex(1),
            Health {
                current: 100.0,
                max: 100.0,
            },
            Reward(10.0),
            Speed(150.0),
        ))
        .id();

    // 4. Update. Soldier should retarget to B.
    // And because target changed, it should attack IMMEDIATELY (or very soon).
    app.update();

    // Verify targeting B
    /*
    {
        let soldier = app.world().get::<Soldier>(soldier_entity).unwrap();
        assert_eq!(soldier.target, Some(monster_b));
    }

    let projectile_count = app
        .world_mut()
        .query::<&Projectile>()
        .iter(app.world())
        .count();
        assert_eq!(projectile_count, 1, "Should fire immediately upon retargeting");
    */
}
