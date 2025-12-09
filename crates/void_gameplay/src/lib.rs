use {
    bevy::{asset::LoadedFolder, prelude::*},
    void_core::{GameState, VoidCorePlugin},
};

mod configs;
mod portal;
mod soldier;

use {
    configs::{EnemyConfig, PortalConfig, SoldierConfig},
    portal::{
        despawn_dead_enemies, enemy_lifetime, move_enemies, on_enemy_spawned, spawn_enemies,
        spawn_portal, update_enemy_health_ui, AvailableEnemies, Enemy, EnemySpawnTimer, Health,
        Lifetime, PortalSpawnTracker, Reward, SpawnIndex, Speed,
    },
    soldier::{
        move_projectiles, projectile_collision, soldier_attack_logic, soldier_decision_logic,
        soldier_movement_logic, spawn_soldier,
    },
};

#[cfg(test)]
mod test_soldier;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod test_events;

pub struct VoidGameplayPlugin;

#[derive(Resource, Default)]
struct GameConfigHandles {
    portal: Handle<PortalConfig>,
    enemies_folder: Handle<LoadedFolder>,
    soldier: Handle<SoldierConfig>,
}

impl Plugin for VoidGameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
            app.add_plugins(VoidCorePlugin);
        }
        if !app.is_plugin_added::<VoidAssetsPlugin>() {
            app.add_plugins(VoidAssetsPlugin);
        }

        // Register types for Scene usage
        app.register_type::<Enemy>();
        app.register_type::<Health>();
        app.register_type::<Lifetime>();
        app.register_type::<Reward>();
        app.register_type::<Speed>();
        app.register_type::<SpawnIndex>();
        app.register_type::<PendingEnemyStats>();
        // Portal
        app.register_type::<crate::portal::EnemySpawner>();
        app.register_type::<crate::portal::VoidShardsReward>();
        app.register_type::<crate::portal::UpgradePrice>();
        app.register_type::<crate::portal::UpgradeCoef>();
        app.register_type::<crate::portal::Portal>();
        // Soldier
        app.register_type::<crate::soldier::Soldier>();
        app.register_type::<crate::soldier::AttackRange>();
        app.register_type::<crate::soldier::MoveSpeed>();
        app.register_type::<crate::soldier::ProjectileStats>();

        // Ensure core Bevy types used in scene are registered (usually they are by DefaultPlugins)
        app.register_type::<Text2d>();
        app.register_type::<TextFont>();
        app.register_type::<TextColor>();
        app.register_type::<Sprite>();
        app.register_type::<Transform>();

        // Use specific extensions to disambiguate different RON config types
        app.add_plugins((
            RonAssetPlugin::<PortalConfig>::new(&["portal.ron"]),
            RonAssetPlugin::<EnemyConfig>::new(&["enemy.ron"]),
            RonAssetPlugin::<SoldierConfig>::new(&["soldier.ron"]),
        ));

        app.init_resource::<GameConfigHandles>();
        app.init_resource::<PortalSpawnTracker>();
        app.init_resource::<AvailableEnemies>();

        app.add_observer(on_enemy_spawned);

        app.add_systems(Startup, start_loading);
        app.add_systems(
            Update,
            check_assets_ready.run_if(in_state(GameState::Loading)),
        );

        app.add_systems(OnEnter(GameState::Playing), (spawn_portal, spawn_soldier));

        app.add_systems(
            Update,
            (
                spawn_enemies,
                move_enemies,
                enemy_lifetime,
                despawn_dead_enemies,
                update_enemy_health_ui,
                soldier_decision_logic,
                soldier_movement_logic,
                soldier_attack_logic,
                move_projectiles,
                projectile_collision,
            )
                .run_if(in_state(GameState::Playing)),
        );

        info!("Void Gameplay initialized");
    }
}

fn start_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut handles: ResMut<GameConfigHandles>,
) {
    handles.portal = asset_server.load("configs/main.portal.ron");
    handles.enemies_folder = asset_server.load_folder("configs/enemies");
    handles.soldier = asset_server.load("configs/main.soldier.ron");

    // Spawn a simple loading text
    commands.spawn((
        Text2d::new("Loading..."),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        LoadingText,
    ));
}

#[derive(Component)]
struct LoadingText;

fn check_assets_ready(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    handles: Res<GameConfigHandles>,
    portal_assets: Res<Assets<PortalConfig>>,
    soldier_assets: Res<Assets<SoldierConfig>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    enemy_assets: Res<Assets<EnemyConfig>>,
    mut available_enemies: ResMut<AvailableEnemies>,
    mut next_state: ResMut<NextState<GameState>>,
    loading_text_query: Query<Entity, With<LoadingText>>,
) {
    if let (Some(portal), Some(soldier), Some(enemies_folder)) = (
        portal_assets.get(&handles.portal),
        soldier_assets.get(&handles.soldier),
        loaded_folders.get(&handles.enemies_folder),
    ) {
        // Insert singleton resources
        commands.insert_resource(portal.clone());
        commands.insert_resource(soldier.clone());

        // Process enemies folder
        available_enemies.0.clear();
        for handle in &enemies_folder.handles {
            // Cast untyped handle to typed handle
            let typed_handle: Handle<EnemyConfig> = handle.clone().typed();
            if let Some(config) = enemy_assets.get(&typed_handle) {
                let scene_handle = asset_server.load(&config.scene_path);
                available_enemies.0.push(LoadedEnemy {
                    config: config.clone(),
                    scene: scene_handle,
                });
            }
        }

        if available_enemies.0.is_empty() {
            warn!("No enemies loaded from configs/enemies/");
        } else {
             info!("Loaded {} enemy configs", available_enemies.0.len());
        }

        // Initialize EnemySpawnTimer from config
        commands.insert_resource(EnemySpawnTimer(Timer::from_seconds(
            portal.spawn_timer,
            TimerMode::Repeating,
        )));

        // Despawn loading text
        for entity in loading_text_query.iter() {
            commands.entity(entity).despawn();
        }

        info!("Configs loaded. Transitioning to Playing.");
        next_state.set(GameState::Playing);
    }
}
