use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct EnemyKilled {
    pub entity: Entity,
}
