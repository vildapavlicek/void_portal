#[cfg(test)]
mod tests {
    use super::*;
    use void_core::events::EnemyKilled;
    use crate::portal::{
        Portal, PortalSpawnTracker, EnemySpawnTimer, LoadedEnemy, PendingEnemyStats, spawn_enemies, AvailableEnemies
    };
    use crate::configs::{PortalConfig, EnemyConfig};
    use bevy::prelude::*;
    use bevy::window::{WindowResolution, PrimaryWindow};

    #[test]
    fn test_enemy_spawning() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_once()));
        app.init_resource::<PortalSpawnTracker>();
        // app.add_message::<EnemyKilled>(); handles resource initialization for messages usually.
        app.add_message::<EnemyKilled>();

        // Mock Window
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(800, 600),
                ..default()
            },
            PrimaryWindow,
        ));

        // Setup resources
        app.insert_resource(PortalConfig {
            spawn_timer: 1.0,
            base_void_shards_reward: 10.0,
            base_upgrade_price: 100.0,
            upgrade_price_increase_coef: 1.2,
            base_enemy_health: 100.0,
            base_enemy_speed: 50.0,
            base_enemy_lifetime: 10.0,
            base_enemy_reward: 5.0,
        });

        let mut available_enemies = AvailableEnemies::default();
        available_enemies.0.push(LoadedEnemy {
            config: EnemyConfig {
                health_coef: 1.0,
                lifetime_coef: 1.0,
                speed_coef: 1.0,
                spawn_limit: 5,
                reward_coef: 1.0,
                scene_path: "scenes/enemies/basic.scn.ron".to_string(),
            },
            scene: Handle::default(), // Use default handle
        });
        app.insert_resource(available_enemies);

        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(0.0, TimerMode::Once))); // Ready immediately

        // Spawn Portal
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Portal,
        ));

        app.add_systems(Update, spawn_enemies);

        app.update();

        // Verify Enemy Entity with PendingEnemyStats exists
        let mut query = app.world_mut().query::<&PendingEnemyStats>();
        assert_eq!(query.iter(app.world()).count(), 1);

        let pending_stats = query.iter(app.world()).next().unwrap();
        assert_eq!(pending_stats.max_health, 100.0);
    }
}
