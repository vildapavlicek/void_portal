use bevy::prelude::*;
use void_core::VoidCorePlugin;
use void_assets::VoidAssetsPlugin;

mod portal;
use portal::{spawn_portal, spawn_enemies, move_enemies, enemy_lifetime, EnemySpawnTimer};

#[cfg(test)]
mod tests;

pub struct VoidGameplayPlugin;

impl Plugin for VoidGameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
             app.add_plugins(VoidCorePlugin);
        }
        if !app.is_plugin_added::<VoidAssetsPlugin>() {
            app.add_plugins(VoidAssetsPlugin);
        }

        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(7.5, TimerMode::Repeating)));

        app.add_systems(Update, (
            spawn_portal,
            spawn_enemies,
            move_enemies,
            enemy_lifetime
        ));

        info!("Void Gameplay initialized");
    }
}
