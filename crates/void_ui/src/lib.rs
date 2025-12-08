use {bevy::prelude::*, void_core::VoidCorePlugin};

pub struct VoidUiPlugin;

impl Plugin for VoidUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VoidCorePlugin>() {
            app.add_plugins(VoidCorePlugin);
        }
        info!("Void UI initialized");
    }
}
