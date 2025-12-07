use bevy::prelude::*;

pub struct VoidCorePlugin;

impl Plugin for VoidCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        info!("Void Core initialized");
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    debug!("Camera setup complete");
}
