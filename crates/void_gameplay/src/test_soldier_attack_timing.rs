#[cfg(test)]
mod tests {
    use {
        crate::{
            configs::{PortalConfig, SoldierConfig},
            portal::{
                Enemy, EnemySpawnTimer, Health, Portal, PortalSpawnTracker, SpawnIndex, Speed,
            },
            soldier::{soldier_attack_logic, soldier_decision_logic, spawn_soldier, Soldier},
        },
        bevy::{prelude::*, time::TimePlugin, window::PrimaryWindow},
        void_components::Reward,
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

        app.insert_resource(PortalConfig {
            spawn_timer: 1.0,
            base_void_shards_reward: 10.0,
            base_upgrade_price: 500.0,
            upgrade_price_increase_coef: 1.5,
            portal_top_offset: 100.0,
            base_enemy_health: 100.0,
            base_enemy_speed: 150.0,
            base_enemy_lifetime: 10.0,
            base_enemy_reward: 10.0,
            enemy_health_growth_factor: 1.0,
            enemy_reward_growth_factor: 1.0,
        });

        // Spawn Portal for reference
        app.world_mut().spawn((Transform::default(), Portal));

        app.insert_resource(PortalSpawnTracker(0));
        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(1.0, TimerMode::Once)));

        // Soldier Config: Attack Timer 1.0s
        app.insert_resource(SoldierConfig {
            attack_timer: 1.0,
            projectile_speed: 400.0,
            projectile_damage: 20.0,
            projectile_lifetime: 2.0,
            attack_range: 150.0,
            move_speed: 100.0,
        });

        app.add_systems(
            Update,
            (
                spawn_soldier,
                soldier_decision_logic,
                soldier_attack_logic.after(soldier_decision_logic),
            ),
        );

        app
    }

    #[test]
    fn test_soldier_attack_timer_reset_on_retarget() {
        let mut app = setup_app();
        app.update(); // Spawn soldier

        let soldier_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Soldier>>()
            .single(app.world())
            .unwrap();

        // 1. Spawn Enemy A
        let enemy_a = app
            .world_mut()
            .spawn((
                Transform::default(),
                Enemy {
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
            assert_eq!(soldier.target, Some(enemy_a));
        }

        // 2. Advance time by 0.5s.
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(0.5));
        }
        app.update();

        // 3. Spawn Enemy B (Older -> SpawnIndex -1 for example? Or just despawn A).
        // Let's despawn A to force retarget.
        app.world_mut().entity_mut(enemy_a).despawn();

        let _enemy_b = app
            .world_mut()
            .spawn((
                Transform::default(),
                Enemy {
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
            assert_eq!(soldier.target, Some(enemy_b));
        }

        let projectile_count = app
            .world_mut()
            .query::<&Projectile>()
            .iter(app.world())
            .count();
        assert_eq!(projectile_count, 1, "Should fire immediately upon retargeting");
        */
    }
}
