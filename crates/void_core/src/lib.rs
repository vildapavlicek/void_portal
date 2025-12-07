use bevy::prelude::*;

pub struct VoidCorePlugin;

impl Plugin for VoidCorePlugin {
    fn build(&self, _app: &mut App) {
        info!("Void Core initialized");
    }
}
