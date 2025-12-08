#[cfg(test)]
mod tests {
    use {
        crate::{
            portal::{
                despawn_dead_enemies, enemy_lifetime, spawn_enemies, spawn_portal,
                update_enemy_health_ui, Enemy, EnemySpawnTimer, Health, Portal, PortalSpawnTracker,
                SpawnIndex,
            },
            soldier::{
                move_projectiles, projectile_collision, soldier_acquire_target, soldier_attack,
                spawn_soldier, Projectile, Soldier,
            },
        },
        bevy::{prelude::*, time::TimePlugin, window::PrimaryWindow},
    };

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
        app.insert_resource(Time::<()>::default());

        app.world_mut().spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..default()
            },
            PrimaryWindow,
        ));

        // Spawn Portal
        app.world_mut().spawn((Transform::default(), Portal));

        // Insert Tracker Resource (required for targeting)
        app.insert_resource(PortalSpawnTracker(10)); // Start at 10 to allow testing wrapping/subtraction

        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));
        app.insert_resource(crate::configs::SoldierConfig {
            attack_timer: 1.0,
            projectile_speed: 400.0,
            projectile_damage: 20.0,
            projectile_lifetime: 2.0,
        });

        app.add_systems(
            Update,
            (
                spawn_soldier,
                soldier_acquire_target,
                soldier_attack,
                move_projectiles,
                projectile_collision.after(move_projectiles),
                despawn_dead_enemies.after(projectile_collision),
            ),
        );
        app
    }

    #[test]
    fn test_soldier_spawn_and_combat() {
        let mut app = setup_app();

        // 1. Check Soldier Spawning
        app.update();
        let soldier_entity = {
            let mut soldier_query = app.world_mut().query::<(&Transform, &Soldier)>();
            let (transform, _) = soldier_query
                .iter(app.world())
                .next()
                .expect("Soldier should be spawned");
            assert_eq!(transform.translation.y, -225.0);
            soldier_query.iter(app.world()).next().unwrap().1.target
        };
        assert!(soldier_entity.is_none());

        // 2. Spawn an Enemy manually
        let enemy_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(9), // Tracker is at 10, so age = 10 - 9 = 1
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // 3. Acquire Target
        app.update(); // soldier_acquire_target runs
        {
            let mut soldier_query = app.world_mut().query::<&Soldier>();
            let soldier = soldier_query.iter(app.world()).next().unwrap();
            assert_eq!(soldier.target, Some(enemy_entity));
        }

        // 4. Attack (Soldier timer 1.0s, ticking...)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(1.1));
        }
        app.update(); // soldier_attack runs

        // Check Projectile Spawned
        let projectile_entity = {
            let mut query = app.world_mut().query::<&Projectile>();
            assert_eq!(query.iter(app.world()).count(), 1);
            app.world_mut()
                .query::<Entity>()
                .iter(app.world())
                .find(|e| *e != enemy_entity && app.world().get::<Soldier>(*e).is_none())
                .unwrap() // Find projectile entity roughly
        };

        // 5. Move Projectile to hit Enemy
        // Soldier at (0, -225). Enemy at (0, 0). Distance 225.
        // Projectile speed 400. Should hit in ~0.56s.
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(0.6));
        }
        app.update(); // move_projectiles, projectile_collision

        // Check Enemy Health
        {
            let mut query = app.world_mut().query::<&Health>();
            let health = query.get(app.world(), enemy_entity).unwrap();
            assert_eq!(health.current, 80.0); // 100 - 20
        }

        // Check Projectile Despawned
        {
            let mut query = app.world_mut().query::<&Projectile>();
            assert_eq!(query.iter(app.world()).count(), 0);
        }

        // 6. Kill Enemy
        // 4 more hits needed.
        for _ in 0..4 {
            // Cooldown
            {
                let mut time = app.world_mut().resource_mut::<Time>();
                time.advance_by(std::time::Duration::from_secs_f32(1.1));
            }
            app.update(); // fire
                          // Travel
            {
                let mut time = app.world_mut().resource_mut::<Time>();
                time.advance_by(std::time::Duration::from_secs_f32(0.6));
            }
            app.update(); // hit
        }

        // Check Enemy Dead/Despawned
        {
            let mut query = app.world_mut().query::<&Enemy>();
            assert_eq!(query.iter(app.world()).count(), 0);
        }
    }

    #[test]
    fn test_soldier_targeting_oldest() {
        let mut app = setup_app();
        app.update(); // Spawn soldier

        // Spawn 3 Enemies with different indices
        // Tracker at 10.
        // Enemy A: Index 8. Age = 2.
        // Enemy B: Index 5. Age = 5. (Oldest)
        // Enemy C: Index 9. Age = 1.

        let enemy_a = app
            .world_mut()
            .spawn((
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(8),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
            ))
            .id();

        let enemy_b = app
            .world_mut()
            .spawn((
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(5),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
            ))
            .id();

        let enemy_c = app
            .world_mut()
            .spawn((
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(9),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
            ))
            .id();

        // 1. Check targeting oldest (B)
        app.update();
        {
            let mut soldier_query = app.world_mut().query::<&Soldier>();
            let soldier = soldier_query.iter(app.world()).next().unwrap();
            assert_eq!(soldier.target, Some(enemy_b));
        }

        // 2. Kill B, check retargeting to next oldest (A)
        app.world_mut().entity_mut(enemy_b).despawn();
        app.update(); // Retarget
        {
            let mut soldier_query = app.world_mut().query::<&Soldier>();
            let soldier = soldier_query.iter(app.world()).next().unwrap();
            assert_eq!(soldier.target, Some(enemy_a));
        }
    }

    #[test]
    fn test_soldier_targeting_wrapping() {
        let mut app = setup_app();
        app.update();

        // Set tracker to 2 (just wrapped around u32::MAX)
        app.insert_resource(PortalSpawnTracker(2));

        // Enemy A: Index 1. Age = 2 - 1 = 1. (New)
        // Enemy B: Index u32::MAX. Age = 2 - u32::MAX = 3. (Oldest, wrapped)

        let enemy_a = app
            .world_mut()
            .spawn((
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(1),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
            ))
            .id();

        let enemy_b = app
            .world_mut()
            .spawn((
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(u32::MAX),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
            ))
            .id();

        app.update();
        {
            let mut soldier_query = app.world_mut().query::<&Soldier>();
            let soldier = soldier_query.iter(app.world()).next().unwrap();
            assert_eq!(
                soldier.target,
                Some(enemy_b),
                "Should target Enemy B because it is older across the wrap boundary"
            );
        }
    }
}
