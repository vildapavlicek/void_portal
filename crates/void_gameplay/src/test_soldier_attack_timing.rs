#[cfg(test)]
mod tests {
    use {
        crate::{
            configs::{PortalConfig, SoldierConfig},
            portal::{Enemy, Health, PortalSpawnTracker, SpawnIndex},
            soldier::{
                soldier_attack_logic, soldier_decision_logic, soldier_movement_logic, spawn_soldier,
                Projectile, Soldier,
            },
        },
        bevy::{prelude::*, time::TimePlugin, window::PrimaryWindow},
        void_core::events::EnemyKilled,
    };

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
        app.insert_resource(Time::<()>::default());
        app.add_message::<EnemyKilled>();

        app.world_mut().spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..default()
            },
            PrimaryWindow,
        ));

        // Required resources
        app.insert_resource(PortalConfig {
            spawn_timer: 1.0,
            base_void_shards_reward: 10.0,
            base_upgrade_price: 500.0,
            upgrade_price_increase_coef: 1.5,
            portal_top_offset: 100.0,
            base_enemy_health: 10.0,
            base_enemy_speed: 150.0,
            base_enemy_lifetime: 10.0,
            base_enemy_reward: 10.0,
        });
        app.insert_resource(PortalSpawnTracker(10));
        app.insert_resource(SoldierConfig {
            attack_timer: 1.0,
            projectile_speed: 400.0,
            projectile_damage: 20.0,
            projectile_lifetime: 2.0,
            attack_range: 150.0,
            move_speed: 100.0,
        });

        // Systems
        app.add_systems(
            Update,
            (
                spawn_soldier,
                soldier_movement_logic,
                soldier_decision_logic.after(soldier_movement_logic),
                soldier_attack_logic.after(soldier_decision_logic),
            ),
        );

        app
    }

    #[test]
    fn test_soldier_attacks_immediately_on_new_target() {
        let mut app = setup_app();

        // Spawn Soldier
        app.update();
        let soldier_entity = app.world_mut().query_filtered::<Entity, With<Soldier>>().single(app.world()).unwrap();

        // Move Soldier to (0,0) for simplicity
        app.world_mut().get_mut::<Transform>(soldier_entity).unwrap().translation = Vec3::ZERO;

        // Spawn Enemy in Range (at 50.0, range is 150.0)
        let _enemy_entity = app.world_mut().spawn((
            Transform::from_xyz(50.0, 0.0, 0.0),
            Enemy { target_position: Vec2::ZERO },
            SpawnIndex(0),
            Health { current: 10.0, max: 10.0 },
        )).id();

        // 1. Run update.
        // - soldier_decision_logic: Should acquire target and add Attacking.
        // - soldier_attack_logic: Should tick timer.
        // EXPECTATION: If fix is applied, it should fire immediately.
        // CURRENT: It should wait for timer (1.0s).
        app.update();

        // Check Projectile
        let projectile_count = app.world_mut().query::<&Projectile>().iter(app.world()).count();
        assert_eq!(projectile_count, 1, "Projectile should be spawned immediately upon acquiring new target");
    }
}
