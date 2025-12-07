use bevy::prelude::*;
use void_core::VoidCorePlugin;
use void_assets::VoidAssetsPlugin;

pub struct VoidGameplayPlugin;

impl Plugin for VoidGameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
             app.add_plugins(VoidCorePlugin);
        }
        if !app.is_plugin_added::<VoidAssetsPlugin>() {
            app.add_plugins(VoidAssetsPlugin);
        }
        info!("Void Gameplay initialized");
    }
}
