use {bevy::prelude::*, void_core::VoidCorePlugin};

pub struct VoidAssetsPlugin;

impl Plugin for VoidAssetsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
            app.add_plugins(VoidCorePlugin);
        }
        info!("Void Assets initialized");
    }
}
