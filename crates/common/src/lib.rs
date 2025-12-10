use bevy::prelude::*;

pub mod events;
pub use events::*;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Reward(pub f32);

#[derive(Component, Debug, Clone, Reflect)]
pub struct Dead {
    pub despawn_timer: Timer,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}
