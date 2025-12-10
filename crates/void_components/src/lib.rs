use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Reward(pub f32);

#[derive(Component, Debug, Clone, Reflect)]
pub struct Dead {
    pub despawn_timer: Timer,
}
