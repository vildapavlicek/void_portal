use bevy::prelude::*;
use void_core::VoidCorePlugin;
use void_assets::VoidAssetsPlugin;

mod portal;
mod soldier;

use portal::{
    spawn_portal, spawn_enemies, move_enemies, enemy_lifetime, despawn_dead_enemies,
    update_enemy_health_ui, EnemySpawnTimer
};
use soldier::{
    spawn_soldier, soldier_acquire_target, soldier_attack, move_projectiles, projectile_collision
};

use void_core::config::{EnemyConfig, SoldierConfig};

#[derive(Resource)]
pub struct GameConfigHandles {
    pub enemy: Handle<EnemyConfig>,
    pub soldier: Handle<SoldierConfig>,
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod test_soldier;

pub struct VoidGameplayPlugin;

impl Plugin for VoidGameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
             app.add_plugins(VoidCorePlugin);
        }
        if !app.is_plugin_added::<VoidAssetsPlugin>() {
            app.add_plugins(VoidAssetsPlugin);
        }

        app.add_systems(Startup, load_configs);
        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(7.5, TimerMode::Repeating)));

        app.add_systems(Update, (
            spawn_portal,
            spawn_enemies,
            move_enemies,
            enemy_lifetime,
            despawn_dead_enemies,
            update_enemy_health_ui,
            spawn_soldier,
            soldier_acquire_target,
            soldier_attack,
            move_projectiles,
            projectile_collision,
        ));

        info!("Void Gameplay initialized");
    }
}

fn load_configs(mut commands: Commands, asset_server: Res<AssetServer>) {
    let enemy_handle = asset_server.load("configs/enemy.ron");
    let soldier_handle = asset_server.load("configs/soldier.ron");

    commands.insert_resource(GameConfigHandles {
        enemy: enemy_handle,
        soldier: soldier_handle,
    });
}
