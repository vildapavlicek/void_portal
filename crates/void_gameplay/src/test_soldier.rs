#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use bevy::time::TimePlugin;
    use crate::portal::{spawn_portal, spawn_enemies, enemy_lifetime, despawn_dead_enemies, update_enemy_health_ui, Portal, Enemy, Health, EnemySpawnTimer};
    use crate::soldier::{spawn_soldier, soldier_acquire_target, soldier_attack, move_projectiles, projectile_collision, Soldier, Projectile};

    #[test]
    fn test_soldier_spawn_and_combat() {
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

        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.insert_resource(crate::configs::SoldierConfig {
            attack_timer: 1.0,
            projectile_speed: 400.0,
            projectile_damage: 20.0,
            projectile_lifetime: 2.0,
        });

        app.add_systems(Update, (
            spawn_soldier,
            soldier_acquire_target,
            soldier_attack,
            move_projectiles,
            projectile_collision.after(move_projectiles),
            despawn_dead_enemies.after(projectile_collision)
        ));

        // 1. Check Soldier Spawning
        app.update();
        let soldier_entity = {
            let mut query = app.world_mut().query::<Entity>();
            let entities: Vec<Entity> = query.iter(app.world()).collect();
            let mut soldier_query = app.world_mut().query::<(&Transform, &Soldier)>();
            let (transform, _) = soldier_query.iter(app.world()).next().expect("Soldier should be spawned");
            // Window height 600. Bottom 25% starts at -300. Middle of bottom 25% is -300 + (600 * 0.125) = -300 + 75 = -225.
            assert_eq!(transform.translation.y, -225.0);
            soldier_query.iter(app.world()).next().unwrap().1.target
        };
        assert!(soldier_entity.is_none());

        // 2. Spawn an Enemy manually
        let enemy_entity = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Enemy { target_position: Vec2::ZERO },
            Health { current: 100.0, max: 100.0 },
        )).id();

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
            app.world_mut().query::<Entity>().iter(app.world()).find(|e| *e != enemy_entity && app.world().get::<Soldier>(*e).is_none()).unwrap() // Find projectile entity roughly
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
}
