use bevy::prelude::*;
use void_core::{VoidCorePlugin, GameState};
use void_assets::VoidAssetsPlugin;
use bevy_common_assets::ron::RonAssetPlugin;

mod portal;
mod soldier;
mod configs;

use configs::{GlobalConfig, EnemyConfig, SoldierConfig};
use portal::{
    spawn_portal, spawn_enemies, move_enemies, enemy_lifetime, despawn_dead_enemies,
    update_enemy_health_ui, EnemySpawnTimer
};
use soldier::{
    spawn_soldier, soldier_acquire_target, soldier_attack, move_projectiles, projectile_collision
};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod test_soldier;

pub struct VoidGameplayPlugin;

#[derive(Resource, Default)]
struct GameConfigHandles {
    global: Handle<GlobalConfig>,
    enemy: Handle<EnemyConfig>,
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

        app.add_plugins((
            RonAssetPlugin::<GlobalConfig>::new(&["global.ron"]),
            RonAssetPlugin::<EnemyConfig>::new(&["enemy.ron"]),
            RonAssetPlugin::<SoldierConfig>::new(&["soldier.ron"]),
        ));

        app.init_resource::<GameConfigHandles>();

        // Remove the hardcoded timer resource here, it will be added when loading is done
        // However, existing systems might need it to exist or we should add it with a dummy value until loaded?
        // But since we switch to Playing state only after loading, systems in Playing won't run yet.
        // Wait, current systems are in Update without run conditions. They run always.
        // I need to change that.

        app.add_systems(Startup, start_loading);
        app.add_systems(Update, check_assets_ready.run_if(in_state(GameState::Loading)));

        app.add_systems(OnEnter(GameState::Playing), (
             spawn_portal,
             spawn_soldier,
        ));

        app.add_systems(Update, (
            spawn_enemies,
            move_enemies,
            enemy_lifetime,
            despawn_dead_enemies,
            update_enemy_health_ui,
            soldier_acquire_target,
            soldier_attack,
            move_projectiles,
            projectile_collision,
        ).run_if(in_state(GameState::Playing)));

        info!("Void Gameplay initialized");
    }
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>, mut handles: ResMut<GameConfigHandles>) {
    handles.global = asset_server.load("configs/global.ron");
    handles.enemy = asset_server.load("configs/enemy.ron");
    handles.soldier = asset_server.load("configs/soldier.ron");

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
    handles: Res<GameConfigHandles>,
    global_assets: Res<Assets<GlobalConfig>>,
    enemy_assets: Res<Assets<EnemyConfig>>,
    soldier_assets: Res<Assets<SoldierConfig>>,
    mut next_state: ResMut<NextState<GameState>>,
    loading_text_query: Query<Entity, With<LoadingText>>,
) {
    if let (Some(global), Some(enemy), Some(soldier)) = (
        global_assets.get(&handles.global),
        enemy_assets.get(&handles.enemy),
        soldier_assets.get(&handles.soldier),
    ) {
        // Insert resources
        commands.insert_resource(global.clone());
        commands.insert_resource(enemy.clone());
        commands.insert_resource(soldier.clone());

        // Initialize EnemySpawnTimer from config
        commands.insert_resource(EnemySpawnTimer(Timer::from_seconds(global.spawn_timer, TimerMode::Repeating)));

        // Despawn loading text
        for entity in loading_text_query.iter() {
            commands.entity(entity).despawn();
        }

        info!("Configs loaded. Transitioning to Playing.");
        next_state.set(GameState::Playing);
    }
}
