#[cfg(test)]
mod tests {
    // Import from the sibling module 'portal'
    use crate::portal::{
        enemy_lifetime, spawn_enemies, spawn_portal, Enemy, EnemySpawnTimer, Portal,
        PortalSpawnTracker,
    };
    use bevy::{prelude::*, time::TimePlugin, window::PrimaryWindow};

    #[test]
    fn test_portal_spawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // We don't add WindowPlugin to avoid window creation in tests

        // Spawn a window with specific resolution.
        app.world_mut().spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..default()
            },
            PrimaryWindow,
        ));

        // Add the systems
        app.add_systems(Update, spawn_portal);

        app.update();

        // Check if portal exists
        let mut portal_query = app.world_mut().query::<&Portal>();
        assert_eq!(portal_query.iter(app.world()).count(), 1);

        let mut transform_query = app.world_mut().query::<(&Transform, &Portal)>();
        let (transform, _) = transform_query.iter(app.world()).next().unwrap();

        // Window height 600, half is 300. Portal should be at 300 - 50 = 250.
        assert_eq!(transform.translation.y, 250.0);
    }

    #[test]
    fn test_enemy_spawn_and_lifetime() {
        let mut app = App::new();
        // Use MinimalPlugins but disable TimePlugin so we can control Time manually
        app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>());

        // Manually insert Time resource
        app.insert_resource(Time::<()>::default());

        app.world_mut().spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(800, 600),
                ..default()
            },
            PrimaryWindow,
        ));

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 250.0, 0.0), Portal));

        app.insert_resource(PortalSpawnTracker(0));

        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(
            7.5,
            TimerMode::Repeating,
        )));
        app.insert_resource(crate::configs::EnemyConfig {
            max_health: 100.0,
            lifetime: 10.0,
            speed: 100.0,
            spawn_limit: 5,
        });

        // Add systems
        app.add_systems(Update, (spawn_enemies, enemy_lifetime));

        // Tick time by 7.4s
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(7.4));
        }
        app.update();
        {
            let mut query = app.world_mut().query::<&Enemy>();
            assert_eq!(query.iter(app.world()).count(), 0);
        }

        // Tick by 0.2s (Total 7.6s) -> Should spawn
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(0.2));
        }
        app.update();
        {
            let mut query = app.world_mut().query::<&Enemy>();
            assert_eq!(query.iter(app.world()).count(), 1);
        }

        // Tick by 7.5s -> Spawn another
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(7.5));
        }
        app.update();
        {
            let mut query = app.world_mut().query::<&Enemy>();
            assert_eq!(query.iter(app.world()).count(), 2);
        }

        // Tick by 3.0s (Total time approx 18.1s)
        // First enemy spawned at 7.6s. Dies at 17.6s.
        // Current time will be 15.1 + 3.0 = 18.1s.
        // First enemy should die.
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(3.0));
        }
        app.update();

        {
            let mut query = app.world_mut().query::<&Enemy>();
            assert_eq!(query.iter(app.world()).count(), 1);
        }
    }
}
