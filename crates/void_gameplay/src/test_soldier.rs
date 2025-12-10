#[cfg(test)]
mod tests {
    use {
        crate::{
            configs::PortalConfig, // Added import
            portal::{
                despawn_dead_bodies, handle_dying_enemies, Enemy, EnemySpawnTimer, Health, Portal,
                PortalSpawnTracker, SpawnIndex, Speed,
            },
            soldier::{
                move_projectiles, projectile_collision, soldier_attack_logic,
                soldier_decision_logic, soldier_movement_logic, spawn_soldier, Attacking, Moving,
                Projectile, Soldier,
            },
        },
        bevy::{prelude::*, time::TimePlugin, window::PrimaryWindow},
        void_core::events::EnemyKilled, // Added import
    };

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());
        app.insert_resource(Time::<()>::default());
        app.add_message::<EnemyKilled>(); // Added message registration

        app.world_mut().spawn((
            Window {
                // Bevy 0.17+ WindowResolution::new takes u32
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..default()
            },
            PrimaryWindow,
        ));

        // Insert PortalConfig resource (needed if spawn_portal is used, or generally good to have)
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

        // Spawn Portal (spawn_portal system needs PortalConfig, but here we spawn manually sometimes?
        // No, the test manually spawns a Portal entity below.
        // However, if we use `spawn_portal` system anywhere, we need the resource.
        // `spawn_soldier` uses `Portal` entity position.

        // Let's spawn the Portal entity manually as before, but ensure it matches component expectations if systems rely on them.
        // `spawn_soldier` queries `&Portal`.
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
            attack_range: 150.0,
            move_speed: 100.0,
        });

        app.add_systems(
            Update,
            (
                spawn_soldier,
                soldier_movement_logic,
                soldier_decision_logic.after(soldier_movement_logic),
                soldier_attack_logic.after(soldier_decision_logic),
                move_projectiles,
                projectile_collision.after(move_projectiles),
                handle_dying_enemies.after(projectile_collision),
                despawn_dead_bodies,
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
                crate::portal::Reward(10.0), // Added Reward component as required by despawn_dead_enemies
                Speed(150.0),                // Added Speed component
            ))
            .id();

        // 3. Acquire Target
        app.update(); // soldier_decision_logic runs, assigns target, likely Moving or Attacking
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
        app.update(); // decision -> attack logic runs

        // Check Projectile Spawned
        let _projectile_entity = {
            let mut query = app.world_mut().query::<&Projectile>();
            assert_eq!(query.iter(app.world()).count(), 1);
            app.world_mut()
                .query::<Entity>()
                .iter(app.world())
                .find(|e| *e != enemy_entity && app.world().get::<Soldier>(*e).is_none())
                .unwrap() // Find projectile entity roughly
        };

        // 5. Move Projectile to hit Enemy
        // Soldier moved to ~115 (Start -225 + 1.1*100 = -115).
        // Distance 115. Speed 400. Hit in ~0.2875s.
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(0.3));
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
                time.advance_by(std::time::Duration::from_secs_f32(0.3));
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
                Transform::default(),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(8),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
                crate::portal::Reward(10.0),
                Speed(150.0),
            ))
            .id();

        let enemy_b = app
            .world_mut()
            .spawn((
                Transform::default(),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(5),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
                crate::portal::Reward(10.0),
                Speed(150.0),
            ))
            .id();

        let _enemy_c = app
            .world_mut()
            .spawn((
                Transform::default(),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(9),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
                crate::portal::Reward(10.0),
                Speed(150.0),
            ))
            .id();

        // 1. Check targeting oldest (B)
        app.update(); // Decision logic runs
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

        let _enemy_a = app
            .world_mut()
            .spawn((
                Transform::default(),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(1),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
                crate::portal::Reward(10.0),
                Speed(150.0),
            ))
            .id();

        let enemy_b = app
            .world_mut()
            .spawn((
                Transform::default(),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(u32::MAX),
                Health {
                    current: 10.0,
                    max: 10.0,
                },
                crate::portal::Reward(10.0),
                Speed(150.0),
            ))
            .id();

        app.update(); // Decision logic
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

    #[test]
    fn test_soldier_movement_and_range() {
        let mut app = setup_app();

        // Spawn Soldier manually to control position
        app.update(); // Spawns soldier at default pos (0, -225, 0)

        let soldier_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Soldier>>()
            .single(app.world())
            .expect("Soldier should be spawned");

        // Move soldier to (0, -500)
        let mut soldier_transform = app
            .world_mut()
            .get_mut::<Transform>(soldier_entity)
            .expect("Soldier entity should have Transform");
        soldier_transform.translation = Vec3::new(0.0, -500.0, 0.0);

        // Attack Range is 150.
        // Move speed is 100.

        // Spawn Enemy at (0, 0)
        let enemy_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Enemy {
                    target_position: Vec2::ZERO,
                },
                SpawnIndex(0),
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                crate::portal::Reward(10.0),
                Speed(150.0),
            ))
            .id();

        app.update(); // soldier_decision_logic runs. Should pick target. Distance 500 > 150. Should add Moving.

        // Verify Target Acquired
        let soldier = app.world().get::<Soldier>(soldier_entity).unwrap();
        assert_eq!(soldier.target, Some(enemy_entity));

        // Verify Moving Component
        assert!(app.world().get::<Moving>(soldier_entity).is_some());
        assert!(app.world().get::<Attacking>(soldier_entity).is_none());

        // 1. Advance time 1 second.
        // Soldier should move 100 units towards (0,0). New pos: (0, -400)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(1.0));
        }
        app.update(); // movement logic runs.

        let soldier_transform = app.world().get::<Transform>(soldier_entity).unwrap();
        assert!(soldier_transform.translation.y > -500.0);
        assert!((soldier_transform.translation.y - -400.0).abs() < 1.0); // Approx -400

        // Check NO projectile (distance 400 > 150)
        assert_eq!(
            app.world_mut()
                .query::<&Projectile>()
                .iter(app.world())
                .count(),
            0
        );

        // 2. Advance time to get within range.
        // Current Y: -400. Target Y: -150 (boundary). Distance to travel: 250.
        // Speed 100. Need 2.5 seconds.
        // Let's go 2.6 seconds.
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(2.6));
        }
        app.update();

        // The system flow is:
        // Frame N (distance > range): Moving system moves soldier. New distance <= range? Removes Moving.
        // Frame N+1: Decision system sees no Moving/Attacking. Checks target. Distance <= range. Adds Attacking.
        // Frame N+1 (or same frame if ordered correctly?): Attacking system runs.

        // Check if Moving is removed
        assert!(app.world().get::<Moving>(soldier_entity).is_none());

        // Since we ran `app.update()`, and systems run in schedule...
        // If decision logic runs before movement/attack logic:
        // Frame X: Movement logic moves soldier into range. Removes Moving.
        // Frame X+1: Decision logic sees no action. Adds Attacking.

        // Let's check if we are in Attacking state now or if we need another update.
        // If Moving removed it in Frame X, and Decision runs before Movement (default parallel/ambiguous order unless explicit),
        // we might need another frame for Decision to pick it up.

        // Let's force another update just to be sure the state transition happens.
        app.update();

        assert!(app.world().get::<Attacking>(soldier_entity).is_some());

        let soldier_transform = app.world().get::<Transform>(soldier_entity).unwrap();
        let dist = soldier_transform.translation.distance(Vec3::ZERO);
        assert!(dist <= 150.0 + 2.0); // Allow small buffer

        // Check Projectile Spawned
        // Timer should tick.
        assert!(
            app.world_mut()
                .query::<&Projectile>()
                .iter(app.world())
                .count()
                > 0
        );
    }
}
