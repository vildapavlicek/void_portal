use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct EnemyKilled {
    pub reward: f32,
}
