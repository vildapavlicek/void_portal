use bevy::prelude::*;

pub mod events;
pub use events::*;

pub struct VoidCorePlugin;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

impl Plugin for VoidCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_message::<EnemyKilled>();
        app.add_systems(Startup, setup_camera);
        info!("Void Core initialized");
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    debug!("Camera setup complete");
}
